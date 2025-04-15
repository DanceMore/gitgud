use clap::Parser;
use colored::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Opinionated Git repository scanner to keep you Organized and On Task
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory containing Git repositories to scan
    #[arg(default_value = ".")]
    directory: PathBuf,

    /// Enable verbose debug output
    #[arg(short, long)]
    debug: bool,

    /// Number of threads to use (default: auto)
    #[arg(short, long)]
    threads: Option<usize>,

    /// Show all repositories, even those with no issues
    #[arg(short, long)]
    all: bool,

    /// Path to config file (default: ~/.gitgud.toml)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Check for untracked files
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    check_untracked: bool,

    /// Check for unstaged changes
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    check_unstaged: bool,

    /// Check if branch is ahead of remote
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    check_ahead: bool,

    /// Check if repository has no remotes
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    check_no_remotes: bool,

    /// Check if branch is not a default branch (main, master, develop)
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    check_branch: bool,
}

/// Config file structure that can be loaded from TOML
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    #[serde(default = "default_threads")]
    threads: Option<usize>,

    #[serde(default = "default_true")]
    check_untracked: bool,

    #[serde(default = "default_true")]
    check_unstaged: bool,

    #[serde(default = "default_true")]
    check_ahead: bool,

    #[serde(default = "default_true")]
    check_no_remotes: bool,

    #[serde(default = "default_true")]
    check_branch: bool,

    #[serde(default)]
    default_paths: Vec<PathBuf>,
}

fn default_threads() -> Option<usize> {
    None
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threads: None,
            check_untracked: true,
            check_unstaged: true,
            check_ahead: true,
            check_no_remotes: true,
            check_branch: true,
            default_paths: vec![],
        }
    }
}

#[derive(Debug)]
struct RepoFilters {
    check_untracked: bool,
    check_unstaged: bool,
    check_ahead: bool,
    check_no_remotes: bool,
    check_branch: bool,
}

struct RepoStatus {
    untracked_files: bool,
    unstaged_changes: bool,
    ahead_of_remote: bool,
    missing_remote: bool,
    non_default_branch: Option<String>,
}

impl RepoStatus {
    fn has_issues(&self, filters: &RepoFilters) -> bool {
        (filters.check_untracked && self.untracked_files)
            || (filters.check_unstaged && self.unstaged_changes)
            || (filters.check_ahead && self.ahead_of_remote)
            || (filters.check_no_remotes && self.missing_remote)
            || (filters.check_branch && self.non_default_branch.is_some())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Load config file if it exists
    let config = load_config(args.config.as_deref())?;

    // Merge config with command line arguments
    let filters = RepoFilters {
        check_untracked: args.check_untracked,
        check_unstaged: args.check_unstaged,
        check_ahead: args.check_ahead,
        check_no_remotes: args.check_no_remotes,
        check_branch: args.check_branch,
    };

    // Configure thread pool using either command line or config value
    let threads = args.threads.or(config.threads);
    if let Some(threads) = threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .unwrap();
    }

    // Resolve and display target directory
    let root = if args.directory == PathBuf::from(".") {
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

    if args.debug {
        println!("[-] Found {} git repositories", entries.len());
        println!("[-] Active filters: {:?}", filters);
    }

    // Process repositories in parallel
    let results = Arc::new(Mutex::new(Vec::new()));

    entries.par_iter().for_each(|entry| {
        let repo_path = entry.path();
        if args.debug {
            println!("[-] Checking repository {}", repo_path.display());
        }

        let repo_status = check_repo_status(&repo_path, &filters, args.debug);

        let mut results_guard = results.lock().unwrap();
        if args.all || repo_status.has_issues(&filters) {
            results_guard.push((repo_path.clone(), repo_status));
        }
    });

    // Display results
    let results_guard = results.lock().unwrap();
    for (repo_path, status) in results_guard.iter() {
        display_repo_status(repo_path, &status, &filters);
    }

    println!("Scan complete: {} repositories processed", entries.len());

    Ok(())
}

fn load_config(config_path: Option<&Path>) -> Result<Config, Box<dyn std::error::Error>> {
    // Determine config file path
    let config_path = if let Some(path) = config_path {
        path.to_path_buf()
    } else {
        // Try to find config in default locations
        let home_dir = dirs::home_dir().unwrap_or_default();
        let home_config = home_dir.join(".gitgud.toml");

        if home_config.exists() {
            home_config
        } else {
            // Also try XDG config directory
            match dirs::config_dir() {
                Some(config_dir) => {
                    let xdg_config = config_dir.join("gitgud").join("config.toml");
                    if xdg_config.exists() {
                        xdg_config
                    } else {
                        // Return default config if no file found
                        return Ok(Config::default());
                    }
                }
                None => return Ok(Config::default()),
            }
        }
    };

    // If the specified config file doesn't exist, return default config
    if !config_path.exists() {
        return Ok(Config::default());
    }

    // Read and parse config file
    let config_content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_content)?;

    Ok(config)
}

fn check_repo_status(repo_path: &Path, filters: &RepoFilters, debug: bool) -> RepoStatus {
    let mut status = RepoStatus {
        untracked_files: false,
        unstaged_changes: false,
        ahead_of_remote: false,
        missing_remote: false,
        non_default_branch: None,
    };

    // Only run the git status command if we need any of its information
    if filters.check_untracked || filters.check_unstaged || filters.check_ahead {
        if let Ok(output) = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .arg("status")
            .arg("--porcelain")
            .arg("-b") // Include branch info
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);

            if filters.check_untracked {
                status.untracked_files =
                    output_str.lines().any(|line| line.trim().starts_with("??"));
            }

            if filters.check_unstaged {
                status.unstaged_changes = output_str
                    .lines()
                    .any(|line| line.trim().starts_with("M") || line.trim().starts_with("D"));
            }

            if filters.check_ahead {
                status.ahead_of_remote = output_str
                    .lines()
                    .any(|line| line.trim().starts_with("##") && line.contains("[ahead "));
            }

            if debug {
                println!(
                    "[-] Status output for {}: {}",
                    repo_path.display(),
                    output_str
                );
            }
        }
    }

    // Check remotes if needed
    if filters.check_no_remotes {
        if let Ok(output) = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .arg("remote")
            .output()
        {
            let remotes = String::from_utf8_lossy(&output.stdout);
            status.missing_remote = remotes.trim().is_empty();

            if debug && status.missing_remote {
                println!("[-] No remotes found for {}", repo_path.display());
            }
        }
    }

    // Get current branch if needed
    if filters.check_branch {
        if let Ok(output) = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()
        {
            let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if !(current_branch == "master"
                || current_branch == "main"
                || current_branch == "develop")
            {
                status.non_default_branch = Some(current_branch.clone());

                if debug {
                    println!(
                        "[-] Non-default branch for {}: {}",
                        repo_path.display(),
                        current_branch
                    );
                }
            }
        }
    }

    status
}

fn display_repo_status(repo_path: &Path, status: &RepoStatus, filters: &RepoFilters) {
    let mut printed = false;

    if filters.check_untracked && status.untracked_files {
        println!(
            "{}",
            format!("[+] {} => untracked files found", repo_path.display())
                .green()
                .bold()
        );
        printed = true;
    }

    if filters.check_unstaged && status.unstaged_changes {
        println!(
            "{}",
            format!(
                "[~] {} => changes not staged for commit",
                repo_path.display()
            )
            .yellow()
            .bold()
        );
        printed = true;
    }

    if filters.check_ahead && status.ahead_of_remote {
        println!(
            "{}",
            format!("[!] {} => branch ahead of remote", repo_path.display())
                .red()
                .bold()
        );
        printed = true;
    }

    if filters.check_no_remotes && status.missing_remote {
        println!(
            "{}",
            format!("[!] {} => repo missing remote", repo_path.display())
                .red()
                .bold()
        );
        printed = true;
    }

    if filters.check_branch {
        if let Some(branch) = &status.non_default_branch {
            println!(
                "{}",
                format!(
                    "[!] {} => currently on a checked out branch: {}",
                    repo_path.display(),
                    branch
                )
                .cyan()
                .bold()
            );
            printed = true;
        }
    }

    if printed {
        println!("");
    }
}
