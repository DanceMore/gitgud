use crate::git::remote::get_remote_url;
use crate::github::pr::PullRequestInfo;
use octocrab::Octocrab;
use std::path::Path;

/// Fetch GitHub PR information for a repository
pub async fn fetch_github_prs(
    repo_path: &Path, 
    github_token: Option<&str>,
    debug: bool
) -> Result<Vec<PullRequestInfo>, Box<dyn std::error::Error>> {
    // Get remote URL
    let remote_url = match get_remote_url(repo_path) {
        Some(url) => url,
        None => return Ok(Vec::new()), // No remote URL found
    };
    
    // Parse the GitHub repo URL to extract owner and repo
    let (owner, repo) = parse_github_url(&remote_url)?;
    
    if debug {
        println!("[-] Fetching PRs for {}/{}", owner, repo);
    }
    
    // Create GitHub client with token if available
    let octocrab = match github_token {
        Some(token) => Octocrab::builder().personal_token(token.to_string()).build()?,
        None => Octocrab::builder().build()?,
    };
    
    // Fetch open pull requests
    let pulls = octocrab
        .pulls(owner, repo)
        .list()
        .state(octocrab::params::State::Open)
        .send()
        .await?;
    
    let mut pr_info = Vec::new();
    for pull in pulls.items {
        pr_info.push(PullRequestInfo {
            number: pull.number,
            title: pull.title.expect("no Pull Request Title found ??"),
            branch: pull.head.ref_field,
            is_draft: pull.draft.unwrap_or(false),
        });
    }
    
    Ok(pr_info)
}

/// Helper function to parse GitHub URL
pub fn parse_github_url(url: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Handle various GitHub URL formats
    // Examples:
    // - https://github.com/owner/repo.git
    // - git@github.com:owner/repo.git
    // - git://github.com/owner/repo.git
    
    let url = url.trim();
    
    // HTTPS format
    if let Some(path) = url.strip_prefix("https://github.com/") {
        let path = path.strip_suffix(".git").unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }
    
    // SSH format
    if let Some(path) = url.strip_prefix("git@github.com:") {
        let path = path.strip_suffix(".git").unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }
    
    // Git protocol format
    if let Some(path) = url.strip_prefix("git://github.com/") {
        let path = path.strip_suffix(".git").unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }
    
    Err("Unable to parse GitHub URL".into())
}
