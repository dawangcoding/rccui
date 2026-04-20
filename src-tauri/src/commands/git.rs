use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::{Value, json};
use tokio::process::Command;

use crate::error::AppError;
use crate::services::project_scanner;
use crate::state::AppState;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Resolve a project name to an absolute path, with path traversal protection.
async fn resolve_project_path(project: &str) -> Result<PathBuf, AppError> {
    let path = if project.starts_with('/') {
        PathBuf::from(project)
    } else {
        let actual = project_scanner::extract_project_directory(project).await;
        PathBuf::from(actual)
    };

    if path == PathBuf::from("/") {
        return Err(AppError::BadRequest("Cannot operate on root directory".into()));
    }

    if !path.exists() {
        return Err(AppError::NotFound(format!(
            "Project path not found: {}",
            path.display()
        )));
    }

    Ok(path)
}

/// Validate a branch name (no shell metacharacters).
fn validate_ref(name: &str) -> Result<(), AppError> {
    if name.is_empty() || name.contains('\0') || name.contains("..") {
        return Err(AppError::BadRequest("Invalid ref name".into()));
    }
    if name.contains(';') || name.contains('|') || name.contains('&') || name.contains('`') {
        return Err(AppError::BadRequest(
            "Invalid characters in ref name".into(),
        ));
    }
    Ok(())
}

/// Validate a file path (no null bytes, no traversal).
fn validate_file_path(file: &str, project_path: &PathBuf) -> Result<PathBuf, AppError> {
    if file.contains('\0') {
        return Err(AppError::BadRequest("Invalid file path".into()));
    }
    let full = project_path.join(file);
    let canonical_project = project_path.canonicalize().unwrap_or(project_path.clone());
    let canonical_file = full.canonicalize().unwrap_or(full.clone());

    if !canonical_file.starts_with(&canonical_project) {
        return Err(AppError::BadRequest("Path traversal detected".into()));
    }
    Ok(full)
}

/// Run a git command and return its output.
async fn run_git(project_path: &PathBuf, args: &[&str]) -> Result<String, AppError> {
    let cmd_str = format!("git {}", args.join(" "));
    tracing::debug!(cmd = %cmd_str, path = %project_path.display(), "executing git command");

    let output = Command::new("git")
        .args(args)
        .current_dir(project_path)
        .output()
        .await
        .map_err(|e| {
            tracing::error!(cmd = %cmd_str, error = %e, "failed to spawn git process");
            AppError::Internal(e.into())
        })?;

    if output.status.success() {
        tracing::debug!(cmd = %cmd_str, "git command succeeded");
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        tracing::warn!(cmd = %cmd_str, stderr = %stderr.trim(), "git command failed");
        Err(AppError::Internal(anyhow::anyhow!("git error: {stderr}")))
    }
}

/// Run a git command, returning stdout even on non-zero exit (for diff, etc.).
async fn run_git_lossy(project_path: &PathBuf, args: &[&str]) -> Result<String, AppError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(project_path)
        .output()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ─── Request Types ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitRequest {
    pub project: String,
    pub message: String,
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialCommitRequest {
    pub project: String,
    #[serde(default = "default_initial_commit_message")]
    pub message: String,
    #[serde(default)]
    pub files: Vec<String>,
}

fn default_initial_commit_message() -> String {
    "Initial commit".to_string()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchRequest {
    pub project: String,
    pub branch: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscardRequest {
    pub project: String,
    pub file: String,
}

// ─── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn git_status(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
) -> Result<Value, String> {
    _git_status(&project).await.map_err(|e| e.to_string())
}

async fn _git_status(project: &str) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;

    let branch = run_git(&path, &["rev-parse", "--abbrev-ref", "HEAD"])
        .await
        .unwrap_or_else(|_| "main".to_string())
        .trim()
        .to_string();

    let has_commits = run_git(&path, &["rev-parse", "HEAD"]).await.is_ok();
    let status_output = run_git(&path, &["status", "--porcelain=v1"])
        .await
        .unwrap_or_default();

    let mut modified = Vec::new();
    let mut added = Vec::new();
    let mut deleted = Vec::new();
    let mut untracked = Vec::new();

    for line in status_output.lines() {
        if line.len() < 4 {
            continue;
        }
        let xy = &line[..2];
        let file = line[3..].trim().to_string();

        match xy.trim() {
            "M" | "MM" | "AM" => modified.push(file),
            "A" => added.push(file),
            "D" => deleted.push(file),
            "??" => untracked.push(file),
            "R" | "RM" => modified.push(file),
            "C" => added.push(file),
            _ => {
                if xy.contains('M') {
                    modified.push(file);
                } else if xy.contains('D') {
                    deleted.push(file);
                } else if xy.contains('A') {
                    added.push(file);
                }
            }
        }
    }

    Ok(json!({
        "branch": branch,
        "hasCommits": has_commits,
        "modified": modified,
        "added": added,
        "deleted": deleted,
        "untracked": untracked
    }))
}

#[tauri::command]
pub async fn git_diff(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
    file: String,
) -> Result<Value, String> {
    _git_diff(&project, &file).await.map_err(|e| e.to_string())
}

async fn _git_diff(project: &str, file: &str) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;
    validate_file_path(file, &path)?;

    // Check if file is untracked
    let is_untracked = run_git(&path, &["ls-files", "--error-unmatch", file])
        .await
        .is_err();

    let mut output = if is_untracked {
        // For untracked files, git diff produces nothing.
        // Use --no-index to show the full file content as additions.
        run_git_lossy(&path, &["diff", "--no-index", "/dev/null", file]).await?
    } else {
        let mut out = run_git_lossy(&path, &["diff", "--", file]).await?;
        let staged = run_git_lossy(&path, &["diff", "--cached", "--", file]).await?;
        if !staged.is_empty() {
            out.push_str(&staged);
        }
        out
    };

    let max_len = 500_000;
    let is_truncated = output.len() > max_len;
    if is_truncated {
        output.truncate(max_len);
    }

    Ok(json!({
        "diff": output,
        "isTruncated": is_truncated
    }))
}

#[tauri::command]
pub async fn git_file_with_diff(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
    file: String,
) -> Result<Value, String> {
    _git_file_with_diff(&project, &file)
        .await
        .map_err(|e| e.to_string())
}

async fn _git_file_with_diff(project: &str, file: &str) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;
    let file_path = validate_file_path(file, &path)?;

    let is_untracked = run_git(&path, &["ls-files", "--error-unmatch", file])
        .await
        .is_err();

    let current_content = tokio::fs::read_to_string(&file_path)
        .await
        .unwrap_or_default();

    let old_content = if is_untracked {
        String::new()
    } else {
        run_git_lossy(&path, &["show", &format!("HEAD:{}", file)])
            .await
            .unwrap_or_default()
    };

    let is_deleted = !file_path.exists();

    Ok(json!({
        "currentContent": current_content,
        "oldContent": old_content,
        "isDeleted": is_deleted,
        "isUntracked": is_untracked
    }))
}

#[tauri::command]
pub async fn git_initial_commit(
    _state: tauri::State<'_, Arc<AppState>>,
    body: InitialCommitRequest,
) -> Result<Value, String> {
    _git_initial_commit(body).await.map_err(|e| e.to_string())
}

async fn _git_initial_commit(body: InitialCommitRequest) -> Result<Value, AppError> {
    let path = resolve_project_path(&body.project).await?;

    if body.files.is_empty() {
        run_git(&path, &["add", "-A"]).await?;
    } else {
        for file in &body.files {
            validate_file_path(file, &path)?;
            run_git(&path, &["add", file]).await?;
        }
    }

    let message = if body.message.trim().is_empty() {
        "Initial commit".to_string()
    } else {
        body.message
    };

    let output = run_git(&path, &["commit", "-m", &message]).await?;
    Ok(json!({ "success": true, "output": output }))
}

#[tauri::command]
pub async fn git_commit(
    _state: tauri::State<'_, Arc<AppState>>,
    body: CommitRequest,
) -> Result<Value, String> {
    _git_commit(body).await.map_err(|e| e.to_string())
}

async fn _git_commit(body: CommitRequest) -> Result<Value, AppError> {
    let path = resolve_project_path(&body.project).await?;

    if body.files.is_empty() {
        run_git(&path, &["add", "-A"]).await?;
    } else {
        for file in &body.files {
            validate_file_path(file, &path)?;
            run_git(&path, &["add", file]).await?;
        }
    }

    let output = run_git(&path, &["commit", "-m", &body.message]).await?;
    Ok(json!({ "success": true, "output": output }))
}

#[tauri::command]
pub async fn git_revert_local_commit(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
) -> Result<Value, String> {
    _git_revert_local_commit(&project)
        .await
        .map_err(|e| e.to_string())
}

async fn _git_revert_local_commit(project: &str) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;
    let output = run_git(&path, &["reset", "--soft", "HEAD~1"]).await?;
    Ok(json!({ "success": true, "output": output }))
}

#[tauri::command]
pub async fn git_branches(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
) -> Result<Value, String> {
    _git_branches(&project).await.map_err(|e| e.to_string())
}

async fn _git_branches(project: &str) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;

    let local = run_git(&path, &["branch", "--format=%(refname:short)"])
        .await
        .unwrap_or_default();
    let remote = run_git(&path, &["branch", "-r", "--format=%(refname:short)"])
        .await
        .unwrap_or_default();

    let local_branches: Vec<&str> = local.lines().filter(|l| !l.is_empty()).collect();
    let remote_branches: Vec<&str> = remote.lines().filter(|l| !l.is_empty()).collect();

    let mut all: Vec<&str> = local_branches.clone();
    all.extend(remote_branches.iter());

    Ok(json!({
        "branches": all,
        "localBranches": local_branches,
        "remoteBranches": remote_branches
    }))
}

#[tauri::command]
pub async fn git_checkout(
    _state: tauri::State<'_, Arc<AppState>>,
    body: BranchRequest,
) -> Result<Value, String> {
    _git_checkout(body).await.map_err(|e| e.to_string())
}

async fn _git_checkout(body: BranchRequest) -> Result<Value, AppError> {
    let path = resolve_project_path(&body.project).await?;
    validate_ref(&body.branch)?;
    let output = run_git(&path, &["checkout", &body.branch]).await?;
    Ok(json!({ "success": true, "output": output }))
}

#[tauri::command]
pub async fn git_create_branch(
    _state: tauri::State<'_, Arc<AppState>>,
    body: BranchRequest,
) -> Result<Value, String> {
    _git_create_branch(body).await.map_err(|e| e.to_string())
}

async fn _git_create_branch(body: BranchRequest) -> Result<Value, AppError> {
    let path = resolve_project_path(&body.project).await?;
    validate_ref(&body.branch)?;
    let output = run_git(&path, &["checkout", "-b", &body.branch]).await?;
    Ok(json!({ "success": true, "output": output }))
}

#[tauri::command]
pub async fn git_delete_branch(
    _state: tauri::State<'_, Arc<AppState>>,
    body: BranchRequest,
) -> Result<Value, String> {
    _git_delete_branch(body).await.map_err(|e| e.to_string())
}

async fn _git_delete_branch(body: BranchRequest) -> Result<Value, AppError> {
    let path = resolve_project_path(&body.project).await?;
    validate_ref(&body.branch)?;
    let output = run_git(&path, &["branch", "-d", &body.branch]).await?;
    Ok(json!({ "success": true, "output": output }))
}

#[tauri::command]
pub async fn git_commits(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
    limit: Option<u32>,
) -> Result<Value, String> {
    _git_commits(&project, limit)
        .await
        .map_err(|e| e.to_string())
}

async fn _git_commits(project: &str, limit: Option<u32>) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;
    let limit = limit.unwrap_or(50).min(100);
    let limit_str = format!("-{limit}");

    let output = run_git(
        &path,
        &[
            "log",
            &limit_str,
            "--format=%H|%an|%ae|%aI|%s",
            "--shortstat",
        ],
    )
    .await
    .unwrap_or_default();

    let mut result = Vec::new();
    let mut current_commit: Option<Value> = None;

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() == 5 {
            if let Some(c) = current_commit.take() {
                result.push(c);
            }
            current_commit = Some(json!({
                "hash": parts[0],
                "author": parts[1],
                "email": parts[2],
                "date": parts[3],
                "message": parts[4],
                "stats": null
            }));
        } else if let Some(ref mut c) = current_commit {
            c["stats"] = Value::String(line.trim().to_string());
        }
    }
    if let Some(c) = current_commit {
        result.push(c);
    }

    Ok(json!({ "commits": result }))
}

#[tauri::command]
pub async fn git_commit_diff(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
    commit: String,
) -> Result<Value, String> {
    _git_commit_diff(&project, &commit)
        .await
        .map_err(|e| e.to_string())
}

async fn _git_commit_diff(project: &str, commit: &str) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;
    validate_ref(commit)?;

    let mut output = run_git_lossy(&path, &["show", commit]).await?;

    let max_len = 500_000;
    let is_truncated = output.len() > max_len;
    if is_truncated {
        output.truncate(max_len);
    }

    Ok(json!({
        "diff": output,
        "isTruncated": is_truncated
    }))
}

#[tauri::command]
pub async fn git_remote_status(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
) -> Result<Value, String> {
    _git_remote_status(&project)
        .await
        .map_err(|e| e.to_string())
}

async fn _git_remote_status(project: &str) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;

    let branch = run_git(&path, &["rev-parse", "--abbrev-ref", "HEAD"])
        .await
        .unwrap_or_else(|_| "main".to_string())
        .trim()
        .to_string();

    let has_remote = run_git(&path, &["remote"])
        .await
        .map(|o| !o.trim().is_empty())
        .unwrap_or(false);

    let upstream_result = run_git(
        &path,
        &[
            "rev-parse",
            "--abbrev-ref",
            &format!("{branch}@{{upstream}}"),
        ],
    )
    .await;
    let has_upstream = upstream_result.is_ok();

    let (ahead, behind) = if has_upstream {
        let count = run_git(
            &path,
            &[
                "rev-list",
                "--left-right",
                "--count",
                &format!("{branch}...{branch}@{{upstream}}"),
            ],
        )
        .await
        .unwrap_or_default();

        let parts: Vec<&str> = count.trim().split('\t').collect();
        let a: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let b: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        (a, b)
    } else {
        (0, 0)
    };

    Ok(json!({
        "hasRemote": has_remote,
        "hasUpstream": has_upstream,
        "branch": branch,
        "ahead": ahead,
        "behind": behind,
        "isUpToDate": ahead == 0 && behind == 0
    }))
}

#[tauri::command]
pub async fn git_fetch(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
) -> Result<Value, String> {
    _git_fetch(&project).await.map_err(|e| e.to_string())
}

async fn _git_fetch(project: &str) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;
    tracing::info!(project = %project, "fetching from remote");
    let output = run_git(&path, &["fetch", "--all"]).await?;
    tracing::info!(project = %project, "fetch completed");
    Ok(json!({ "success": true, "output": output }))
}

#[tauri::command]
pub async fn git_pull(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
) -> Result<Value, String> {
    _git_pull(&project).await.map_err(|e| e.to_string())
}

async fn _git_pull(project: &str) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;
    tracing::info!(project = %project, "pulling from remote");
    let output = run_git(&path, &["pull"]).await?;
    tracing::info!(project = %project, "pull completed");
    Ok(json!({ "success": true, "output": output }))
}

#[tauri::command]
pub async fn git_push(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
) -> Result<Value, String> {
    _git_push(&project).await.map_err(|e| e.to_string())
}

async fn _git_push(project: &str) -> Result<Value, AppError> {
    let path = resolve_project_path(project).await?;

    // Check if current branch has an upstream; if not, push with --set-upstream
    let has_upstream = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .current_dir(&path)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    tracing::info!(project = %project, has_upstream = %has_upstream, "pushing to remote");
    let output = if has_upstream {
        run_git(&path, &["push"]).await?
    } else {
        // Get current branch name
        let branch = run_git(&path, &["rev-parse", "--abbrev-ref", "HEAD"]).await?;
        let branch = branch.trim();
        run_git(&path, &["push", "-u", "origin", branch]).await?
    };
    tracing::info!(project = %project, "push completed");

    Ok(json!({ "success": true, "output": output }))
}

#[tauri::command]
pub async fn git_publish(
    _state: tauri::State<'_, Arc<AppState>>,
    body: BranchRequest,
) -> Result<Value, String> {
    _git_publish(body).await.map_err(|e| e.to_string())
}

async fn _git_publish(body: BranchRequest) -> Result<Value, AppError> {
    let path = resolve_project_path(&body.project).await?;
    validate_ref(&body.branch)?;
    tracing::info!(project = %body.project, branch = %body.branch, "publishing branch to remote");
    let output = run_git(&path, &["push", "-u", "origin", &body.branch]).await?;
    tracing::info!(project = %body.project, branch = %body.branch, "branch published");
    Ok(json!({ "success": true, "output": output }))
}

#[tauri::command]
pub async fn git_discard(
    _state: tauri::State<'_, Arc<AppState>>,
    body: DiscardRequest,
) -> Result<Value, String> {
    _git_discard(body).await.map_err(|e| e.to_string())
}

async fn _git_discard(body: DiscardRequest) -> Result<Value, AppError> {
    let path = resolve_project_path(&body.project).await?;
    validate_file_path(&body.file, &path)?;

    let _ = run_git(&path, &["reset", "HEAD", "--", &body.file]).await;
    run_git(&path, &["checkout", "--", &body.file]).await?;

    Ok(json!({ "success": true, "message": "Changes discarded" }))
}

#[tauri::command]
pub async fn git_delete_untracked(
    _state: tauri::State<'_, Arc<AppState>>,
    body: DiscardRequest,
) -> Result<Value, String> {
    _git_delete_untracked(body)
        .await
        .map_err(|e| e.to_string())
}

async fn _git_delete_untracked(body: DiscardRequest) -> Result<Value, AppError> {
    let path = resolve_project_path(&body.project).await?;
    let file_path = validate_file_path(&body.file, &path)?;

    if file_path.is_dir() {
        tokio::fs::remove_dir_all(&file_path)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    } else {
        tokio::fs::remove_file(&file_path)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    }

    Ok(json!({ "success": true, "message": "File deleted" }))
}

// ─── GitHub CLI (gh) ─────────────────────────────────────────────────────────

/// Run gh CLI command in project directory.
async fn run_gh(path: &std::path::Path, args: &[&str]) -> Result<String, AppError> {
    let cmd_str = format!("gh {}", args.join(" "));
    tracing::debug!(cmd = %cmd_str, path = %path.display(), "executing gh command");

    let output = Command::new("gh")
        .args(args)
        .current_dir(path)
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                tracing::error!("GitHub CLI (gh) not found in PATH");
                AppError::BadRequest("GitHub CLI (gh) is not installed".into())
            } else {
                tracing::error!(cmd = %cmd_str, error = %e, "failed to spawn gh process");
                AppError::Internal(e.into())
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stderr_lower = stderr.to_lowercase();
        tracing::warn!(cmd = %cmd_str, stderr = %stderr.trim(), "gh command failed");
        if stderr_lower.contains("not logged") || stderr_lower.contains("auth") {
            return Err(AppError::BadRequest(
                "GitHub CLI not authenticated. Run 'gh auth login' in terminal".into(),
            ));
        }
        if stderr_lower.contains("not a github")
            || stderr_lower.contains("no github")
            || stderr_lower.contains("none of the git remotes")
            || stderr_lower.contains("no git remotes")
        {
            return Err(AppError::BadRequest(
                "NO_GITHUB_REMOTE".into(),
            ));
        }
        return Err(AppError::Internal(anyhow::anyhow!(
            "gh command failed: {}",
            stderr.trim()
        )));
    }

    tracing::debug!(cmd = %cmd_str, "gh command succeeded");
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub async fn gh_repo_view(
    _state: tauri::State<'_, Arc<AppState>>,
    project: String,
) -> Result<Value, String> {
    async fn _fn(project: &str) -> Result<Value, AppError> {
        let path = resolve_project_path(project).await?;
        tracing::info!(project = %project, "loading GitHub repo info");
        let json_fields = "name,owner,description,url,homepageUrl,defaultBranchRef,\
            stargazerCount,forkCount,isPrivate,primaryLanguage,repositoryTopics,\
            createdAt,updatedAt,diskUsage,licenseInfo";
        let output = run_gh(&path, &["repo", "view", "--json", json_fields]).await?;
        let mut value: Value =
            serde_json::from_str(&output).map_err(|e| AppError::Internal(e.into()))?;

        // gh CLI returns null for empty arrays/strings; normalize for frontend deserialization
        if let Value::Object(ref mut map) = value {
            if map.get("repositoryTopics").is_some_and(Value::is_null) {
                map.insert("repositoryTopics".to_string(), Value::Array(vec![]));
            }
            for key in ["description", "homepageUrl", "createdAt", "updatedAt"] {
                if map.get(key).is_some_and(Value::is_null) {
                    map.insert(key.to_string(), Value::String(String::new()));
                }
            }
        }

        Ok(value)
    }
    _fn(&project).await.map_err(|e| e.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhRepoCreateRequest {
    pub project: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_private: bool,
    #[serde(default = "default_true")]
    pub push: bool,
}

fn default_true() -> bool {
    true
}

#[tauri::command]
pub async fn gh_repo_create(
    _state: tauri::State<'_, Arc<AppState>>,
    body: GhRepoCreateRequest,
) -> Result<Value, String> {
    async fn _fn(body: &GhRepoCreateRequest) -> Result<Value, AppError> {
        let path = resolve_project_path(&body.project).await?;
        tracing::info!(project = %body.project, repo = %body.name, private = %body.is_private, "creating GitHub repository");

        let mut args = vec!["repo", "create", &body.name, "--source=."];

        if body.is_private {
            args.push("--private");
        } else {
            args.push("--public");
        }

        let desc_flag;
        if !body.description.is_empty() {
            desc_flag = format!("--description={}", body.description);
            args.push(&desc_flag);
        }

        if body.push {
            args.push("--push");
        }

        let output = run_gh(&path, &args).await?;
        tracing::info!(project = %body.project, repo = %body.name, "GitHub repository created successfully");
        Ok(json!({ "success": true, "output": output.trim() }))
    }
    _fn(&body).await.map_err(|e| e.to_string())
}
