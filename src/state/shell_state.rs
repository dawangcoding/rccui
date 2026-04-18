use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::tauri::commands;
use crate::tauri::types::{ShellInitParams, ShellOutputPayload, ShellAuthUrlPayload};

// ─── ShellContext ────────────────────────────────────────────────────────────

/// Shell/terminal state for the active PTY session. Provided at the App level.
#[derive(Clone, Copy)]
pub struct ShellContext {
    /// PTY session key returned by shell_init (used for backend commands)
    pub session_key: RwSignal<Option<String>>,
    /// xterm.js instance ID (used for frontend rendering, e.g. "term_hi3")
    pub terminal_id: RwSignal<Option<String>>,
    /// Whether the PTY is connected and running
    pub is_connected: RwSignal<bool>,
    /// Whether shell_init is in progress
    pub is_initializing: RwSignal<bool>,
    /// Error message from last operation
    pub error_message: RwSignal<Option<String>>,
    /// Auth URL detected from PTY output (e.g., Claude login)
    pub auth_url: RwSignal<Option<String>>,
}

impl ShellContext {
    pub fn new() -> Self {
        Self {
            session_key: RwSignal::new(None),
            terminal_id: RwSignal::new(None),
            is_connected: RwSignal::new(false),
            is_initializing: RwSignal::new(false),
            error_message: RwSignal::new(None),
            auth_url: RwSignal::new(None),
        }
    }

    /// Initialize or reconnect a PTY session.
    /// `term_id` is the xterm.js instance ID (e.g. "term_hi3").
    pub fn init_session(
        &self,
        term_id: String,
        project_path: String,
        session_id: Option<String>,
        provider: String,
        cols: u16,
        rows: u16,
    ) {
        let ctx = *self;
        ctx.terminal_id.set(Some(term_id));
        ctx.is_initializing.set(true);
        ctx.error_message.set(None);
        ctx.auth_url.set(None);

        let has_session = session_id.is_some();

        spawn_local(async move {
            let params = ShellInitParams {
                project_path,
                session_id,
                has_session,
                provider,
                cols,
                rows,
                initial_command: None,
                is_plain_shell: true,
            };

            match commands::shell_init(params).await {
                Ok(result) => {
                    ctx.session_key.set(Some(result.session_key));
                    ctx.is_connected.set(true);

                    // Write replay buffer to terminal using the xterm instance ID
                    if !result.buffer.is_empty() {
                        if let Some(tid) = ctx.terminal_id.get_untracked() {
                            let replay = result.buffer.join("");
                            xterm_write(&tid, &replay);
                        }
                    }
                }
                Err(e) => {
                    ctx.error_message.set(Some(e));
                    ctx.is_connected.set(false);
                }
            }
            ctx.is_initializing.set(false);
        });
    }

    /// Send user input to the PTY.
    pub fn send_input(&self, data: String) {
        let ctx = *self;
        if let Some(key) = ctx.session_key.get_untracked() {
            spawn_local(async move {
                if let Err(e) = commands::shell_input(&key, &data).await {
                    ctx.error_message.set(Some(format!("Input error: {e}")));
                }
            });
        }
    }

    /// Handle incoming shell_output event — write to xterm.
    pub fn handle_output(&self, payload: ShellOutputPayload) {
        // Only handle output for the active session
        if let Some(key) = self.session_key.get_untracked() {
            if payload.session_key == key {
                // Use terminal_id (xterm instance ID), not session_key
                if let Some(tid) = self.terminal_id.get_untracked() {
                    xterm_write(&tid, &payload.data);
                }

                // Check if process exited (contains exit message)
                if payload.data.contains("Process exited") {
                    self.is_connected.set(false);
                }
            }
        }
    }

    /// Handle incoming shell_auth_url event.
    pub fn handle_auth_url(&self, payload: ShellAuthUrlPayload) {
        if let Some(key) = self.session_key.get_untracked() {
            if payload.session_key == key {
                self.auth_url.set(Some(payload.url));
            }
        }
    }

    /// Resize the PTY to match terminal dimensions.
    pub fn resize(&self, cols: u16, rows: u16) {
        let ctx = *self;
        if let Some(key) = ctx.session_key.get_untracked() {
            spawn_local(async move {
                let _ = commands::shell_resize(&key, cols, rows).await;
            });
        }
    }

    /// Detach from the current session (starts 30-min cleanup timer).
    pub fn detach(&self) {
        let ctx = *self;
        if let Some(key) = ctx.session_key.get_untracked() {
            spawn_local(async move {
                let _ = commands::shell_detach(&key).await;
            });
        }
    }

    /// Clear shell state (e.g., when switching projects).
    pub fn clear(&self) {
        // Detach from current session first
        self.detach();
        // Dispose xterm instance using the correct terminal_id
        if let Some(tid) = self.terminal_id.get_untracked() {
            xterm_dispose(&tid);
        }
        self.session_key.set(None);
        self.terminal_id.set(None);
        self.is_connected.set(false);
        self.is_initializing.set(false);
        self.error_message.set(None);
        self.auth_url.set(None);
    }
}

// ─── JS Interop helpers (call XtermBridge from Rust) ─────────────────────────

/// Write data to the xterm instance.
fn xterm_write(id: &str, data: &str) {
    let _ = js_sys::Reflect::apply(
        &js_sys::Function::from(
            js_sys::Reflect::get(
                &xterm_bridge(),
                &wasm_bindgen::JsValue::from_str("write"),
            )
            .unwrap(),
        ),
        &xterm_bridge(),
        &js_sys::Array::of2(
            &wasm_bindgen::JsValue::from_str(id),
            &wasm_bindgen::JsValue::from_str(data),
        ),
    );
}

/// Dispose the xterm instance.
fn xterm_dispose(id: &str) {
    let _ = js_sys::Reflect::apply(
        &js_sys::Function::from(
            js_sys::Reflect::get(
                &xterm_bridge(),
                &wasm_bindgen::JsValue::from_str("dispose"),
            )
            .unwrap(),
        ),
        &xterm_bridge(),
        &js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(id)),
    );
}

/// Get the `window.XtermBridge` object.
fn xterm_bridge() -> wasm_bindgen::JsValue {
    js_sys::Reflect::get(
        &wasm_bindgen::JsValue::from(web_sys::window().unwrap()),
        &wasm_bindgen::JsValue::from_str("XtermBridge"),
    )
    .unwrap()
}
