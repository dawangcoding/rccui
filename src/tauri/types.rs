use serde::{Deserialize, Serialize};
use serde_json::Value;

// ─── Provider ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SessionProvider {
    #[default]
    Claude,
    Cursor,
    Codex,
    Gemini,
}

impl std::fmt::Display for SessionProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionProvider::Claude => write!(f, "claude"),
            SessionProvider::Cursor => write!(f, "cursor"),
            SessionProvider::Codex => write!(f, "codex"),
            SessionProvider::Gemini => write!(f, "gemini"),
        }
    }
}

// ─── Project ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    pub path: String,
    pub display_name: String,
    pub full_path: String,
    pub is_custom_name: bool,
    #[serde(default)]
    pub is_manually_added: Option<bool>,
    #[serde(default)]
    pub sessions: Vec<SessionInfo>,
    #[serde(default)]
    pub cursor_sessions: Vec<SessionInfo>,
    #[serde(default)]
    pub codex_sessions: Vec<SessionInfo>,
    #[serde(default)]
    pub gemini_sessions: Vec<SessionInfo>,
    #[serde(default)]
    pub session_meta: SessionMeta,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: String,
    pub summary: String,
    pub message_count: usize,
    pub last_activity: String,
    #[serde(default)]
    pub cwd: Option<String>,
    pub provider: String,
    #[serde(default)]
    pub is_grouped: Option<bool>,
    #[serde(default)]
    pub group_size: Option<usize>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMeta {
    pub has_more: bool,
    pub total: usize,
}

// ─── Session List Response ───────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionListResponse {
    pub sessions: Vec<SessionInfo>,
    pub total: usize,
    pub has_more: bool,
}

// ─── Chat ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub provider: String,
    #[serde(default)]
    pub new_session_id: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub is_new_session: Option<bool>,
    #[serde(default)]
    pub aborted: Option<bool>,
    // Tool use
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<Value>,
    #[serde(default)]
    pub tool_result: Option<Value>,
    #[serde(default)]
    pub tool_id: Option<String>,
    // Permission
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub extra: Option<Value>,
}

// ─── NormalizedMessage ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedMessage {
    pub id: String,
    pub session_id: String,
    pub timestamp: String,
    pub provider: SessionProvider,
    pub kind: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<Value>,
    #[serde(default)]
    pub tool_id: Option<String>,
    #[serde(default)]
    pub is_error: Option<bool>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub input: Option<Value>,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub new_session_id: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub aborted: Option<bool>,
    #[serde(default)]
    pub images: Option<Vec<ImageData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageData {
    pub name: String,
    pub data: String,
    pub mime_type: String,
}

// ─── Fetch History Result ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchHistoryResult {
    pub messages: Vec<NormalizedMessage>,
    pub total: usize,
    pub has_more: bool,
    pub offset: u32,
    pub limit: Option<u32>,
    #[serde(default)]
    pub token_usage: Option<Value>,
}

// ─── Permission Request (frontend state) ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequest {
    pub request_id: String,
    pub session_id: String,
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: Option<Value>,
}

// ─── Token Usage ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub used: u64,
    pub total: u64,
    #[serde(default)]
    pub unsupported: Option<bool>,
    #[serde(default)]
    pub breakdown: Option<Value>,
}

// ─── Shell ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellInitParams {
    pub project_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default)]
    pub has_session: bool,
    pub provider: String,
    pub cols: u16,
    pub rows: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_command: Option<String>,
    #[serde(default)]
    pub is_plain_shell: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellInitResult {
    pub session_key: String,
    pub buffer: Vec<String>,
    pub reconnected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellOutputPayload {
    pub session_key: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellAuthUrlPayload {
    pub session_key: String,
    pub url: String,
}

// ─── Projects Updated Event ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsUpdatedPayload {
    pub change_type: String,
    #[serde(default)]
    pub changed_file: Option<String>,
    #[serde(default)]
    pub watch_provider: Option<String>,
    pub projects: Vec<Value>,
}

// ─── Onboarding ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingStatus {
    pub success: bool,
    pub has_completed_onboarding: bool,
}

// ─── App Tab ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppTab {
    #[default]
    Chat,
    Shell,
    Files,
    Git,
}

impl AppTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppTab::Chat => "chat",
            AppTab::Shell => "shell",
            AppTab::Files => "files",
            AppTab::Git => "git",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "shell" => AppTab::Shell,
            "files" => AppTab::Files,
            "git" => AppTab::Git,
            _ => AppTab::Chat,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AppTab::Chat => "Chat",
            AppTab::Shell => "Shell",
            AppTab::Files => "Files",
            AppTab::Git => "Git",
        }
    }

    pub fn label_key(&self) -> &'static str {
        match self {
            AppTab::Chat => "tab-chat",
            AppTab::Shell => "tab-shell",
            AppTab::Files => "tab-files",
            AppTab::Git => "tab-git",
        }
    }

    pub fn is_implemented(&self) -> bool {
        matches!(self, AppTab::Chat | AppTab::Shell)
    }
}
