mod args;
mod config;
mod display;
mod git;
mod github;
mod repo;

use args::Args;
use clap::Parser;
use colored::Colorize;
use config::Config;
use display::display_repos_status;
use git::status::check_git_status;
use github::api::fetch_github_prs;
use repo::filters::RepoFilters;
use repo::status::RepoStatus;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Load config file if it exists
    let config = config::load_config(args.config.as_deref())?;

    // Load protected branches if specified
    let mut protected_branches = config.github.protected_branches.clone();
    if let Some(path) = &args.protected_branches_file {
        let file_branches = config::load_protected_branches(path)?;
        protected_branches.extend(file_branches);
    }

    // Create a HashSet for faster lookups
    let protected_branches: HashSet<String> = protected_branches.into_iter().collect();

    // Merge config with command line arguments
    let filters = RepoFilters {
        check_untracked: args.check_untracked,
        check_unstaged: args.check_unstaged,
        check_ahead: args.check_ahead,
        check_no_remotes: args.check_no_remotes,
        check_branch: args.check_branch,
        // Command line args should take precedence over config
        check_prs: args.check_prs,
        // For include_draft_prs, we should only apply it if check_prs is true,
        // but still respect the command line value
        include_draft_prs: args.include_draft_prs,
    };

    // Get GitHub token
    let github_token = args.github_token.or(config.github.token.clone());

    // Configure thread pool using either command line or config value
    let threads = args.threads.or(config.threads);
    if let Some(threads) = threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .unwrap();
    }

    // Resolve and display target directory
    let root = if args.directory == std::path::PathBuf::from(".") {
        std::env::current_dir().unwrap()
    } else {
        args.directory.clone()
    };

    println!(
        "{}",
        format!("[?] Target directory: {}", root.display())
            .cyan()
            .bold()
    );

    // Get all entries in directory
    let entries: Vec<_> = fs::read_dir(&root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|entry| entry.path().join(".git").is_dir())
        .collect();

    // Store the count before processing
    let repo_count = entries.len();

    if args.debug {
        println!("[-] Found {} git repositories", repo_count);
        println!("[-] Active filters: {:?}", filters);
    }

    // Process repositories in parallel
    let results = Arc::new(Mutex::new(Vec::new()));

    // We use a specific collection for handles to avoid consuming 'entries'
    let handles: Vec<_> = entries
        .iter() // Use iter() instead of into_iter() to keep ownership
        .map(|entry| {
            let repo_path = entry.path();
            let filters = filters.clone();
            let results = Arc::clone(&results);
            let token = github_token.clone();
            let protected_branches = protected_branches.clone();
            let debug = args.debug;

            tokio::spawn(async move {
                if debug {
                    println!("[-] Checking repository {}", repo_path.display());
                }

                // Get git status
                let git_status = check_git_status(&repo_path, &filters, debug);

                // Get GitHub PR info if needed
                let mut prs = Vec::new();
                if filters.check_prs {
                    if let Ok(repo_prs) =
                        fetch_github_prs(&repo_path, token.as_deref(), debug).await
                    {
                        if debug {
                            println!(
                                "[-] Found {} open PRs for {}",
                                repo_prs.len(),
                                repo_path.display()
                            );
                        }

                        // Filter draft PRs if needed
                        prs = if filters.include_draft_prs {
                            repo_prs
                        } else {
                            repo_prs.into_iter().filter(|pr| !pr.is_draft).collect()
                        };
                    } else if debug {
                        println!("[-] Failed to fetch PRs for {}", repo_path.display());
                    }
                }

                // Combine into repo status
                let repo_status = RepoStatus::new(git_status, prs, protected_branches);

                let mut results_guard = results.lock().unwrap();
                if args.all || repo_status.has_issues(&filters) {
                    results_guard.push((repo_path.clone(), repo_status));
                }
            })
        })
        .collect();

    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }

    // Display results
    let results_guard = results.lock().unwrap();
    display_repos_status(&results_guard, &filters);

    println!("Scan complete: {} repositories processed", repo_count);

    Ok(())
}
