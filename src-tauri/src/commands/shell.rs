use std::collections::{HashSet, VecDeque};
use std::io::Read;
use std::sync::Arc;

use portable_pty::PtySize;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::Emitter;
use tracing::{debug, info, warn};

use crate::error::AppError;
use crate::services::pty_manager;
use crate::state::{AppState, PtySession, PTY_BUFFER_CAP};

// --- Event Payloads ----------------------------------------------------------

#[derive(Clone, Serialize)]
struct ShellOutputPayload {
    session_key: String,
    data: String,
}

#[derive(Clone, Serialize)]
struct ShellAuthUrlPayload {
    session_key: String,
    url: String,
}

/// Maximum number of concurrent PTY sessions.
const MAX_PTY_SESSIONS: usize = 10;

/// Timeout before killing a disconnected PTY session (30 minutes).
const PTY_SESSION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30 * 60);

// --- Shell Init --------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellInitParams {
    pub project_path: String,
    pub session_id: Option<String>,
    #[serde(default)]
    pub has_session: bool,
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_cols")]
    pub cols: u16,
    #[serde(default = "default_rows")]
    pub rows: u16,
    pub initial_command: Option<String>,
    #[serde(default)]
    pub is_plain_shell: bool,
}

fn default_provider() -> String {
    "claude".to_string()
}
fn default_cols() -> u16 {
    80
}
fn default_rows() -> u16 {
    24
}

/// Initialize or reconnect a PTY shell session.
/// Returns `{ sessionKey, buffer }` — the buffer contains replay data for reconnection.
#[tauri::command]
pub async fn shell_init(
    state: tauri::State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
    params: ShellInitParams,
) -> Result<Value, String> {
    async fn _shell_init(
        state: &Arc<AppState>,
        app: &tauri::AppHandle,
        params: ShellInitParams,
    ) -> Result<Value, AppError> {
        let resolved_path = pty_manager::validate_project_path(&params.project_path)?;
        if let Some(ref sid) = params.session_id {
            pty_manager::validate_session_id(sid)?;
        }

        let is_login = pty_manager::is_login_command(params.initial_command.as_deref());

        let session_key = pty_manager::compute_session_key(
            &params.project_path,
            params.session_id.as_deref(),
            params.initial_command.as_deref(),
            params.is_plain_shell,
        );

        let mut sessions = state.pty_sessions.lock().await;

        // For login commands, always kill existing session and create fresh
        if is_login && let Some(mut existing) = sessions.remove(&session_key) {
            info!(%session_key, "Killing existing session for login command");
            let _ = existing.child.kill();
            if let Some(h) = existing.cleanup_handle.take() {
                h.abort();
            }
            if let Some(h) = existing.reader_handle.take() {
                h.abort();
            }
        }

        // Check for existing session (reconnect)
        if let Some(session) = sessions.get_mut(&session_key) {
            info!(%session_key, "Reconnecting to existing PTY session");

            // Cancel cleanup timer
            if let Some(handle) = session.cleanup_handle.take() {
                handle.abort();
            }

            // Collect buffer for replay
            let buffer: Vec<String> = session.buffer.iter().cloned().collect();

            // Resize to current terminal dimensions
            let _ = session.master.resize(PtySize {
                rows: params.rows,
                cols: params.cols,
                pixel_width: 0,
                pixel_height: 0,
            });

            return Ok(json!({
                "sessionKey": session_key,
                "buffer": buffer,
                "reconnected": true,
            }));
        }

        // Check session limit
        if sessions.len() >= MAX_PTY_SESSIONS {
            return Err(AppError::BadRequest(
                "Too many active PTY sessions. Close some terminals first.".to_string(),
            ));
        }

        // Build and spawn
        let (shell, args) = pty_manager::build_shell_command(
            &params.provider,
            params.has_session,
            params.session_id.as_deref(),
            params.initial_command.as_deref(),
            params.is_plain_shell,
        );

        let cwd = resolved_path.to_string_lossy().to_string();
        let spawned = pty_manager::spawn_pty(&shell, &args, &cwd, params.cols, params.rows)?;

        info!(
            %session_key, provider = %params.provider, %cwd,
            "New PTY session created"
        );

        // Start PTY reader task — emits shell_output events via AppHandle
        let reader_app = app.clone();
        let reader_state = state.clone();
        let reader_key = session_key.clone();
        let mut reader = spawned.reader;

        let reader_handle = tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            let mut url_buffer = String::new();
            let mut announced_urls = HashSet::new();

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]).to_string();

                        // Detect auth URLs
                        let detected = pty_manager::detect_auth_urls(
                            &mut url_buffer,
                            &data,
                            &mut announced_urls,
                        );
                        for (url, _auto_open) in detected {
                            let _ = reader_app.emit("shell_auth_url", ShellAuthUrlPayload {
                                session_key: reader_key.clone(),
                                url,
                            });
                        }

                        // Emit output event
                        let _ = reader_app.emit("shell_output", ShellOutputPayload {
                            session_key: reader_key.clone(),
                            data: data.clone(),
                        });

                        // Buffer update via runtime handle
                        let state2 = reader_state.clone();
                        let key2 = reader_key.clone();
                        let data2 = data;
                        if let Ok(handle) = tokio::runtime::Handle::try_current() {
                            handle.spawn(async move {
                                let mut sessions = state2.pty_sessions.lock().await;
                                if let Some(session) = sessions.get_mut(&key2) {
                                    pty_manager::push_buffer(&mut session.buffer, data2);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        debug!(error = %e, "PTY read ended");
                        break;
                    }
                }
            }

            // Process exited
            info!(key = %reader_key, "PTY process ended");

            let exit_msg = {
                let state3 = reader_state.clone();
                let key3 = reader_key.clone();
                if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    let exit_code = handle.block_on(async {
                        let mut sessions = state3.pty_sessions.lock().await;
                        if let Some(session) = sessions.get_mut(&key3) {
                            session.child.try_wait().ok().flatten().map(|s| {
                                s.exit_code()
                            })
                        } else {
                            None
                        }
                    });
                    match exit_code {
                        Some(code) => format!(
                            "\r\n\x1b[33mProcess exited with code {code}\x1b[0m\r\n"
                        ),
                        None => "\r\n\x1b[33mProcess exited\x1b[0m\r\n".to_string(),
                    }
                } else {
                    "\r\n\x1b[33mProcess exited\x1b[0m\r\n".to_string()
                }
            };

            let _ = reader_app.emit("shell_output", ShellOutputPayload {
                session_key: reader_key.clone(),
                data: exit_msg,
            });

            // Remove session from map
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let state4 = reader_state.clone();
                let key4 = reader_key;
                handle.spawn(async move {
                    let mut sessions = state4.pty_sessions.lock().await;
                    if let Some(mut removed) = sessions.remove(&key4)
                        && let Some(h) = removed.cleanup_handle.take()
                    {
                        h.abort();
                    }
                });
            }
        });

        // Register session
        let pty_session = PtySession {
            master: spawned.master,
            writer: spawned.writer,
            child: spawned.child,
            project_path: params.project_path,
            session_id: params.session_id.unwrap_or_default(),
            provider: params.provider,
            buffer: VecDeque::with_capacity(PTY_BUFFER_CAP),
            reader_handle: Some(reader_handle),
            cleanup_handle: None,
        };

        sessions.insert(session_key.clone(), pty_session);

        Ok(json!({
            "sessionKey": session_key,
            "buffer": [],
            "reconnected": false,
        }))
    }

    _shell_init(state.inner(), &app, params)
        .await
        .map_err(|e| e.to_string())
}

// --- Shell Input -------------------------------------------------------------

#[tauri::command]
pub async fn shell_input(
    state: tauri::State<'_, Arc<AppState>>,
    session_key: String,
    data: String,
) -> Result<Value, String> {
    let mut sessions = state.pty_sessions.lock().await;
    if let Some(session) = sessions.get_mut(&session_key)
        && let Err(e) = session.writer.write_all(data.as_bytes())
    {
        warn!(error = %e, "Failed to write to PTY");
    }
    Ok(json!({"ok": true}))
}

// --- Shell Resize ------------------------------------------------------------

#[tauri::command]
pub async fn shell_resize(
    state: tauri::State<'_, Arc<AppState>>,
    session_key: String,
    cols: u16,
    rows: u16,
) -> Result<Value, String> {
    let sessions = state.pty_sessions.lock().await;
    if let Some(session) = sessions.get(&session_key)
        && let Err(e) = session.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
    {
        warn!(error = %e, "Failed to resize PTY");
    }
    Ok(json!({"ok": true}))
}

// --- Shell Detach ------------------------------------------------------------

/// Detach from a PTY session. Starts a cleanup timer that will kill the
/// session after 30 minutes if not reconnected.
#[tauri::command]
pub async fn shell_detach(
    state: tauri::State<'_, Arc<AppState>>,
    session_key: String,
) -> Result<Value, String> {
    let mut sessions = state.pty_sessions.lock().await;
    if let Some(session) = sessions.get_mut(&session_key) {
        // Start cleanup timer
        let state2 = state.inner().clone();
        let key = session_key.clone();
        let cleanup_handle = tokio::spawn(async move {
            tokio::time::sleep(PTY_SESSION_TIMEOUT).await;
            info!(%key, "PTY session timeout — killing");
            let mut sessions = state2.pty_sessions.lock().await;
            if let Some(mut removed) = sessions.remove(&key) {
                let _ = removed.child.kill();
                if let Some(h) = removed.reader_handle.take() {
                    h.abort();
                }
            }
        });

        session.cleanup_handle = Some(cleanup_handle);
        info!(%session_key, "PTY session detached, cleanup timer started (30 min)");
    }
    Ok(json!({"ok": true}))
}
