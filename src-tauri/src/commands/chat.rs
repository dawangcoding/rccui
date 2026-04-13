use std::sync::Arc;

use serde_json::{Value, json};

use crate::services::chat::{ChatCommand, execute_command};
use crate::state::AppState;

/// Execute a chat command (dispatches to the appropriate provider).
/// The frontend sends the same JSON shape as the original WebSocket ChatCommand.
/// Results stream back as `chat_response` Tauri events.
#[tauri::command]
pub async fn chat_execute(
    state: tauri::State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
    command: Value,
) -> Result<Value, String> {
    let cmd: ChatCommand = serde_json::from_value(command)
        .map_err(|e| format!("Invalid chat command: {e}"))?;

    let state = state.inner().clone();
    let app_clone = app.clone();
    tokio::spawn(async move {
        execute_command(cmd, app_clone, state).await;
    });

    Ok(json!({"status": "started"}))
}
