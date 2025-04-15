use std::path::Path;
use std::process::Command;

pub fn get_remote_url(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("config")
        .arg("--get")
        .arg("remote.origin.url")
        .output()
        .ok()?;

    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !url.is_empty() {
            return Some(url);
        }
    }

    None
}

pub fn list_remote_branches(repo_path: &Path) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("branch")
        .arg("-r")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .map(|line| {
                    // Strip "origin/" prefix if present
                    if let Some(branch) = line.strip_prefix("origin/") {
                        branch.to_string()
                    } else {
                        line
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}
