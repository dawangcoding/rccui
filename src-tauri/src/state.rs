use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::sync::Arc;

use dashmap::DashMap;
use portable_pty::MasterPty;
use sqlx::SqlitePool;
use tokio::sync::{Mutex, RwLock, mpsc};

use crate::config::AppConfig;

/// Tracks an active CLI session.
pub struct ActiveSession {
    pub child: tokio::process::Child,
    pub abort_tx: tokio::sync::oneshot::Sender<()>,
    /// Channel for writing to the CLI process's stdin (e.g. permission responses).
    pub stdin_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
}

/// Maximum number of entries in the PTY circular replay buffer.
pub const PTY_BUFFER_CAP: usize = 5000;

/// PTY session for interactive shell WebSocket.
pub struct PtySession {
    /// PTY master handle — used for resize operations.
    pub master: Box<dyn MasterPty + Send>,
    /// PTY master writer — used to send input to the child process.
    pub writer: Box<dyn Write + Send>,
    /// Child process handle.
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    /// Project path this session was spawned in.
    pub project_path: String,
    /// Session ID (may be empty for plain-shell).
    pub session_id: String,
    /// Provider name (claude, cursor, codex, gemini, plain-shell).
    pub provider: String,
    /// Circular replay buffer for reconnection.
    pub buffer: VecDeque<String>,
    /// Background task that reads from PTY stdout and forwards output.
    pub reader_handle: Option<tokio::task::JoinHandle<()>>,
    /// Delayed cleanup task (spawned on WebSocket disconnect, cancelled on reconnect).
    pub cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Default local user ID (desktop app has no auth, single user).
pub const LOCAL_USER_ID: i64 = 1;

/// Central application state shared across all handlers.
pub struct AppState {
    /// Database connection pool
    pub db: SqlitePool,

    /// Application configuration
    pub config: AppConfig,

    /// Local user ID (always 1 for desktop app)
    pub local_user_id: i64,

    /// Tauri app handle for emitting events from background tasks
    pub app_handle: tauri::AppHandle,

    /// Active CLI sessions per provider: key = "provider:session_id"
    pub active_sessions: DashMap<String, Arc<Mutex<ActiveSession>>>,

    /// PTY sessions for shell reuse: key = session_key
    pub pty_sessions: Arc<Mutex<HashMap<String, PtySession>>>,

    /// Cached project list
    pub project_cache: Arc<RwLock<Option<Vec<serde_json::Value>>>>,
}

impl AppState {
    pub fn new(db: SqlitePool, config: AppConfig, app_handle: tauri::AppHandle) -> Self {
        Self {
            db,
            config,
            local_user_id: LOCAL_USER_ID,
            app_handle,
            active_sessions: DashMap::new(),
            pty_sessions: Arc::new(Mutex::new(HashMap::new())),
            project_cache: Arc::new(RwLock::new(None)),
        }
    }
}
