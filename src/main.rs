use colored::*;
use std::env;
use std::fs;
use std::io::BufRead;
use std::process::Command;

fn main() -> std::io::Result<()> {
    let root = env::args().nth(1).unwrap_or(".".into());
    let root = if root == "." {
        env::current_dir().unwrap()
    } else {
        root.into()
    };
    println!(
        "{}",
        format!("[?] target basedir is {}", root.display())
            .cyan()
            .bold()
    );
    env::set_current_dir(&root)?;

    let debug = env::args().any(|arg| arg == "--debug");

    for entry in fs::read_dir(&root)? {
        let mut printed = false;
        let entry = entry?;
        if debug {
            println!("[-] checking directory {}", &entry.path().display());
        }
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let repo_dir = entry.path().join(".git");
        if !repo_dir.is_dir() {
            if debug {
                println!("[---] not a repo, skipping...");
            }
            continue;
        }

        if debug {
            println!("[-+] directory is a git repo");
        }

        env::set_current_dir(&repo_dir.parent().unwrap())?;

        // advanced git status handling
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .output()?;
        let output_str = String::from_utf8_lossy(&output.stdout);

        let untracked_files_found = output_str.lines().any(|line| line.trim().starts_with("??"));
        let changes_not_staged_for_commit = output_str
            .lines()
            .any(|line| line.trim().starts_with("M") || line.trim().starts_with("D"));
        let branch_ahead_of_remote = output_str
            .lines()
            .any(|line| line.trim().starts_with("##") && line.contains("[ahead "));

        if untracked_files_found {
            println!(
                "{}",
                format!(
                    "[+] {} => untracked files found",
                    repo_dir.parent().unwrap().display()
                )
                .green()
                .bold()
            );
            printed = true;
        }

        if changes_not_staged_for_commit {
            println!(
                "{}",
                format!(
                    "[~] {} => changes not staged for commit",
                    repo_dir.parent().unwrap().display()
                )
                .yellow()
                .bold()
            );
            printed = true;
        }

        if branch_ahead_of_remote {
            println!(
                "{}",
                format!(
                    "[!] {} => branch ahead of remote",
                    repo_dir.parent().unwrap().display()
                )
                .red()
                .bold()
            );
            printed = true;
        }

        let output = Command::new("git").arg("remote").arg("-v").output()?;
        let remotes = output.stdout.lines().collect::<Vec<_>>();

        if remotes.len() < 2 {
            println!(
                "{}",
                format!(
                    "[!] {} => repo missing remote",
                    repo_dir.parent().unwrap().display()
                )
                .red()
                .bold()
            );
            printed = true;
        }

	// Get the current branch name
        let branch_output = Command::new("git").arg("rev-parse").arg("--abbrev-ref").arg("HEAD").output()?;
        let current_branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

        // acceptable default branches when not working:
        // master, main, develop
        if !(current_branch == "master" || current_branch == "main" || current_branch == "develop" ) {
            println!(
                "{}",
                format!(
                    "[!] {} => currently on a checked out branch: {}",
                    repo_dir.parent().unwrap().display(), current_branch
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

    Ok(())
}
