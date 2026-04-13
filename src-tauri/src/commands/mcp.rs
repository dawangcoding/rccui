use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;

use crate::error::AppError;

// ─── Claude MCP CLI ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn mcp_cli_list() -> Result<Value, String> {
    _mcp_cli_list().await.map_err(|e| e.to_string())
}

async fn _mcp_cli_list() -> Result<Value, AppError> {
    let output = Command::new("claude")
        .args(["mcp", "list", "--json"])
        .output()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to run claude: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let servers: Value = serde_json::from_str(&stdout).unwrap_or(Value::Array(vec![]));

    Ok(json!({
        "success": true,
        "output": stdout.trim(),
        "servers": servers
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMcpServerRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub server_type: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<Value>,
    pub env: Option<Value>,
    pub scope: Option<String>,
    pub project_path: Option<String>,
}

#[tauri::command]
pub async fn mcp_cli_add(
    body: AddMcpServerRequest,
) -> Result<Value, String> {
    _mcp_cli_add(body).await.map_err(|e| e.to_string())
}

async fn _mcp_cli_add(body: AddMcpServerRequest) -> Result<Value, AppError> {
    if body.name.is_empty() {
        return Err(AppError::BadRequest("Server name is required".into()));
    }

    let scope = body.scope.as_deref().unwrap_or("user");
    let server_type = body.server_type.as_deref().unwrap_or("stdio");

    let mut args = vec![
        "mcp".to_string(),
        "add".to_string(),
        "--scope".to_string(),
        scope.to_string(),
        "--type".to_string(),
        server_type.to_string(),
        body.name.clone(),
    ];

    // For stdio: command + args
    if let Some(ref cmd) = body.command {
        args.push(cmd.clone());
    }
    if let Some(ref server_args) = body.args {
        for a in server_args {
            args.push(a.clone());
        }
    }

    // For http/sse: url
    if let Some(ref url) = body.url {
        args.push(url.clone());
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let output = Command::new("claude")
        .args(&args_refs)
        .output()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to run claude: {e}")))?;

    if output.status.success() {
        Ok(json!({
            "success": true,
            "message": format!("Server '{}' added successfully", body.name)
        }))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::Internal(anyhow::anyhow!(
            "claude mcp add failed: {stderr}"
        )))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMcpJsonRequest {
    pub name: String,
    pub json_config: Value,
    pub scope: Option<String>,
    pub project_path: Option<String>,
}

#[tauri::command]
pub async fn mcp_cli_add_json(
    body: AddMcpJsonRequest,
) -> Result<Value, String> {
    _mcp_cli_add_json(body).await.map_err(|e| e.to_string())
}

async fn _mcp_cli_add_json(body: AddMcpJsonRequest) -> Result<Value, AppError> {
    if body.name.is_empty() {
        return Err(AppError::BadRequest("Server name is required".into()));
    }

    let scope = body.scope.as_deref().unwrap_or("user");

    // Write config directly to the Claude config file
    let home = dirs::home_dir().unwrap_or_default();
    let config_path = home.join(".claude.json");

    let mut config: Value = if config_path.exists() {
        let data = tokio::fs::read_to_string(&config_path)
            .await
            .unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&data).unwrap_or(json!({}))
    } else {
        json!({})
    };

    let servers_key = if scope == "local" {
        "localMcpServers"
    } else {
        "mcpServers"
    };
    if config.get(servers_key).is_none() {
        config[servers_key] = json!({});
    }
    config[servers_key][&body.name] = body.json_config.clone();

    let data = serde_json::to_string_pretty(&config).unwrap_or_default();
    tokio::fs::write(&config_path, data)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(json!({
        "success": true,
        "config": body.json_config
    }))
}

#[tauri::command]
pub async fn mcp_cli_remove(
    name: String,
    scope: Option<String>,
) -> Result<Value, String> {
    _mcp_cli_remove(&name, scope.as_deref())
        .await
        .map_err(|e| e.to_string())
}

async fn _mcp_cli_remove(name: &str, scope: Option<&str>) -> Result<Value, AppError> {
    let scope = scope.unwrap_or("user");

    let output = Command::new("claude")
        .args(["mcp", "remove", "--scope", scope, name])
        .output()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to run claude: {e}")))?;

    if output.status.success() {
        Ok(json!({
            "success": true,
            "message": format!("Server '{name}' removed")
        }))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::Internal(anyhow::anyhow!(
            "claude mcp remove failed: {stderr}"
        )))
    }
}

#[tauri::command]
pub async fn mcp_cli_get(
    name: String,
) -> Result<Value, String> {
    _mcp_cli_get(&name).await.map_err(|e| e.to_string())
}

async fn _mcp_cli_get(name: &str) -> Result<Value, AppError> {
    let output = Command::new("claude")
        .args(["mcp", "get", "--json", name])
        .output()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to run claude: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let server: Value = serde_json::from_str(&stdout).unwrap_or(Value::Null);

    Ok(json!({
        "success": true,
        "output": stdout.trim(),
        "server": server
    }))
}

// ─── Claude Config Direct Read ───────────────────────────────────────────────

#[tauri::command]
pub async fn mcp_config_read() -> Result<Value, String> {
    _mcp_config_read().await.map_err(|e| e.to_string())
}

async fn _mcp_config_read() -> Result<Value, AppError> {
    let home = dirs::home_dir().unwrap_or_default();

    // Try multiple config paths
    let config_paths = vec![
        home.join(".claude.json"),
        home.join(".claude").join("settings.json"),
    ];

    let mut servers = json!({});
    let mut config_path_used = String::new();

    for path in config_paths {
        if path.exists() {
            if let Ok(data) = tokio::fs::read_to_string(&path).await {
                if let Ok(config) = serde_json::from_str::<Value>(&data) {
                    if let Some(mcp) = config.get("mcpServers") {
                        servers = mcp.clone();
                        config_path_used = path.to_string_lossy().to_string();
                        break;
                    }
                }
            }
        }
    }

    Ok(json!({
        "success": true,
        "servers": servers,
        "configPath": config_path_used
    }))
}

// ─── Cursor MCP ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn cursor_mcp_list() -> Result<Value, String> {
    _cursor_mcp_list().await.map_err(|e| e.to_string())
}

async fn _cursor_mcp_list() -> Result<Value, AppError> {
    let home = dirs::home_dir().unwrap_or_default();
    let config_path = home.join(".cursor").join("mcp.json");

    let (servers, path_str) = if config_path.exists() {
        let data = tokio::fs::read_to_string(&config_path)
            .await
            .unwrap_or_else(|_| "{}".to_string());
        let config: Value = serde_json::from_str(&data).unwrap_or(json!({}));
        let servers = config.get("mcpServers").cloned().unwrap_or(json!({}));
        (servers, config_path.to_string_lossy().to_string())
    } else {
        (json!({}), String::new())
    };

    Ok(json!({
        "success": true,
        "servers": servers,
        "path": path_str
    }))
}

#[tauri::command]
pub async fn cursor_mcp_add(
    body: AddMcpServerRequest,
) -> Result<Value, String> {
    _cursor_mcp_add(body).await.map_err(|e| e.to_string())
}

async fn _cursor_mcp_add(body: AddMcpServerRequest) -> Result<Value, AppError> {
    if body.name.is_empty() {
        return Err(AppError::BadRequest("Server name is required".into()));
    }

    let home = dirs::home_dir().unwrap_or_default();
    let config_path = home.join(".cursor").join("mcp.json");

    let mut config: Value = if config_path.exists() {
        let data = tokio::fs::read_to_string(&config_path)
            .await
            .unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&data).unwrap_or(json!({}))
    } else {
        json!({})
    };

    if config.get("mcpServers").is_none() {
        config["mcpServers"] = json!({});
    }

    let server_type = body.server_type.as_deref().unwrap_or("stdio");
    let mut server_config = json!({ "type": server_type });

    if let Some(ref cmd) = body.command {
        server_config["command"] = json!(cmd);
    }
    if let Some(ref args) = body.args {
        server_config["args"] = json!(args);
    }
    if let Some(ref url) = body.url {
        server_config["url"] = json!(url);
    }
    if let Some(ref env) = body.env {
        server_config["env"] = env.clone();
    }

    config["mcpServers"][&body.name] = server_config;

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }

    let data = serde_json::to_string_pretty(&config).unwrap_or_default();
    tokio::fs::write(&config_path, data)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(json!({
        "success": true,
        "message": format!("Server '{}' added to Cursor config", body.name),
        "config": config
    }))
}

#[tauri::command]
pub async fn cursor_mcp_add_json(
    body: AddMcpJsonRequest,
) -> Result<Value, String> {
    _cursor_mcp_add_json(body).await.map_err(|e| e.to_string())
}

async fn _cursor_mcp_add_json(body: AddMcpJsonRequest) -> Result<Value, AppError> {
    if body.name.is_empty() {
        return Err(AppError::BadRequest("Server name is required".into()));
    }

    let home = dirs::home_dir().unwrap_or_default();
    let config_path = home.join(".cursor").join("mcp.json");

    let mut config: Value = if config_path.exists() {
        let data = tokio::fs::read_to_string(&config_path)
            .await
            .unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&data).unwrap_or(json!({}))
    } else {
        json!({})
    };

    if config.get("mcpServers").is_none() {
        config["mcpServers"] = json!({});
    }
    config["mcpServers"][&body.name] = body.json_config.clone();

    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }

    let data = serde_json::to_string_pretty(&config).unwrap_or_default();
    tokio::fs::write(&config_path, data)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(json!({
        "success": true,
        "config": body.json_config
    }))
}

#[tauri::command]
pub async fn cursor_mcp_remove(
    name: String,
) -> Result<Value, String> {
    _cursor_mcp_remove(&name).await.map_err(|e| e.to_string())
}

async fn _cursor_mcp_remove(name: &str) -> Result<Value, AppError> {
    let home = dirs::home_dir().unwrap_or_default();
    let config_path = home.join(".cursor").join("mcp.json");

    if !config_path.exists() {
        return Err(AppError::NotFound("Cursor MCP config not found".into()));
    }

    let data = tokio::fs::read_to_string(&config_path)
        .await
        .unwrap_or_else(|_| "{}".to_string());
    let mut config: Value = serde_json::from_str(&data).unwrap_or(json!({}));

    if let Some(servers) = config
        .get_mut("mcpServers")
        .and_then(|v| v.as_object_mut())
    {
        servers.remove(name);
    }

    let data = serde_json::to_string_pretty(&config).unwrap_or_default();
    tokio::fs::write(&config_path, data)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(json!({
        "success": true,
        "message": format!("Server '{name}' removed from Cursor config")
    }))
}

// ─── MCP Utilities ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn mcp_all_servers() -> Result<Value, String> {
    _mcp_all_servers().await.map_err(|e| e.to_string())
}

async fn _mcp_all_servers() -> Result<Value, AppError> {
    let home = dirs::home_dir().unwrap_or_default();

    // Claude user config
    let user_servers = read_mcp_servers(&home.join(".claude.json"), "mcpServers").await;
    let local_servers = read_mcp_servers(&home.join(".claude.json"), "localMcpServers").await;

    // Cursor config
    let cursor_servers =
        read_mcp_servers(&home.join(".cursor").join("mcp.json"), "mcpServers").await;

    // Codex config
    let codex_servers =
        read_mcp_servers(&home.join(".codex").join("config.json"), "mcpServers").await;

    // Collect all server names
    let mut all = Vec::new();
    if let Some(obj) = user_servers.as_object() {
        for (name, config) in obj {
            all.push(json!({ "name": name, "config": config, "source": "claude-user" }));
        }
    }
    if let Some(obj) = local_servers.as_object() {
        for (name, config) in obj {
            all.push(json!({ "name": name, "config": config, "source": "claude-local" }));
        }
    }
    if let Some(obj) = cursor_servers.as_object() {
        for (name, config) in obj {
            all.push(json!({ "name": name, "config": config, "source": "cursor" }));
        }
    }
    if let Some(obj) = codex_servers.as_object() {
        for (name, config) in obj {
            all.push(json!({ "name": name, "config": config, "source": "codex" }));
        }
    }

    Ok(json!({
        "servers": all,
        "userServers": user_servers,
        "localServers": local_servers,
        "cursorServers": cursor_servers,
        "codexServers": codex_servers
    }))
}

async fn read_mcp_servers(path: &std::path::Path, key: &str) -> Value {
    if !path.exists() {
        return json!({});
    }
    let data = tokio::fs::read_to_string(path)
        .await
        .unwrap_or_else(|_| "{}".to_string());
    let config: Value = serde_json::from_str(&data).unwrap_or(json!({}));
    config.get(key).cloned().unwrap_or(json!({}))
}
