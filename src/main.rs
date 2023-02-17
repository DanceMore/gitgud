use std::env;
use std::fs;
use std::io::BufRead;
use std::process::Command;

fn main() -> std::io::Result<()> {
    let root = env::args().nth(1).unwrap_or(".".into());
    println!("[+] target basedir is {}", &root);

    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let repo_dir = entry.path().join(".git");
        if !repo_dir.is_dir() {
            continue;
        }

        env::set_current_dir(&repo_dir.parent().unwrap())?;
        println!("[-] checking directory {}", &repo_dir.display());

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
    }

    Ok(())
}
