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
        let mut dirty = false;
        let output = Command::new("git").arg("status").output()?;

        for line in output.stdout.lines() {
            if let Ok(line) = line {
                if line.contains("Your branch is ahead") || line.contains("Changes not staged for commit") {
                    dirty = true;
                    break;
                }
            }
        }

        if dirty {
            println!("[!!!] dirty repo found => {}", repo_dir.parent().unwrap().display());
        }

        let output = Command::new("git").arg("remote").arg("-v").output()?;
        let remotes = output.stdout.lines().collect::<Vec<_>>();

        if remotes.len() < 2 {
            println!("[!!!] repo missing remote => {}", repo_dir.parent().unwrap().display());
        }
    }

    Ok(())
}
