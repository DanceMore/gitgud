use colored::*;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

fn main() -> std::io::Result<()> {
    // Parse arguments
    let args: Vec<String> = env::args().collect();
    let debug = args.iter().any(|arg| arg == "--debug");

    let root = args.get(1).map(|s| s.as_str()).unwrap_or(".");
    let root = if root == "." {
        env::current_dir().unwrap()
    } else {
        PathBuf::from(root)
    };

    println!(
        "{}",
        format!("[?] target basedir is {}", root.display())
            .cyan()
            .bold()
    );

    // Get all entries in directory
    let entries: Vec<_> = fs::read_dir(&root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|entry| entry.path().join(".git").is_dir())
        .collect();

    if debug {
        println!("[-] found {} git repositories", entries.len());
    }

    // Process repositories in parallel
    let results = Arc::new(Mutex::new(Vec::new()));

    entries.par_iter().for_each(|entry| {
        let repo_path = entry.path();
        if debug {
            println!("[-] checking repository {}", repo_path.display());
        }

        let repo_status = check_repo_status(&repo_path, debug);

        if repo_status.has_issues() {
            let mut results_guard = results.lock().unwrap();
            results_guard.push((repo_path.clone(), repo_status));
        }
    });

    // Display results
    let results_guard = results.lock().unwrap();
    for (repo_path, status) in results_guard.iter() {
        display_repo_status(repo_path, &status);
    }

    Ok(())
}

struct RepoStatus {
    untracked_files: bool,
    unstaged_changes: bool,
    ahead_of_remote: bool,
    missing_remote: bool,
    non_default_branch: Option<String>,
}

impl RepoStatus {
    fn has_issues(&self) -> bool {
        self.untracked_files
            || self.unstaged_changes
            || self.ahead_of_remote
            || self.missing_remote
            || self.non_default_branch.is_some()
    }
}

fn check_repo_status(repo_path: &Path, debug: bool) -> RepoStatus {
    let mut status = RepoStatus {
        untracked_files: false,
        unstaged_changes: false,
        ahead_of_remote: false,
        missing_remote: false,
        non_default_branch: None,
    };

    // Run git status (single command instead of multiple)
    if let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("status")
        .arg("--porcelain")
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);

        status.untracked_files = output_str.lines().any(|line| line.trim().starts_with("??"));
        status.unstaged_changes = output_str
            .lines()
            .any(|line| line.trim().starts_with("M") || line.trim().starts_with("D"));
        status.ahead_of_remote = output_str
            .lines()
            .any(|line| line.trim().starts_with("##") && line.contains("[ahead "));
    }

    // Check remotes
    if let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("remote")
        .output()
    {
        let remotes = String::from_utf8_lossy(&output.stdout);
        status.missing_remote = remotes.trim().is_empty();
    }

    // Get current branch
    if let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
    {
        let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if !(current_branch == "master" || current_branch == "main" || current_branch == "develop")
        {
            status.non_default_branch = Some(current_branch);
        }
    }

    status
}

fn display_repo_status(repo_path: &Path, status: &RepoStatus) {
    let mut printed = false;

    if status.untracked_files {
        println!(
            "{}",
            format!("[+] {} => untracked files found", repo_path.display())
                .green()
                .bold()
        );
        printed = true;
    }

    if status.unstaged_changes {
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

    if status.ahead_of_remote {
        println!(
            "{}",
            format!("[!] {} => branch ahead of remote", repo_path.display())
                .red()
                .bold()
        );
        printed = true;
    }

    if status.missing_remote {
        println!(
            "{}",
            format!("[!] {} => repo missing remote", repo_path.display())
                .red()
                .bold()
        );
        printed = true;
    }

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

    if printed {
        println!("");
    }
}
