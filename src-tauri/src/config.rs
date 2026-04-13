use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_path: PathBuf,
    pub workspaces_root: PathBuf,
    pub context_window: u32,
}

impl AppConfig {
    /// Build config using Tauri's app data directory for DB storage.
    pub fn new(app_data_dir: PathBuf) -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        let database_path = std::env::var("DATABASE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| app_data_dir.join("rccui.db"));

        let workspaces_root = std::env::var("WORKSPACES_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home);

        let context_window = std::env::var("CONTEXT_WINDOW")
            .unwrap_or_else(|_| "160000".to_string())
            .parse()
            .unwrap_or(160000);

        tracing::info!(
            ?database_path,
            ?workspaces_root,
            context_window,
            "Configuration loaded"
        );

        Self {
            database_path,
            workspaces_root,
            context_window,
        }
    }
}
