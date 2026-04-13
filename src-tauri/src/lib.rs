mod commands;
mod config;
mod db;
mod error;
mod providers;
mod services;
mod state;

use std::sync::Arc;

use config::AppConfig;
use state::AppState;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Determine app data directory for DB storage
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            // Build configuration
            let config = AppConfig::new(app_data_dir);

            // Initialize database (block on async)
            let pool = tauri::async_runtime::block_on(async {
                db::init_pool(&config.database_path).await
            })
            .expect("failed to initialize database");

            tracing::info!("Database initialized at {:?}", config.database_path);

            // Auto-create default local user if none exists
            tauri::async_runtime::block_on(async {
                if !db::users::has_users(&pool).await.unwrap_or(false) {
                    tracing::info!("Creating default local user");
                    let _ = db::users::create_user(&pool, "local", "").await;
                }
            });

            // Build application state
            let app_state = Arc::new(AppState::new(pool, config, app.handle().clone()));

            // Start background file watcher for session changes
            services::file_watcher::start_file_watcher(app_state.clone());

            app.manage(app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Git commands
            commands::git::git_status,
            commands::git::git_diff,
            commands::git::git_file_with_diff,
            commands::git::git_initial_commit,
            commands::git::git_commit,
            commands::git::git_revert_local_commit,
            commands::git::git_branches,
            commands::git::git_checkout,
            commands::git::git_create_branch,
            commands::git::git_delete_branch,
            commands::git::git_commits,
            commands::git::git_commit_diff,
            commands::git::git_remote_status,
            commands::git::git_fetch,
            commands::git::git_pull,
            commands::git::git_push,
            commands::git::git_publish,
            commands::git::git_discard,
            commands::git::git_delete_untracked,
            // User commands
            commands::user::get_git_config,
            commands::user::update_git_config,
            commands::user::complete_onboarding,
            commands::user::get_onboarding_status,
            // Settings commands
            commands::settings::list_api_keys,
            commands::settings::create_api_key,
            commands::settings::delete_api_key,
            commands::settings::toggle_api_key,
            commands::settings::list_credentials,
            commands::settings::create_credential,
            commands::settings::delete_credential,
            commands::settings::toggle_credential,
            commands::settings::get_notification_preferences,
            commands::settings::update_notification_preferences,
            commands::settings::get_vapid_public_key,
            commands::settings::push_subscribe,
            commands::settings::push_unsubscribe,
            // Project commands
            commands::projects::list_projects,
            commands::projects::add_project,
            commands::projects::create_workspace,
            commands::projects::rename_project,
            commands::projects::delete_project,
            commands::projects::list_files,
            commands::projects::create_file,
            commands::projects::rename_file,
            commands::projects::delete_file,
            commands::projects::upload_files,
            commands::projects::upload_images,
            commands::projects::save_file,
            commands::projects::read_file,
            commands::projects::read_file_content,
            commands::projects::read_raw_file,
            // Session commands
            commands::sessions::list_sessions,
            commands::sessions::get_session_messages,
            commands::sessions::delete_session,
            commands::sessions::delete_project_session,
            commands::sessions::delete_codex_session,
            commands::sessions::delete_gemini_session,
            commands::sessions::rename_session,
            commands::sessions::set_session_name,
            commands::sessions::delete_session_name,
            commands::sessions::get_session_token_usage,
            // Commands handler
            commands::commands_handler::list_commands,
            commands::commands_handler::load_command,
            commands::commands_handler::execute_command,
            // MCP commands
            commands::mcp::mcp_cli_list,
            commands::mcp::mcp_cli_add,
            commands::mcp::mcp_cli_add_json,
            commands::mcp::mcp_cli_remove,
            commands::mcp::mcp_cli_get,
            commands::mcp::mcp_config_read,
            commands::mcp::cursor_mcp_list,
            commands::mcp::cursor_mcp_add,
            commands::mcp::cursor_mcp_add_json,
            commands::mcp::cursor_mcp_remove,
            commands::mcp::mcp_all_servers,
            // Chat commands
            commands::chat::chat_execute,
            // Shell commands
            commands::shell::shell_init,
            commands::shell::shell_input,
            commands::shell::shell_resize,
            commands::shell::shell_detach,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
