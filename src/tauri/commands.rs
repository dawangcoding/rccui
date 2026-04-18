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
    call("shell_init", &params).await
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
