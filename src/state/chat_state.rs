use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::json;

use crate::tauri::commands;
use crate::tauri::types::{
    ChatResponse, NormalizedMessage, PermissionRequest, SessionProvider, TokenUsage,
};
use crate::ui::toast_custom::toaster::expect_toaster;

// ─── ChatContext ─────────────────────────────────────────────────────────────

/// Chat state for the active session. Provided at the App level.
#[derive(Clone, Copy)]
pub struct ChatContext {
    /// Messages for the current session
    pub messages: RwSignal<Vec<NormalizedMessage>>,
    /// Whether a response is currently streaming
    pub is_streaming: RwSignal<bool>,
    /// Current provider
    pub provider: RwSignal<SessionProvider>,
    /// Current model override (None = provider default)
    pub model: RwSignal<Option<String>>,
    /// Permission mode: "default", "auto-accept", "auto-deny"
    pub permission_mode: RwSignal<String>,
    /// Pending permission request from the assistant
    pub pending_permission: RwSignal<Option<PermissionRequest>>,
    /// Active session ID being chatted with
    pub active_session_id: RwSignal<Option<String>>,
    /// Whether historical messages are loading
    pub messages_loading: RwSignal<bool>,
    /// Accumulated stream content for the current assistant response
    pub stream_content: RwSignal<String>,
    /// Token usage for the current session
    pub token_usage: RwSignal<Option<TokenUsage>>,
}

impl ChatContext {
    pub fn new() -> Self {
        Self {
            messages: RwSignal::new(Vec::new()),
            is_streaming: RwSignal::new(false),
            provider: RwSignal::new(SessionProvider::Claude),
            model: RwSignal::new(None),
            permission_mode: RwSignal::new("default".to_string()),
            pending_permission: RwSignal::new(None),
            active_session_id: RwSignal::new(None),
            messages_loading: RwSignal::new(false),
            stream_content: RwSignal::new(String::new()),
            token_usage: RwSignal::new(None),
        }
    }

    /// Load messages for a session from history.
    pub fn load_messages(
        &self,
        session_id: String,
        provider: SessionProvider,
        project_name: Option<String>,
    ) {
        let ctx = *self;
        ctx.messages_loading.set(true);
        ctx.messages.set(Vec::new());
        ctx.active_session_id.set(Some(session_id.clone()));
        ctx.stream_content.set(String::new());
        ctx.pending_permission.set(None);
        ctx.is_streaming.set(false);

        spawn_local(async move {
            let provider_str = provider.to_string();
            match commands::get_session_messages(
                &session_id,
                Some(&provider_str),
                project_name.as_deref(),
                None,
                None,
                None,
            )
            .await
            {
                Ok(result) => {
                    ctx.messages.set(result.messages);
                    if let Some(usage) = result.token_usage {
                        if let Ok(token_usage) = serde_json::from_value::<TokenUsage>(usage) {
                            ctx.token_usage.set(Some(token_usage));
                        }
                    }
                }
                Err(e) => {
                    expect_toaster().error(format!("Failed to load messages: {e}"));
                }
            }
            ctx.messages_loading.set(false);
        });
    }

    /// Send a new chat message.
    pub fn send_message(
        &self,
        message: String,
        project_path: Option<String>,
        session_id: Option<String>,
    ) {
        let ctx = *self;
        let provider = ctx.provider.get_untracked();
        let model = ctx.model.get_untracked();
        let permission_mode = ctx.permission_mode.get_untracked();

        // Add user message to the list immediately
        let user_msg = NormalizedMessage {
            id: format!("user_{}", js_sys::Date::now() as u64),
            session_id: session_id.clone().unwrap_or_default(),
            timestamp: js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default(),
            provider,
            kind: "text".to_string(),
            role: Some("user".to_string()),
            content: Some(message.clone()),
            tool_name: None,
            tool_input: None,
            tool_id: None,
            is_error: None,
            request_id: None,
            input: None,
            context: None,
            new_session_id: None,
            exit_code: None,
            aborted: None,
            images: None,
        };
        ctx.messages.update(|msgs| msgs.push(user_msg));
        ctx.is_streaming.set(true);
        ctx.stream_content.set(String::new());

        // Build command JSON
        let mut options = json!({});
        if let Some(path) = project_path {
            options["projectPath"] = json!(path);
        }
        if let Some(sid) = &session_id {
            options["sessionId"] = json!(sid);
            options["resume"] = json!(true);
        }
        if let Some(m) = model {
            options["model"] = json!(m);
        }
        if permission_mode != "default" {
            options["permissionMode"] = json!(permission_mode);
        }

        let command = match provider {
            SessionProvider::Claude => json!({
                "type": "claude-command",
                "command": message,
                "options": options,
            }),
            SessionProvider::Cursor => json!({
                "type": "cursor-command",
                "command": message,
                "sessionId": session_id,
                "options": options,
            }),
            SessionProvider::Codex => json!({
                "type": "codex-command",
                "command": message,
                "options": options,
            }),
            SessionProvider::Gemini => json!({
                "type": "gemini-command",
                "command": message,
                "options": options,
            }),
        };

        spawn_local(async move {
            if let Err(e) = commands::chat_execute(command).await {
                expect_toaster().error(format!("Failed to send message: {e}"));
                ctx.is_streaming.set(false);
            }
        });
    }

    /// Handle an incoming chat_response event.
    pub fn handle_chat_response(&self, resp: ChatResponse) {
        let ctx = *self;
        let kind = resp.kind.as_str();

        // Only process events for the active session (or if we have no active session yet)
        let active_sid = ctx.active_session_id.get_untracked();
        if let Some(ref active) = active_sid {
            if let Some(ref resp_sid) = resp.session_id {
                if resp_sid != active {
                    // Check if this is a new session notification
                    if kind != "session_created" {
                        return;
                    }
                }
            }
        }

        match kind {
            "stream_delta" => {
                if let Some(content) = &resp.content {
                    ctx.stream_content.update(|s| s.push_str(content));
                }
            }
            "session_created" => {
                if let Some(new_id) = &resp.new_session_id {
                    ctx.active_session_id.set(Some(new_id.clone()));
                }
            }
            "complete" => {
                // Finalize the streaming message
                let stream = ctx.stream_content.get_untracked();
                if !stream.is_empty() {
                    let session_id = ctx
                        .active_session_id
                        .get_untracked()
                        .unwrap_or_default();
                    let provider_str = resp.provider.as_str();
                    let provider = match provider_str {
                        "cursor" => SessionProvider::Cursor,
                        "codex" => SessionProvider::Codex,
                        "gemini" => SessionProvider::Gemini,
                        _ => SessionProvider::Claude,
                    };
                    let msg = NormalizedMessage {
                        id: resp.id.clone(),
                        session_id,
                        timestamp: js_sys::Date::new_0()
                            .to_iso_string()
                            .as_string()
                            .unwrap_or_default(),
                        provider,
                        kind: "text".to_string(),
                        role: Some("assistant".to_string()),
                        content: Some(stream),
                        tool_name: None,
                        tool_input: None,
                        tool_id: None,
                        is_error: None,
                        request_id: None,
                        input: None,
                        context: None,
                        new_session_id: None,
                        exit_code: resp.exit_code,
                        aborted: resp.aborted,
                        images: None,
                    };
                    ctx.messages.update(|msgs| msgs.push(msg));
                }
                ctx.stream_content.set(String::new());
                ctx.is_streaming.set(false);
            }
            "error" => {
                let session_id = ctx
                    .active_session_id
                    .get_untracked()
                    .unwrap_or_default();
                let provider_str = resp.provider.as_str();
                let provider = match provider_str {
                    "cursor" => SessionProvider::Cursor,
                    "codex" => SessionProvider::Codex,
                    "gemini" => SessionProvider::Gemini,
                    _ => SessionProvider::Claude,
                };
                let msg = NormalizedMessage {
                    id: resp.id.clone(),
                    session_id,
                    timestamp: js_sys::Date::new_0()
                        .to_iso_string()
                        .as_string()
                        .unwrap_or_default(),
                    provider,
                    kind: "error".to_string(),
                    role: Some("assistant".to_string()),
                    content: resp.content.clone(),
                    tool_name: None,
                    tool_input: None,
                    tool_id: None,
                    is_error: Some(true),
                    request_id: None,
                    input: None,
                    context: None,
                    new_session_id: None,
                    exit_code: None,
                    aborted: None,
                    images: None,
                };
                ctx.messages.update(|msgs| msgs.push(msg));
                ctx.stream_content.set(String::new());
                ctx.is_streaming.set(false);
            }
            "permission_request" => {
                if let (Some(request_id), Some(session_id)) =
                    (&resp.request_id, &resp.session_id)
                {
                    let perm = PermissionRequest {
                        request_id: request_id.clone(),
                        session_id: session_id.clone(),
                        tool_name: resp
                            .tool_name
                            .clone()
                            .unwrap_or_else(|| "Unknown tool".to_string()),
                        tool_input: resp.tool_input.clone(),
                    };
                    ctx.pending_permission.set(Some(perm));
                }
            }
            "tool_use" => {
                let session_id = ctx
                    .active_session_id
                    .get_untracked()
                    .unwrap_or_default();
                let provider_str = resp.provider.as_str();
                let provider = match provider_str {
                    "cursor" => SessionProvider::Cursor,
                    "codex" => SessionProvider::Codex,
                    "gemini" => SessionProvider::Gemini,
                    _ => SessionProvider::Claude,
                };

                // Flush any accumulated stream content first
                let stream = ctx.stream_content.get_untracked();
                if !stream.is_empty() {
                    let text_msg = NormalizedMessage {
                        id: format!("text_{}", js_sys::Date::now() as u64),
                        session_id: session_id.clone(),
                        timestamp: js_sys::Date::new_0()
                            .to_iso_string()
                            .as_string()
                            .unwrap_or_default(),
                        provider,
                        kind: "text".to_string(),
                        role: Some("assistant".to_string()),
                        content: Some(stream),
                        tool_name: None,
                        tool_input: None,
                        tool_id: None,
                        is_error: None,
                        request_id: None,
                        input: None,
                        context: None,
                        new_session_id: None,
                        exit_code: None,
                        aborted: None,
                        images: None,
                    };
                    ctx.messages.update(|msgs| msgs.push(text_msg));
                    ctx.stream_content.set(String::new());
                }

                let msg = NormalizedMessage {
                    id: resp.id.clone(),
                    session_id,
                    timestamp: js_sys::Date::new_0()
                        .to_iso_string()
                        .as_string()
                        .unwrap_or_default(),
                    provider,
                    kind: "tool_use".to_string(),
                    role: Some("assistant".to_string()),
                    content: None,
                    tool_name: resp.tool_name.clone(),
                    tool_input: resp.tool_input.clone(),
                    tool_id: resp.tool_id.clone(),
                    is_error: None,
                    request_id: None,
                    input: None,
                    context: None,
                    new_session_id: None,
                    exit_code: None,
                    aborted: None,
                    images: None,
                };
                ctx.messages.update(|msgs| msgs.push(msg));
            }
            "tool_result" => {
                let session_id = ctx
                    .active_session_id
                    .get_untracked()
                    .unwrap_or_default();
                let provider_str = resp.provider.as_str();
                let provider = match provider_str {
                    "cursor" => SessionProvider::Cursor,
                    "codex" => SessionProvider::Codex,
                    "gemini" => SessionProvider::Gemini,
                    _ => SessionProvider::Claude,
                };
                // Extract content and is_error from tool_result value
                // Backend sends tool_result as either:
                //   - a JSON object: {"content": "...", "isError": false}
                //   - a plain string: "..."
                //   - an array of content blocks
                let (extracted_content, extracted_is_error) =
                    match resp.tool_result.as_ref() {
                        Some(serde_json::Value::String(s)) => {
                            (Some(s.clone()), None)
                        }
                        Some(serde_json::Value::Object(obj)) => {
                            let content = match obj.get("content") {
                                Some(serde_json::Value::String(s)) => Some(s.clone()),
                                Some(serde_json::Value::Array(arr)) => {
                                    // Array of content blocks - extract text parts
                                    let texts: Vec<String> = arr
                                        .iter()
                                        .filter_map(|item| {
                                            item.get("text")
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string())
                                        })
                                        .collect();
                                    if texts.is_empty() {
                                        Some(
                                            serde_json::to_string_pretty(&serde_json::Value::Array(arr.clone()))
                                                .unwrap_or_default(),
                                        )
                                    } else {
                                        Some(texts.join("\n"))
                                    }
                                }
                                Some(inner) => Some(inner.to_string()),
                                None => {
                                    // No "content" field - try "output" or serialize the whole object
                                    match obj.get("output") {
                                        Some(serde_json::Value::String(s)) => Some(s.clone()),
                                        Some(v) => Some(
                                            serde_json::to_string_pretty(v)
                                                .unwrap_or_else(|_| v.to_string()),
                                        ),
                                        None => Some(
                                            serde_json::to_string_pretty(
                                                &serde_json::Value::Object(obj.clone()),
                                            )
                                            .unwrap_or_default(),
                                        ),
                                    }
                                }
                            };
                            let is_error = obj
                                .get("is_error")
                                .or(obj.get("isError"))
                                .and_then(|v| v.as_bool());
                            (content, is_error)
                        }
                        Some(other) => (
                            Some(
                                serde_json::to_string_pretty(other)
                                    .unwrap_or_else(|_| other.to_string()),
                            ),
                            None,
                        ),
                        None => (resp.content.clone(), None),
                    };

                let msg = NormalizedMessage {
                    id: resp.id.clone(),
                    session_id,
                    timestamp: js_sys::Date::new_0()
                        .to_iso_string()
                        .as_string()
                        .unwrap_or_default(),
                    provider,
                    kind: "tool_result".to_string(),
                    role: Some("assistant".to_string()),
                    content: extracted_content,
                    tool_name: resp.tool_name.clone(),
                    tool_input: None,
                    tool_id: resp.tool_id.clone(),
                    is_error: extracted_is_error,
                    request_id: None,
                    input: None,
                    context: None,
                    new_session_id: None,
                    exit_code: None,
                    aborted: None,
                    images: None,
                };
                ctx.messages.update(|msgs| msgs.push(msg));
            }
            "thinking" => {
                if let Some(content) = resp.content.clone() {
                    if !content.is_empty() {
                        let session_id = ctx
                            .active_session_id
                            .get_untracked()
                            .unwrap_or_default();
                        let provider_str = resp.provider.as_str();
                        let provider = match provider_str {
                            "cursor" => SessionProvider::Cursor,
                            "codex" => SessionProvider::Codex,
                            "gemini" => SessionProvider::Gemini,
                            _ => SessionProvider::Claude,
                        };

                        // Flush any accumulated stream content first
                        let stream = ctx.stream_content.get_untracked();
                        if !stream.is_empty() {
                            let text_msg = NormalizedMessage {
                                id: format!("text_{}", js_sys::Date::now() as u64),
                                session_id: session_id.clone(),
                                timestamp: js_sys::Date::new_0()
                                    .to_iso_string()
                                    .as_string()
                                    .unwrap_or_default(),
                                provider,
                                kind: "text".to_string(),
                                role: Some("assistant".to_string()),
                                content: Some(stream),
                                tool_name: None,
                                tool_input: None,
                                tool_id: None,
                                is_error: None,
                                request_id: None,
                                input: None,
                                context: None,
                                new_session_id: None,
                                exit_code: None,
                                aborted: None,
                                images: None,
                            };
                            ctx.messages.update(|msgs| msgs.push(text_msg));
                            ctx.stream_content.set(String::new());
                        }

                        let msg = NormalizedMessage {
                            id: format!("thinking_{}", js_sys::Date::now() as u64),
                            session_id,
                            timestamp: js_sys::Date::new_0()
                                .to_iso_string()
                                .as_string()
                                .unwrap_or_default(),
                            provider,
                            kind: "thinking".to_string(),
                            role: Some("assistant".to_string()),
                            content: Some(content),
                            tool_name: None,
                            tool_input: None,
                            tool_id: None,
                            is_error: None,
                            request_id: None,
                            input: None,
                            context: None,
                            new_session_id: None,
                            exit_code: None,
                            aborted: None,
                            images: None,
                        };
                        ctx.messages.update(|msgs| msgs.push(msg));
                    }
                }
            }
            _ => {
                // Other event types (thinking, status, etc.) - log but don't add to messages
            }
        }
    }

    /// Respond to a pending permission request.
    pub fn respond_permission(&self, allow: bool) {
        let ctx = *self;
        let perm = ctx.pending_permission.get_untracked();
        if let Some(perm) = perm {
            ctx.pending_permission.set(None);

            let command = json!({
                "type": "claude-permission-response",
                "requestId": perm.request_id,
                "allow": allow,
            });

            spawn_local(async move {
                if let Err(e) = commands::chat_execute(command).await {
                    expect_toaster().error(format!("Failed to respond: {e}"));
                }
            });
        }
    }

    /// Abort the current streaming session.
    pub fn abort_session(&self) {
        let ctx = *self;
        let session_id = ctx.active_session_id.get_untracked();
        let provider = ctx.provider.get_untracked();

        if let Some(session_id) = session_id {
            let command = json!({
                "type": "abort-session",
                "sessionId": session_id,
                "provider": provider.to_string(),
            });

            spawn_local(async move {
                if let Err(e) = commands::chat_execute(command).await {
                    expect_toaster().error(format!("Failed to abort: {e}"));
                }
                ctx.is_streaming.set(false);
                ctx.stream_content.set(String::new());
            });
        }
    }

    /// Clear chat state (e.g., when switching projects).
    pub fn clear(&self) {
        self.messages.set(Vec::new());
        self.is_streaming.set(false);
        self.pending_permission.set(None);
        self.active_session_id.set(None);
        self.stream_content.set(String::new());
        self.token_usage.set(None);
    }
}
