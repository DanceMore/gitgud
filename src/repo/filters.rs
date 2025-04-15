#[derive(Debug, Clone)]
pub struct RepoFilters {
    pub check_untracked: bool,
    pub check_unstaged: bool,
    pub check_ahead: bool,
    pub check_no_remotes: bool,
    pub check_branch: bool,
    pub check_prs: bool,
    pub include_draft_prs: bool,
}
