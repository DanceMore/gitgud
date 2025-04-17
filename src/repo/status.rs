use crate::git::status::GitStatus;
use crate::github::pr::PullRequestInfo;
use crate::repo::filters::RepoFilters;
use std::collections::HashSet;

pub struct RepoStatus {
    // Git status
    pub untracked_files: bool,
    pub unstaged_changes: bool,
    pub ahead_of_remote: bool,
    pub missing_remote: bool,
    pub non_default_branch: Option<String>,

    // GitHub PR information
    pub open_prs: Vec<PullRequestInfo>,
}

impl RepoStatus {
    pub fn new(
        git_status: GitStatus,
        prs: Vec<PullRequestInfo>,
        protected_branches: HashSet<String>,
    ) -> Self {
        // Determine if we should warn about non-default branch
        let non_default_branch = if !git_status.is_default_branch
            && !protected_branches.contains(&git_status.current_branch)
        {
            Some(git_status.current_branch.clone())
        } else {
            None
        };

        Self {
            untracked_files: git_status.untracked_files,
            unstaged_changes: git_status.unstaged_changes,
            ahead_of_remote: git_status.ahead_of_remote,
            missing_remote: git_status.missing_remote,
            non_default_branch,
            open_prs: prs,
        }
    }

    pub fn has_issues(&self, filters: &RepoFilters) -> bool {
        (filters.check_untracked && self.untracked_files)
            || (filters.check_unstaged && self.unstaged_changes)
            || (filters.check_ahead && self.ahead_of_remote)
            || (filters.check_no_remotes && self.missing_remote)
            || (filters.check_branch && self.non_default_branch.is_some())
            || (filters.check_prs && !self.open_prs.is_empty())
    }
}
