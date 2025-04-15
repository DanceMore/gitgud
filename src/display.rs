use colored::*;
use std::path::Path;
use crate::repo::filters::RepoFilters;
use crate::repo::status::RepoStatus;

pub fn display_repos_status(
    results: &[(std::path::PathBuf, RepoStatus)], 
    filters: &RepoFilters
) {
    for (repo_path, status) in results {
        display_repo_status(repo_path, status, filters);
    }
}

pub fn display_repo_status(repo_path: &Path, status: &RepoStatus, filters: &RepoFilters) {
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
    
    // Display PR information
    if filters.check_prs && !status.open_prs.is_empty() {
        println!(
            "{}",
            format!("[PR] {} => {} open pull requests:", repo_path.display(), status.open_prs.len())
                .blue()
                .bold()
        );
        
        for pr in &status.open_prs {
            let draft_marker = if pr.is_draft { "[DRAFT] " } else { "" };
            println!(
                "     #{} {} - Branch: {}",
                pr.number,
                format!("{}{}", draft_marker, pr.title).blue(),
                pr.branch.magenta()
            );
        }
        
        printed = true;
    }

    if printed {
        println!("");
    }
}
