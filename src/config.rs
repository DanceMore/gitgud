use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubConfig {
    /// GitHub token for API access
    pub token: Option<String>,

    /// List of static branches to always keep
    #[serde(default)]
    pub protected_branches: Vec<String>,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: None,
            protected_branches: Vec::new(),
        }
    }
}

/// Config file structure that can be loaded from TOML
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_threads")]
    pub threads: Option<usize>,

    #[serde(default = "default_true")]
    pub check_untracked: bool,

    #[serde(default = "default_true")]
    pub check_unstaged: bool,

    #[serde(default = "default_true")]
    pub check_ahead: bool,

    #[serde(default = "default_true")]
    pub check_no_remotes: bool,

    #[serde(default = "default_true")]
    pub check_branch: bool,

    #[serde(default = "default_false")]
    pub check_prs: bool,

    #[serde(default = "default_true")]
    pub include_draft_prs: bool,

    #[serde(default)]
    pub github: GitHubConfig,
}

fn default_threads() -> Option<usize> {
    None
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threads: None,
            check_untracked: true,
            check_unstaged: true,
            check_ahead: true,
            check_no_remotes: true,
            check_branch: true,
            check_prs: false,
            include_draft_prs: true,
            github: GitHubConfig::default(),
        }
    }
}

pub fn load_config(config_path: Option<&Path>) -> Result<Config, Box<dyn std::error::Error>> {
    // Determine config file path
    let config_path = if let Some(path) = config_path {
        path.to_path_buf()
    } else {
        // Try to find config in default locations
        let home_dir = dirs::home_dir().unwrap_or_default();
        let home_config = home_dir.join(".gitgud.toml");

        if home_config.exists() {
            home_config
        } else {
            // Also try XDG config directory
            match dirs::config_dir() {
                Some(config_dir) => {
                    let xdg_config = config_dir.join("gitgud").join("config.toml");
                    if xdg_config.exists() {
                        xdg_config
                    } else {
                        // Return default config if no file found
                        return Ok(Config::default());
                    }
                }
                None => return Ok(Config::default()),
            }
        }
    };

    // If the specified config file doesn't exist, return default config
    if !config_path.exists() {
        return Ok(Config::default());
    }

    // Read and parse config file
    let config_content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_content)?;

    Ok(config)
}

// Function to load protected branches from file
pub fn load_protected_branches(path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let branches: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    Ok(branches)
}
