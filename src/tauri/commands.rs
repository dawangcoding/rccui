use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use super::types::*;

// ─── Tauri invoke binding ────────────────────────────────────────────────────

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Helper: invoke a Tauri command with typed args and result.
async fn call<A: Serialize, R: serde::de::DeserializeOwned>(
    cmd: &str,
    args: &A,
) -> Result<R, String> {
    let js_args =
        serde_wasm_bindgen::to_value(args).map_err(|e| format!("Serialize error: {e}"))?;
    let result = invoke(cmd, js_args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Unknown invoke error".into()))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Deserialize error: {e}"))
}

/// Helper: invoke with no args.
async fn call_no_args<R: serde::de::DeserializeOwned>(cmd: &str) -> Result<R, String> {
    let js_args = JsValue::from(js_sys::Object::new());
    let result = invoke(cmd, js_args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Unknown invoke error".into()))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Deserialize error: {e}"))
}

// ─── Project Commands ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ListProjectsArgs {
    refresh: Option<bool>,
}

pub async fn list_projects(refresh: bool) -> Result<Vec<Project>, String> {
    let args = ListProjectsArgs {
        refresh: Some(refresh),
    };
    let val: Value = call("list_projects", &args).await?;
    // Backend returns Value::Array directly
    let projects: Vec<Project> = serde_json::from_value(val).map_err(|e| e.to_string())?;
    Ok(projects)
}

// ─── Session Commands ────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListSessionsArgs {
    project_name: String,
    provider: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub async fn list_sessions(
    project_name: &str,
    provider: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<SessionListResponse, String> {
    let args = ListSessionsArgs {
        project_name: project_name.to_string(),
        provider: provider.map(|s| s.to_string()),
        limit,
        offset,
    };
    call("list_sessions", &args).await
}

// ─── User / Onboarding Commands ──────────────────────────────────────────────

pub async fn get_onboarding_status() -> Result<OnboardingStatus, String> {
    call_no_args("get_onboarding_status").await
}

pub async fn complete_onboarding() -> Result<Value, String> {
    call_no_args("complete_onboarding").await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RenameSessionArgs {
    session_id: String,
    body: RenameSessionBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RenameSessionBody {
    summary: String,
    provider: Option<String>,
}

pub async fn rename_session(
    session_id: &str,
    summary: &str,
    provider: Option<&str>,
) -> Result<Value, String> {
    let args = RenameSessionArgs {
        session_id: session_id.to_string(),
        body: RenameSessionBody {
            summary: summary.to_string(),
            provider: provider.map(|s| s.to_string()),
        },
    };
    call("rename_session", &args).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteSessionArgs {
    session_id: String,
    provider: Option<String>,
    project_name: Option<String>,
}

pub async fn delete_session(
    session_id: &str,
    provider: Option<&str>,
    project_name: Option<&str>,
) -> Result<Value, String> {
    let args = DeleteSessionArgs {
        session_id: session_id.to_string(),
        provider: provider.map(|s| s.to_string()),
        project_name: project_name.map(|s| s.to_string()),
    };
    call("delete_session", &args).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetSessionNameArgs {
    session_id: String,
    body: SetSessionNameBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetSessionNameBody {
    name: String,
    provider: Option<String>,
}

pub async fn set_session_name(
    session_id: &str,
    name: &str,
    provider: Option<&str>,
) -> Result<Value, String> {
    let args = SetSessionNameArgs {
        session_id: session_id.to_string(),
        body: SetSessionNameBody {
            name: name.to_string(),
            provider: provider.map(|s| s.to_string()),
        },
    };
    call("set_session_name", &args).await
}

// ─── Chat Commands ───────────────────────────────────────────────────────────

pub async fn chat_execute(command: Value) -> Result<Value, String> {
    #[derive(Serialize)]
    struct Args {
        command: Value,
    }
    call("chat_execute", &Args { command }).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetSessionMessagesArgs {
    session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
}

pub async fn get_session_messages(
    session_id: &str,
    provider: Option<&str>,
    project_name: Option<&str>,
    project_path: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<FetchHistoryResult, String> {
    let args = GetSessionMessagesArgs {
        session_id: session_id.to_string(),
        provider: provider.map(|s| s.to_string()),
        project_name: project_name.map(|s| s.to_string()),
        project_path: project_path.map(|s| s.to_string()),
        limit,
        offset,
    };
    call("get_session_messages", &args).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetSessionTokenUsageArgs {
    project_name: String,
    session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
}

pub async fn get_session_token_usage(
    project_name: &str,
    session_id: &str,
    provider: Option<&str>,
) -> Result<TokenUsage, String> {
    let args = GetSessionTokenUsageArgs {
        project_name: project_name.to_string(),
        session_id: session_id.to_string(),
        provider: provider.map(|s| s.to_string()),
    };
    call("get_session_token_usage", &args).await
}

// ─── Shell Commands ──────────────────────────────────────────────────────────

pub async fn shell_init(params: ShellInitParams) -> Result<ShellInitResult, String> {
    #[derive(Serialize)]
    struct Args {
        params: ShellInitParams,
    }
    call("shell_init", &Args { params }).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShellInputArgs {
    session_key: String,
    data: String,
}

pub async fn shell_input(session_key: &str, data: &str) -> Result<Value, String> {
    call(
        "shell_input",
        &ShellInputArgs {
            session_key: session_key.to_string(),
            data: data.to_string(),
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShellResizeArgs {
    session_key: String,
    cols: u16,
    rows: u16,
}

pub async fn shell_resize(session_key: &str, cols: u16, rows: u16) -> Result<Value, String> {
    call(
        "shell_resize",
        &ShellResizeArgs {
            session_key: session_key.to_string(),
            cols,
            rows,
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShellDetachArgs {
    session_key: String,
}

pub async fn shell_detach(session_key: &str) -> Result<Value, String> {
    call(
        "shell_detach",
        &ShellDetachArgs {
            session_key: session_key.to_string(),
        },
    )
    .await
}

// ─── File Commands ───────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListFilesArgs {
    project_name: String,
}

pub async fn list_files(project_name: &str) -> Result<Vec<FileNode>, String> {
    let args = ListFilesArgs {
        project_name: project_name.to_string(),
    };
    let val: Value = call("list_files", &args).await?;
    let nodes: Vec<FileNode> = serde_json::from_value(val).map_err(|e| e.to_string())?;
    Ok(nodes)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadFileArgs {
    project_name: String,
    file_path: String,
}

pub async fn read_file(project_name: &str, file_path: &str) -> Result<ReadFileResponse, String> {
    call(
        "read_file",
        &ReadFileArgs {
            project_name: project_name.to_string(),
            file_path: file_path.to_string(),
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadFileContentArgs {
    project_name: String,
    path: String,
}

pub async fn read_file_content(
    project_name: &str,
    path: &str,
) -> Result<ReadFileContentResponse, String> {
    call(
        "read_file_content",
        &ReadFileContentArgs {
            project_name: project_name.to_string(),
            path: path.to_string(),
        },
    )
    .await
}

#[derive(Serialize)]
struct ReadRawFileArgs {
    path: String,
}

pub async fn read_raw_file(path: &str) -> Result<ReadFileContentResponse, String> {
    call(
        "read_raw_file",
        &ReadRawFileArgs {
            path: path.to_string(),
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveFileArgs {
    project_name: String,
    body: SaveFileBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveFileBody {
    file_path: String,
    content: String,
}

pub async fn save_file(
    project_name: &str,
    file_path: &str,
    content: &str,
) -> Result<FileOperationResponse, String> {
    call(
        "save_file",
        &SaveFileArgs {
            project_name: project_name.to_string(),
            body: SaveFileBody {
                file_path: file_path.to_string(),
                content: content.to_string(),
            },
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateFileArgs {
    project_name: String,
    body: CreateFileBody,
}

#[derive(Serialize)]
struct CreateFileBody {
    path: String,
    #[serde(rename = "type")]
    item_type: String,
    name: String,
}

pub async fn create_file(
    project_name: &str,
    path: &str,
    item_type: &str,
    name: &str,
) -> Result<FileOperationResponse, String> {
    call(
        "create_file",
        &CreateFileArgs {
            project_name: project_name.to_string(),
            body: CreateFileBody {
                path: path.to_string(),
                item_type: item_type.to_string(),
                name: name.to_string(),
            },
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RenameFileArgs {
    project_name: String,
    body: RenameFileBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RenameFileBody {
    old_path: String,
    new_name: String,
}

pub async fn rename_file(
    project_name: &str,
    old_path: &str,
    new_name: &str,
) -> Result<FileOperationResponse, String> {
    call(
        "rename_file",
        &RenameFileArgs {
            project_name: project_name.to_string(),
            body: RenameFileBody {
                old_path: old_path.to_string(),
                new_name: new_name.to_string(),
            },
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteFileArgs {
    project_name: String,
    body: DeleteFileBody,
}

#[derive(Serialize)]
struct DeleteFileBody {
    path: String,
    #[serde(rename = "type")]
    item_type: String,
}

pub async fn delete_file(
    project_name: &str,
    path: &str,
    item_type: &str,
) -> Result<FileOperationResponse, String> {
    call(
        "delete_file",
        &DeleteFileArgs {
            project_name: project_name.to_string(),
            body: DeleteFileBody {
                path: path.to_string(),
                item_type: item_type.to_string(),
            },
        },
    )
    .await
}

// ─── Git Commands ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct GitProjectArgs {
    project: String,
}

pub async fn git_status(project: &str) -> Result<GitStatus, String> {
    call("git_status", &GitProjectArgs { project: project.to_string() }).await
}

#[derive(Serialize)]
struct GitDiffArgs {
    project: String,
    file: String,
}

pub async fn git_diff(project: &str, file: &str) -> Result<GitDiffResponse, String> {
    call(
        "git_diff",
        &GitDiffArgs {
            project: project.to_string(),
            file: file.to_string(),
        },
    )
    .await
}

pub async fn git_file_with_diff(project: &str, file: &str) -> Result<GitFileWithDiff, String> {
    call(
        "git_file_with_diff",
        &GitDiffArgs {
            project: project.to_string(),
            file: file.to_string(),
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitCommitArgs {
    body: GitCommitBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitCommitBody {
    project: String,
    message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    files: Vec<String>,
}

pub async fn git_commit(
    project: &str,
    message: &str,
    files: Vec<String>,
) -> Result<GitOpResponse, String> {
    call(
        "git_commit",
        &GitCommitArgs {
            body: GitCommitBody {
                project: project.to_string(),
                message: message.to_string(),
                files,
            },
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitInitialCommitArgs {
    body: GitInitialCommitBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitInitialCommitBody {
    project: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    files: Vec<String>,
}

pub async fn git_initial_commit(
    project: &str,
    message: &str,
    files: Vec<String>,
) -> Result<GitOpResponse, String> {
    call(
        "git_initial_commit",
        &GitInitialCommitArgs {
            body: GitInitialCommitBody {
                project: project.to_string(),
                message: message.to_string(),
                files,
            },
        },
    )
    .await
}

pub async fn git_revert_local_commit(project: &str) -> Result<GitOpResponse, String> {
    call(
        "git_revert_local_commit",
        &GitProjectArgs { project: project.to_string() },
    )
    .await
}

pub async fn git_branches(project: &str) -> Result<GitBranches, String> {
    call("git_branches", &GitProjectArgs { project: project.to_string() }).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitBranchArgs {
    body: GitBranchBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitBranchBody {
    project: String,
    branch: String,
}

pub async fn git_checkout(project: &str, branch: &str) -> Result<GitOpResponse, String> {
    call(
        "git_checkout",
        &GitBranchArgs {
            body: GitBranchBody {
                project: project.to_string(),
                branch: branch.to_string(),
            },
        },
    )
    .await
}

pub async fn git_create_branch(project: &str, branch: &str) -> Result<GitOpResponse, String> {
    call(
        "git_create_branch",
        &GitBranchArgs {
            body: GitBranchBody {
                project: project.to_string(),
                branch: branch.to_string(),
            },
        },
    )
    .await
}

pub async fn git_delete_branch(project: &str, branch: &str) -> Result<GitOpResponse, String> {
    call(
        "git_delete_branch",
        &GitBranchArgs {
            body: GitBranchBody {
                project: project.to_string(),
                branch: branch.to_string(),
            },
        },
    )
    .await
}

#[derive(Serialize)]
struct GitCommitsArgs {
    project: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
}

pub async fn git_commits(
    project: &str,
    limit: Option<u32>,
) -> Result<GitCommitsResponse, String> {
    call(
        "git_commits",
        &GitCommitsArgs {
            project: project.to_string(),
            limit,
        },
    )
    .await
}

#[derive(Serialize)]
struct GitCommitDiffArgs {
    project: String,
    commit: String,
}

pub async fn git_commit_diff(project: &str, commit: &str) -> Result<GitDiffResponse, String> {
    call(
        "git_commit_diff",
        &GitCommitDiffArgs {
            project: project.to_string(),
            commit: commit.to_string(),
        },
    )
    .await
}

pub async fn git_remote_status(project: &str) -> Result<GitRemoteStatus, String> {
    call(
        "git_remote_status",
        &GitProjectArgs { project: project.to_string() },
    )
    .await
}

pub async fn git_fetch(project: &str) -> Result<GitOpResponse, String> {
    call("git_fetch", &GitProjectArgs { project: project.to_string() }).await
}

pub async fn git_pull(project: &str) -> Result<GitOpResponse, String> {
    call("git_pull", &GitProjectArgs { project: project.to_string() }).await
}

pub async fn git_push(project: &str) -> Result<GitOpResponse, String> {
    call("git_push", &GitProjectArgs { project: project.to_string() }).await
}

pub async fn git_publish(project: &str, branch: &str) -> Result<GitOpResponse, String> {
    call(
        "git_publish",
        &GitBranchArgs {
            body: GitBranchBody {
                project: project.to_string(),
                branch: branch.to_string(),
            },
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitDiscardArgs {
    body: GitDiscardBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitDiscardBody {
    project: String,
    file: String,
}

pub async fn git_discard(project: &str, file: &str) -> Result<GitOpResponse, String> {
    call(
        "git_discard",
        &GitDiscardArgs {
            body: GitDiscardBody {
                project: project.to_string(),
                file: file.to_string(),
            },
        },
    )
    .await
}

pub async fn git_delete_untracked(project: &str, file: &str) -> Result<GitOpResponse, String> {
    call(
        "git_delete_untracked",
        &GitDiscardArgs {
            body: GitDiscardBody {
                project: project.to_string(),
                file: file.to_string(),
            },
        },
    )
    .await
}

// ─── Opener ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct OpenUrlArgs {
    url: String,
}

pub async fn open_url(url: &str) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&OpenUrlArgs { url: url.to_string() })
        .map_err(|e| format!("Serialize error: {e}"))?;
    invoke("plugin:opener|open_url", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to open URL".into()))?;
    Ok(())
}

// ─── GitHub CLI Commands ─────────────────────────────────────────────────────

pub async fn gh_repo_view(project: &str) -> Result<GitHubRepoInfo, String> {
    call(
        "gh_repo_view",
        &GitProjectArgs {
            project: project.to_string(),
        },
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GhRepoCreateArgs {
    body: GhRepoCreateBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GhRepoCreateBody {
    project: String,
    name: String,
    description: String,
    is_private: bool,
    push: bool,
}

pub async fn gh_repo_create(
    project: &str,
    name: &str,
    description: &str,
    is_private: bool,
    push: bool,
) -> Result<GitOpResponse, String> {
    call(
        "gh_repo_create",
        &GhRepoCreateArgs {
            body: GhRepoCreateBody {
                project: project.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                is_private,
                push,
            },
        },
    )
    .await
}
