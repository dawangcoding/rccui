use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::tauri::commands;
use crate::tauri::types::{GitBranches, GitCommit, GitHubRepoInfo, GitRemoteStatus, GitStatus};

// ─── Git Sub-Tab ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitTab {
    #[default]
    Changes,
    History,
    Branches,
    GitHub,
}

// ─── GitContext ──────────────────────────────────────────────────────────────

/// State management for the Git feature.
#[derive(Clone, Copy)]
pub struct GitContext {
    /// Active sub-tab within the Git panel
    pub active_tab: RwSignal<GitTab>,
    /// Git status (branch, modified/added/deleted/untracked files)
    pub status: RwSignal<Option<GitStatus>>,
    /// Whether the status is currently loading
    pub status_loading: RwSignal<bool>,
    /// Remote status (ahead/behind/upstream info)
    pub remote_status: RwSignal<Option<GitRemoteStatus>>,
    /// Commit history list
    pub commits: RwSignal<Vec<GitCommit>>,
    /// Whether commits are loading
    pub commits_loading: RwSignal<bool>,
    /// Branch list
    pub branches: RwSignal<Option<GitBranches>>,
    /// Whether branches are loading
    pub branches_loading: RwSignal<bool>,
    /// Selected file for diff viewing (file path)
    pub selected_diff_file: RwSignal<Option<String>>,
    /// Diff content for the selected file
    pub diff_content: RwSignal<Option<String>>,
    /// Whether diff is loading
    pub diff_loading: RwSignal<bool>,
    /// Selected commit hash for viewing its diff
    pub selected_commit: RwSignal<Option<String>>,
    /// Commit diff content
    pub commit_diff_content: RwSignal<Option<String>>,
    /// Error message from last operation
    pub error_message: RwSignal<Option<String>>,
    /// Whether a git operation (commit/push/pull etc.) is in progress
    pub op_loading: RwSignal<bool>,
    /// GitHub repository info from `gh` CLI
    pub github_info: RwSignal<Option<GitHubRepoInfo>>,
    /// Whether GitHub info is loading
    pub github_loading: RwSignal<bool>,
    /// Whether the project has no GitHub remote (triggers create repo form)
    pub github_no_remote: RwSignal<bool>,
}

impl GitContext {
    pub fn new() -> Self {
        Self {
            active_tab: RwSignal::new(GitTab::Changes),
            status: RwSignal::new(None),
            status_loading: RwSignal::new(false),
            remote_status: RwSignal::new(None),
            commits: RwSignal::new(Vec::new()),
            commits_loading: RwSignal::new(false),
            branches: RwSignal::new(None),
            branches_loading: RwSignal::new(false),
            selected_diff_file: RwSignal::new(None),
            diff_content: RwSignal::new(None),
            diff_loading: RwSignal::new(false),
            selected_commit: RwSignal::new(None),
            commit_diff_content: RwSignal::new(None),
            error_message: RwSignal::new(None),
            op_loading: RwSignal::new(false),
            github_info: RwSignal::new(None),
            github_loading: RwSignal::new(false),
            github_no_remote: RwSignal::new(false),
        }
    }

    /// Load git status for a project.
    pub fn load_status(&self, project: String) {
        let ctx = *self;
        ctx.status_loading.set(true);
        ctx.error_message.set(None);
        let project2 = project.clone();
        spawn_local(async move {
            match commands::git_status(&project).await {
                Ok(s) => ctx.status.set(Some(s)),
                Err(e) => ctx.error_message.set(Some(e)),
            }
            ctx.status_loading.set(false);
        });
        // Also load remote status in parallel
        spawn_local(async move {
            if let Ok(rs) = commands::git_remote_status(&project2).await {
                ctx.remote_status.set(Some(rs));
            }
        });
    }

    /// Load commit history.
    pub fn load_commits(&self, project: String) {
        let ctx = *self;
        ctx.commits_loading.set(true);
        spawn_local(async move {
            match commands::git_commits(&project, Some(50)).await {
                Ok(resp) => ctx.commits.set(resp.commits),
                Err(e) => ctx.error_message.set(Some(e)),
            }
            ctx.commits_loading.set(false);
        });
    }

    /// Load branch list.
    pub fn load_branches(&self, project: String) {
        let ctx = *self;
        ctx.branches_loading.set(true);
        spawn_local(async move {
            match commands::git_branches(&project).await {
                Ok(b) => ctx.branches.set(Some(b)),
                Err(e) => ctx.error_message.set(Some(e)),
            }
            ctx.branches_loading.set(false);
        });
    }

    /// Load diff for a specific file.
    pub fn load_file_diff(&self, project: String, file: String) {
        let ctx = *self;
        ctx.selected_diff_file.set(Some(file.clone()));
        ctx.diff_content.set(None);
        ctx.diff_loading.set(true);
        spawn_local(async move {
            match commands::git_diff(&project, &file).await {
                Ok(resp) => ctx.diff_content.set(Some(resp.diff)),
                Err(e) => ctx.error_message.set(Some(e)),
            }
            ctx.diff_loading.set(false);
        });
    }

    /// Load diff for a specific commit.
    pub fn load_commit_diff(&self, project: String, commit: String) {
        let ctx = *self;
        ctx.selected_commit.set(Some(commit.clone()));
        ctx.commit_diff_content.set(None);
        ctx.diff_loading.set(true);
        spawn_local(async move {
            match commands::git_commit_diff(&project, &commit).await {
                Ok(resp) => ctx.commit_diff_content.set(Some(resp.diff)),
                Err(e) => ctx.error_message.set(Some(e)),
            }
            ctx.diff_loading.set(false);
        });
    }

    /// Clear selected diff state.
    pub fn clear_diff(&self) {
        self.selected_diff_file.set(None);
        self.diff_content.set(None);
        self.selected_commit.set(None);
        self.commit_diff_content.set(None);
    }

    /// Load GitHub repository info via `gh` CLI.
    pub fn load_github_info(&self, project: String) {
        let ctx = *self;
        ctx.github_loading.set(true);
        ctx.error_message.set(None);
        ctx.github_no_remote.set(false);
        spawn_local(async move {
            match commands::gh_repo_view(&project).await {
                Ok(info) => ctx.github_info.set(Some(info)),
                Err(e) => {
                    if e.contains("NO_GITHUB_REMOTE") {
                        ctx.github_no_remote.set(true);
                    } else {
                        ctx.error_message.set(Some(e));
                    }
                }
            }
            ctx.github_loading.set(false);
        });
    }

    /// Reset all state (called when switching projects).
    pub fn clear(&self) {
        self.active_tab.set(GitTab::Changes);
        self.status.set(None);
        self.status_loading.set(false);
        self.remote_status.set(None);
        self.commits.set(Vec::new());
        self.commits_loading.set(false);
        self.branches.set(None);
        self.branches_loading.set(false);
        self.selected_diff_file.set(None);
        self.diff_content.set(None);
        self.diff_loading.set(false);
        self.selected_commit.set(None);
        self.commit_diff_content.set(None);
        self.error_message.set(None);
        self.op_loading.set(false);
        self.github_info.set(None);
        self.github_loading.set(false);
        self.github_no_remote.set(false);
    }
}
