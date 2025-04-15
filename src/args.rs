use clap::Parser;
use std::path::PathBuf;

/// Opinionated Git repository scanner to keep you Organized and On Task
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Directory containing Git repositories to scan
    #[arg(default_value = ".")]
    pub directory: PathBuf,

    /// Enable verbose debug output
    #[arg(short, long)]
    pub debug: bool,

    /// Number of threads to use (default: auto)
    #[arg(short, long)]
    pub threads: Option<usize>,

    /// Show all repositories, even those with no issues
    #[arg(short, long)]
    pub all: bool,

    /// Path to config file (default: ~/.gitgud.toml)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Check for untracked files
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    pub check_untracked: bool,

    /// Check for unstaged changes
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    pub check_unstaged: bool,

    /// Check if branch is ahead of remote
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    pub check_ahead: bool,

    /// Check if repository has no remotes
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    pub check_no_remotes: bool,

    /// Check if branch is not a default branch (main, master, develop)
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    pub check_branch: bool,

    /// Check for open GitHub pull requests
    #[arg(long, action = clap::ArgAction::Set, default_value_t = false)]
    pub check_prs: bool,

    /// Show draft PRs (requires --check-prs)
    #[arg(long, action = clap::ArgAction::Set, default_value_t = true)]
    pub include_draft_prs: bool,

    /// GitHub token (or set GITHUB_TOKEN env var)
    #[arg(long, env = "GITHUB_TOKEN")]
    pub github_token: Option<String>,

    /// Path to a file containing list of protected branches
    #[arg(long)]
    pub protected_branches_file: Option<PathBuf>,
}
