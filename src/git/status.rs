use crate::repo::filters::RepoFilters;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub untracked_files: bool,
    pub unstaged_changes: bool,
    pub ahead_of_remote: bool,
    pub missing_remote: bool,
    pub current_branch: String,
    pub is_default_branch: bool,
}

impl GitStatus {
    pub fn new() -> Self {
        Self {
            untracked_files: false,
            unstaged_changes: false,
            ahead_of_remote: false,
            missing_remote: false,
            current_branch: String::new(),
            is_default_branch: true,
        }
    }
}

pub fn check_git_status(repo_path: &Path, filters: &RepoFilters, debug: bool) -> GitStatus {
    let mut status = GitStatus::new();

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
            status.current_branch = current_branch.clone();

            status.is_default_branch = current_branch == "master"
                || current_branch == "main";

            if debug && !status.is_default_branch {
                println!(
                    "[-] Non-default branch for {}: {}",
                    repo_path.display(),
                    current_branch
                );
            }
        }
    }

    status
}
