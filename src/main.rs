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
    println!("[+] target basedir is {}", root.display());
    env::set_current_dir(&root)?;

    let debug = env::args().any(|arg| arg == "--debug");

    for entry in fs::read_dir(&root)? {
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
        let output = Command::new("git").arg("status").arg("--porcelain").output()?;
        let output_str = String::from_utf8_lossy(&output.stdout);

        let untracked_files_found = output_str.lines().any(|line| line.trim().starts_with("??"));
        let changes_not_staged_for_commit = output_str.lines().any(|line| line.trim().starts_with("M") || line.trim().starts_with("D"));
        let branch_ahead_of_remote = output_str.lines().any(|line| line.trim().starts_with("##") && line.contains("[ahead "));

        if untracked_files_found {
            println!("[!!!] untracked files found => {}", repo_dir.parent().unwrap().display());
        }

        if changes_not_staged_for_commit {
            println!("[!!!] changes not staged for commit => {}", repo_dir.parent().unwrap().display());
        }

        if branch_ahead_of_remote {
            println!("[!!!] branch ahead of remote => {}", repo_dir.parent().unwrap().display());
        }

        let output = Command::new("git").arg("remote").arg("-v").output()?;
        let remotes = output.stdout.lines().collect::<Vec<_>>();

        if remotes.len() < 2 {
            println!("[!!!] repo missing remote => {}", repo_dir.parent().unwrap().display());
        }
    }

    Ok(())
}
