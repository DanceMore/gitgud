#[derive(Debug, Clone)]
pub struct PullRequestInfo {
    pub number: u64,
    pub title: String,
    pub branch: String,
    pub is_draft: bool,
}
