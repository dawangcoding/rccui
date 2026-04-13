use std::sync::Arc;

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::AsyncBufReadExt;
use tracing::debug;

use crate::db;
use crate::error::AppError;
use crate::providers::claude::adapter::ClaudeAdapter;
use crate::providers::codex::adapter::CodexAdapter;
use crate::providers::cursor::adapter::CursorAdapter;
use crate::providers::gemini::adapter::GeminiAdapter;
use crate::providers::types::{FetchHistoryOptions, SessionProvider};
use crate::providers::ProviderAdapter;
use crate::services::project_scanner;
use crate::state::AppState;

// ─── List Sessions ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_sessions(
    state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    provider: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value, String> {
    _list_sessions(&state, &project_name, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

async fn _list_sessions(
    state: &AppState,
    project_name: &str,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value, AppError> {
    let limit = limit.map(|l| l as usize);
    let offset = offset.unwrap_or(0) as usize;

    debug!(%project_name, ?limit, offset, "Listing sessions");

    let (mut sessions, total, has_more) =
        project_scanner::get_claude_sessions(project_name, limit, offset).await;

    // Apply custom session names
    let session_ids: Vec<String> = sessions.iter().map(|s| s.id.clone()).collect();
    if !session_ids.is_empty() {
        if let Ok(names) =
            db::session_names::get_names_batch(&state.db, &session_ids, "claude").await
        {
            for session in sessions.iter_mut() {
                if let Some(custom_name) = names.get(&session.id) {
                    session.name = Some(custom_name.clone());
                    session.summary = custom_name.clone();
                }
            }
        }
    }

    debug!(%project_name, total, has_more, "Sessions listed");

    Ok(json!({
        "sessions": sessions,
        "total": total,
        "hasMore": has_more
    }))
}

// ─── Get Session Messages ────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_session_messages(
    _state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
    provider: Option<String>,
    project_name: Option<String>,
    project_path: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value, String> {
    _get_session_messages(
        &session_id,
        provider.as_deref(),
        project_name,
        project_path,
        limit,
        offset,
    )
    .await
    .map_err(|e| e.to_string())
}

async fn _get_session_messages(
    session_id: &str,
    provider_str: Option<&str>,
    project_name: Option<String>,
    project_path: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value, AppError> {
    let provider_str = provider_str.unwrap_or("claude");
    let provider: SessionProvider = provider_str
        .parse()
        .map_err(|e: String| AppError::BadRequest(e))?;

    debug!(%session_id, %provider_str, "Fetching session messages");

    let opts = FetchHistoryOptions {
        project_name,
        project_path,
        limit,
        offset: offset.unwrap_or(0),
    };

    let result = match provider {
        SessionProvider::Claude => ClaudeAdapter.fetch_history(session_id, opts).await?,
        SessionProvider::Cursor => CursorAdapter.fetch_history(session_id, opts).await?,
        SessionProvider::Codex => CodexAdapter.fetch_history(session_id, opts).await?,
        SessionProvider::Gemini => GeminiAdapter.fetch_history(session_id, opts).await?,
    };

    debug!(
        %session_id,
        %provider_str,
        total = result.total,
        returned = result.messages.len(),
        "Session messages fetched"
    );

    Ok(serde_json::to_value(&result).unwrap_or_default())
}

// ─── Delete Sessions ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn delete_session(
    _state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
    provider: Option<String>,
    project_name: Option<String>,
) -> Result<Value, String> {
    _delete_session(&session_id, provider.as_deref(), project_name.as_deref())
        .await
        .map_err(|e| e.to_string())
}

async fn _delete_session(
    session_id: &str,
    provider: Option<&str>,
    project_name: Option<&str>,
) -> Result<Value, AppError> {
    let provider = provider.unwrap_or("claude");
    let project_name = project_name.unwrap_or("");

    tracing::info!(%session_id, %provider, "Deleting session");

    project_scanner::delete_session(project_name, session_id, provider).await?;

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn delete_project_session(
    _state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    session_id: String,
) -> Result<Value, String> {
    _delete_project_session(&project_name, &session_id)
        .await
        .map_err(|e| e.to_string())
}

async fn _delete_project_session(
    project_name: &str,
    session_id: &str,
) -> Result<Value, AppError> {
    tracing::info!(%session_id, %project_name, "Deleting project session");
    project_scanner::delete_session(project_name, session_id, "claude").await?;
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn delete_codex_session(
    _state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<Value, String> {
    _delete_codex_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

async fn _delete_codex_session(session_id: &str) -> Result<Value, AppError> {
    tracing::info!(%session_id, "Deleting Codex session");
    project_scanner::delete_session("", session_id, "codex").await?;
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn delete_gemini_session(
    _state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<Value, String> {
    _delete_gemini_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

async fn _delete_gemini_session(session_id: &str) -> Result<Value, AppError> {
    tracing::info!(%session_id, "Deleting Gemini session");
    project_scanner::delete_session("", session_id, "gemini").await?;
    Ok(json!({ "success": true }))
}

// ─── Session Naming ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameSessionRequest {
    pub summary: String,
    pub provider: Option<String>,
}

#[tauri::command]
pub async fn rename_session(
    state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
    body: RenameSessionRequest,
) -> Result<Value, String> {
    _rename_session(&state, &session_id, body)
        .await
        .map_err(|e| e.to_string())
}

async fn _rename_session(
    state: &AppState,
    session_id: &str,
    body: RenameSessionRequest,
) -> Result<Value, AppError> {
    let provider = body.provider.as_deref().unwrap_or("claude");
    debug!(%session_id, %provider, name = %body.summary, "Renaming session");
    db::session_names::set_name(&state.db, session_id, provider, &body.summary).await?;

    // Invalidate project cache
    {
        let mut cache = state.project_cache.write().await;
        *cache = None;
    }

    Ok(json!({ "success": true }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetNameRequest {
    pub name: String,
    pub provider: Option<String>,
}

#[tauri::command]
pub async fn set_session_name(
    state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
    body: SetNameRequest,
) -> Result<Value, String> {
    _set_session_name(&state, &session_id, body)
        .await
        .map_err(|e| e.to_string())
}

async fn _set_session_name(
    state: &AppState,
    session_id: &str,
    body: SetNameRequest,
) -> Result<Value, AppError> {
    let provider = body.provider.as_deref().unwrap_or("claude");
    debug!(%session_id, %provider, name = %body.name, "Setting session name");
    db::session_names::set_name(&state.db, session_id, provider, &body.name).await?;

    // Invalidate project cache
    {
        let mut cache = state.project_cache.write().await;
        *cache = None;
    }

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn delete_session_name(
    state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
    provider: Option<String>,
) -> Result<Value, String> {
    _delete_session_name(&state, &session_id, provider.as_deref())
        .await
        .map_err(|e| e.to_string())
}

async fn _delete_session_name(
    state: &AppState,
    session_id: &str,
    provider: Option<&str>,
) -> Result<Value, AppError> {
    let provider = provider.unwrap_or("claude");
    debug!(%session_id, %provider, "Deleting session name");
    db::session_names::delete_name(&state.db, session_id, provider).await?;
    Ok(json!({ "success": true }))
}

// ─── Token Usage ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_session_token_usage(
    state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    session_id: String,
    provider: Option<String>,
) -> Result<Value, String> {
    _get_session_token_usage(&state, &project_name, &session_id, provider.as_deref())
        .await
        .map_err(|e| e.to_string())
}

async fn _get_session_token_usage(
    state: &AppState,
    project_name: &str,
    session_id: &str,
    provider_str: Option<&str>,
) -> Result<Value, AppError> {
    let provider_str = provider_str.unwrap_or("claude");
    debug!(%session_id, %provider_str, %project_name, "Fetching token usage");

    // Validate session ID
    if session_id
        .chars()
        .any(|c| !c.is_alphanumeric() && c != '-' && c != '_' && c != '.')
    {
        return Err(AppError::BadRequest("Invalid sessionId".to_string()));
    }

    // Cursor and Gemini don't support token usage tracking
    if provider_str == "cursor" || provider_str == "gemini" {
        return Ok(json!({
            "used": 0,
            "total": 0,
            "unsupported": true,
            "message": format!("Token usage tracking not available for {} sessions", provider_str)
        }));
    }

    // Codex: scan JSONL for token_count events
    if provider_str == "codex" {
        return get_codex_token_usage(session_id).await;
    }

    // Claude: scan JSONL for latest assistant message with usage
    get_claude_token_usage(state, project_name, session_id).await
}

async fn get_claude_token_usage(
    state: &AppState,
    project_name: &str,
    session_id: &str,
) -> Result<Value, AppError> {
    use crate::providers::claude::adapter::extract_token_usage;

    let home = dirs::home_dir().unwrap_or_default();
    let projects_dir = home.join(".claude").join("projects");
    let session_file = projects_dir
        .join(project_name)
        .join(format!("{session_id}.jsonl"));

    if !session_file.exists() {
        return Err(AppError::NotFound("Session file not found".to_string()));
    }

    let file = tokio::fs::File::open(&session_file).await?;
    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();
    let mut raw_messages: Vec<Value> = Vec::new();

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
            raw_messages.push(parsed);
        }
    }

    let context_window = state.config.context_window as u64;

    match extract_token_usage(&raw_messages) {
        Some(mut usage) => {
            // Override total with configured context window
            if let Some(obj) = usage.as_object_mut() {
                obj.insert("total".to_string(), json!(context_window));
            }
            Ok(usage)
        }
        None => Ok(json!({
            "used": 0,
            "total": context_window,
            "breakdown": { "input": 0, "cacheCreation": 0, "cacheRead": 0 }
        })),
    }
}

async fn get_codex_token_usage(session_id: &str) -> Result<Value, AppError> {
    let home = dirs::home_dir().unwrap_or_default();
    let codex_dir = home.join(".codex").join("sessions");

    if !codex_dir.exists() {
        return Err(AppError::NotFound(
            "Codex sessions directory not found".to_string(),
        ));
    }

    // Find session file recursively
    let session_file = find_codex_session_file(&codex_dir, session_id).await;
    let session_file = match session_file {
        Some(f) => f,
        None => {
            return Err(AppError::NotFound(
                "Codex session file not found".to_string(),
            ))
        }
    };

    let file = tokio::fs::File::open(&session_file).await?;
    let reader = tokio::io::BufReader::new(file);
    let mut lines_vec: Vec<String> = Vec::new();
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        lines_vec.push(line);
    }

    let mut total_tokens: u64 = 0;
    let mut context_window: u64 = 200000;

    // Scan from end for the latest token_count event
    for line in lines_vec.iter().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<Value>(trimmed) {
            if entry.get("type").and_then(|v| v.as_str()) == Some("event_msg") {
                let payload = entry.get("payload");
                if payload.and_then(|p| p.get("type")).and_then(|v| v.as_str())
                    == Some("token_count")
                {
                    if let Some(info) = payload.and_then(|p| p.get("info")) {
                        if let Some(total) = info
                            .get("total_token_usage")
                            .and_then(|t| t.get("total_tokens"))
                            .and_then(|v| v.as_u64())
                        {
                            total_tokens = total;
                        }
                        if let Some(cw) =
                            info.get("model_context_window").and_then(|v| v.as_u64())
                        {
                            context_window = cw;
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(json!({
        "used": total_tokens,
        "total": context_window
    }))
}

async fn find_codex_session_file(
    dir: &std::path::Path,
    session_id: &str,
) -> Option<std::path::PathBuf> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(e) => e,
        Err(_) => return None,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = Box::pin(find_codex_session_file(&path, session_id)).await {
                return Some(found);
            }
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.contains(session_id) && name.ends_with(".jsonl") {
                return Some(path);
            }
        }
    }
    None
}
