use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;

use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use crate::error::AppError;
use crate::services::project_scanner;
use crate::state::AppState;

// ─── Project Management ──────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ProjectsQuery {
    pub refresh: Option<bool>,
}

#[tauri::command]
pub async fn list_projects(
    state: tauri::State<'_, Arc<AppState>>,
    refresh: Option<bool>,
) -> Result<Value, String> {
    _list_projects(&state, refresh.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())
}

async fn _list_projects(state: &AppState, refresh: bool) -> Result<Value, AppError> {
    // Check cache first
    if !refresh {
        let cache = state.project_cache.read().await;
        if let Some(ref cached) = *cache {
            debug!(count = cached.len(), "Projects cache hit");
            return Ok(Value::Array(cached.clone()));
        }
    }

    debug!(refresh, "Scanning projects");

    // Discover projects
    let projects = project_scanner::get_projects(&state.db, None).await;
    let projects_json: Vec<Value> = projects
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect();

    debug!(count = projects_json.len(), "Projects discovered");

    // Update cache
    {
        let mut cache = state.project_cache.write().await;
        *cache = Some(projects_json.clone());
    }

    Ok(Value::Array(projects_json))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProjectRequest {
    pub project_path: String,
}

#[tauri::command]
pub async fn add_project(
    state: tauri::State<'_, Arc<AppState>>,
    body: AddProjectRequest,
) -> Result<Value, String> {
    _add_project(&state, body).await.map_err(|e| e.to_string())
}

async fn _add_project(state: &AppState, body: AddProjectRequest) -> Result<Value, AppError> {
    info!(path = %body.project_path, "Adding project manually");

    let project_name = project_scanner::add_project_manually(&body.project_path).await?;

    // Invalidate cache
    {
        let mut cache = state.project_cache.write().await;
        *cache = None;
    }

    info!(%project_name, "Project added");

    Ok(json!({
        "success": true,
        "projectName": project_name
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRequest {
    pub workspace_type: String,
    pub path: String,
}

#[tauri::command]
pub async fn create_workspace(
    state: tauri::State<'_, Arc<AppState>>,
    body: CreateWorkspaceRequest,
) -> Result<Value, String> {
    _create_workspace(&state, body)
        .await
        .map_err(|e| e.to_string())
}

async fn _create_workspace(state: &AppState, body: CreateWorkspaceRequest) -> Result<Value, AppError> {
    let workspace_path = body.path.trim().to_string();
    if workspace_path.is_empty() {
        return Err(AppError::BadRequest("Path is required".to_string()));
    }

    // Expand ~ to home directory
    let expanded_path = if workspace_path.starts_with("~/") {
        let home = dirs::home_dir().unwrap_or_default();
        home.join(&workspace_path[2..]).to_string_lossy().to_string()
    } else {
        workspace_path.clone()
    };

    let path = std::path::PathBuf::from(&expanded_path);

    // If workspace type is "new", create the directory
    if body.workspace_type == "new" {
        if !path.exists() {
            tokio::fs::create_dir_all(&path).await.map_err(|e| {
                AppError::BadRequest(format!("Failed to create directory: {e}"))
            })?;
            info!(path = %expanded_path, "Created new workspace directory");
        }
    } else if !path.exists() {
        return Err(AppError::BadRequest(format!(
            "Directory does not exist: {expanded_path}"
        )));
    }

    let project_name = project_scanner::add_project_manually(&expanded_path).await?;

    // Invalidate cache
    {
        let mut cache = state.project_cache.write().await;
        *cache = None;
    }

    info!(%project_name, path = %expanded_path, "Workspace created and project added");

    Ok(json!({
        "success": true,
        "project": {
            "name": project_name,
            "path": expanded_path
        }
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameProjectRequest {
    pub display_name: String,
}

#[tauri::command]
pub async fn rename_project(
    state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    body: RenameProjectRequest,
) -> Result<Value, String> {
    _rename_project(&state, &project_name, body)
        .await
        .map_err(|e| e.to_string())
}

async fn _rename_project(
    state: &AppState,
    project_name: &str,
    body: RenameProjectRequest,
) -> Result<Value, AppError> {
    info!(%project_name, display_name = %body.display_name, "Renaming project");

    project_scanner::rename_project(project_name, &body.display_name).await?;

    // Invalidate cache
    {
        let mut cache = state.project_cache.write().await;
        *cache = None;
    }

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn delete_project(
    state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
) -> Result<Value, String> {
    _delete_project(&state, &project_name)
        .await
        .map_err(|e| e.to_string())
}

async fn _delete_project(state: &AppState, project_name: &str) -> Result<Value, AppError> {
    info!(%project_name, "Deleting project");

    project_scanner::delete_project(project_name).await?;

    // Invalidate cache
    {
        let mut cache = state.project_cache.write().await;
        *cache = None;
    }

    Ok(json!({ "success": true }))
}

// ─── File Tree ───────────────────────────────────────────────────────────────

/// Directories to skip when building the file tree.
const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    "dist",
    "build",
    ".git",
    ".svn",
    ".hg",
    "__pycache__",
    ".DS_Store",
    "target",
];

const MAX_DEPTH: usize = 10;

/// Convert Unix permission mode to rwx string (e.g. "rwxr-xr-x").
fn mode_to_rwx(mode: u32) -> String {
    let mut s = String::with_capacity(9);
    for shift in [6, 3, 0] {
        let bits = (mode >> shift) & 7;
        s.push(if bits & 4 != 0 { 'r' } else { '-' });
        s.push(if bits & 2 != 0 { 'w' } else { '-' });
        s.push(if bits & 1 != 0 { 'x' } else { '-' });
    }
    s
}

/// Recursively build a file tree for the given directory.
async fn build_file_tree(dir_path: &std::path::Path, depth: usize) -> Vec<Value> {
    let mut items = Vec::new();

    let mut entries = match tokio::fs::read_dir(dir_path).await {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::PermissionDenied {
                debug!("Error reading directory {}: {e}", dir_path.display());
            }
            return items;
        }
    };

    let mut collected = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        collected.push(entry);
    }

    for entry in collected {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip excluded directories/files
        if EXCLUDED_DIRS.contains(&name.as_str()) {
            continue;
        }

        let item_path = entry.path();
        let is_dir = entry
            .file_type()
            .await
            .map(|ft| ft.is_dir())
            .unwrap_or(false);

        // Gather stats
        let (size, modified, permissions_rwx) = match tokio::fs::metadata(&item_path).await {
            Ok(meta) => {
                let size = meta.len();
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| {
                        let dt: chrono::DateTime<chrono::Utc> = t.into();
                        Some(dt.to_rfc3339())
                    })
                    .unwrap_or_default();
                let mode = meta.permissions().mode();
                let rwx = mode_to_rwx(mode & 0o777);
                (size, modified, rwx)
            }
            Err(_) => (0, String::new(), "---------".to_string()),
        };

        let mut item = json!({
            "name": name,
            "path": item_path.to_string_lossy(),
            "type": if is_dir { "directory" } else { "file" },
            "size": size,
            "modified": modified,
            "permissionsRwx": permissions_rwx,
        });

        // Recurse into subdirectories
        if is_dir && depth < MAX_DEPTH {
            match tokio::fs::read_dir(&item_path).await {
                Ok(_) => {
                    let children =
                        Box::pin(build_file_tree(&item_path, depth + 1)).await;
                    item["children"] = Value::Array(children);
                }
                Err(_) => {
                    item["children"] = Value::Array(vec![]);
                }
            }
        }

        items.push(item);
    }

    // Sort: directories first, then alphabetical by name
    items.sort_by(|a, b| {
        let a_type = a["type"].as_str().unwrap_or("");
        let b_type = b["type"].as_str().unwrap_or("");
        if a_type != b_type {
            if a_type == "directory" {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        } else {
            let a_name = a["name"].as_str().unwrap_or("").to_lowercase();
            let b_name = b["name"].as_str().unwrap_or("").to_lowercase();
            a_name.cmp(&b_name)
        }
    });

    items
}

#[tauri::command]
pub async fn list_files(
    _state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
) -> Result<Value, String> {
    _list_files(&project_name).await.map_err(|e| e.to_string())
}

async fn _list_files(project_name: &str) -> Result<Value, AppError> {
    let project_root = resolve_project_root(project_name).await?;

    debug!(project = %project_name, path = %project_root.display(), "Listing project files");

    let files = build_file_tree(&project_root, 0).await;

    Ok(Value::Array(files))
}

// ─── File Operations ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateFileRequest {
    pub path: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub name: String,
}

#[tauri::command]
pub async fn create_file(
    _state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    body: CreateFileRequest,
) -> Result<Value, String> {
    _create_file(&project_name, body)
        .await
        .map_err(|e| e.to_string())
}

async fn _create_file(project_name: &str, body: CreateFileRequest) -> Result<Value, AppError> {
    let project_root = resolve_project_root(project_name).await?;

    let parent = if body.path.is_empty() {
        project_root.clone()
    } else {
        let p = PathBuf::from(&body.path);
        if p.is_absolute() {
            p
        } else {
            project_root.join(&body.path)
        }
    };

    let target = parent.join(&body.name);

    // Validate path is within project
    let canonical_root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.clone());
    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| AppError::NotFound("Parent directory not found".into()))?;
    if !canonical_parent.starts_with(&canonical_root) {
        return Err(AppError::BadRequest("Path traversal detected".into()));
    }

    if target.exists() {
        return Err(AppError::Conflict(format!(
            "{} already exists: {}",
            if body.item_type == "directory" {
                "Directory"
            } else {
                "File"
            },
            body.name
        )));
    }

    if body.item_type == "directory" {
        tokio::fs::create_dir_all(&target).await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to create directory: {e}"))
        })?;
    } else {
        tokio::fs::write(&target, "").await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to create file: {e}"))
        })?;
    }

    info!(name = %body.name, item_type = %body.item_type, "File/directory created");

    Ok(json!({ "success": true, "path": target.to_string_lossy() }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFileRequest {
    pub old_path: String,
    pub new_name: String,
}

#[tauri::command]
pub async fn rename_file(
    _state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    body: RenameFileRequest,
) -> Result<Value, String> {
    _rename_file(&project_name, body)
        .await
        .map_err(|e| e.to_string())
}

async fn _rename_file(project_name: &str, body: RenameFileRequest) -> Result<Value, AppError> {
    let project_root = resolve_project_root(project_name).await?;
    let old_path = validate_within_project(&body.old_path, &project_root)?;

    let new_path = old_path
        .parent()
        .ok_or_else(|| AppError::BadRequest("Invalid path".into()))?
        .join(&body.new_name);

    if new_path.exists() {
        return Err(AppError::Conflict(format!(
            "A file or directory named '{}' already exists",
            body.new_name
        )));
    }

    tokio::fs::rename(&old_path, &new_path).await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to rename: {e}"))
    })?;

    info!(old = %old_path.display(), new = %new_path.display(), "File renamed");

    Ok(json!({ "success": true, "newPath": new_path.to_string_lossy() }))
}

#[derive(Deserialize)]
pub struct DeleteFileRequest {
    pub path: String,
    #[serde(rename = "type")]
    pub item_type: String,
}

#[tauri::command]
pub async fn delete_file(
    _state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    body: DeleteFileRequest,
) -> Result<Value, String> {
    _delete_file(&project_name, body)
        .await
        .map_err(|e| e.to_string())
}

async fn _delete_file(project_name: &str, body: DeleteFileRequest) -> Result<Value, AppError> {
    let project_root = resolve_project_root(project_name).await?;
    let target = validate_within_project(&body.path, &project_root)?;

    if body.item_type == "directory" {
        tokio::fs::remove_dir_all(&target).await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to delete directory: {e}"))
        })?;
    } else {
        tokio::fs::remove_file(&target).await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to delete file: {e}"))
        })?;
    }

    info!(path = %target.display(), "File/directory deleted");

    Ok(json!({ "success": true }))
}

// ─── File Upload (base64 for Tauri IPC) ──────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileItem {
    pub name: String,
    pub data: String, // base64-encoded content
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFilesRequest {
    pub files: Vec<UploadFileItem>,
}

#[tauri::command]
pub async fn upload_files(
    _state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    body: UploadFilesRequest,
) -> Result<Value, String> {
    _upload_files(&project_name, body)
        .await
        .map_err(|e| e.to_string())
}

async fn _upload_files(project_name: &str, body: UploadFilesRequest) -> Result<Value, AppError> {
    let project_root = resolve_project_root(project_name).await?;
    let mut uploaded = Vec::new();

    for item in &body.files {
        // Prevent path traversal in file name
        let safe_name = std::path::Path::new(&item.name)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| item.name.clone());

        let data = base64::engine::general_purpose::STANDARD
            .decode(&item.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid base64 data: {e}")))?;

        let target = project_root.join(&safe_name);
        tokio::fs::write(&target, &data).await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to write file: {e}"))
        })?;

        uploaded.push(json!({
            "name": safe_name,
            "path": target.to_string_lossy(),
            "size": data.len(),
        }));

        debug!(name = %safe_name, size = data.len(), "File uploaded");
    }

    info!(count = uploaded.len(), "Files uploaded to project");

    Ok(json!({ "success": true, "files": uploaded }))
}

// ─── Image Upload (base64 for Tauri IPC) ─────────────────────────────────────

const MAX_IMAGE_SIZE: usize = 5 * 1024 * 1024; // 5MB
const MAX_IMAGE_COUNT: usize = 5;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadImageItem {
    pub name: String,
    pub data: String,      // base64-encoded content
    pub mime_type: String,  // e.g. "image/png"
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadImagesRequest {
    pub images: Vec<UploadImageItem>,
}

#[tauri::command]
pub async fn upload_images(
    _state: tauri::State<'_, Arc<AppState>>,
    body: UploadImagesRequest,
) -> Result<Value, String> {
    _upload_images(body).await.map_err(|e| e.to_string())
}

async fn _upload_images(body: UploadImagesRequest) -> Result<Value, AppError> {
    if body.images.is_empty() {
        return Err(AppError::BadRequest("No images provided".to_string()));
    }

    let mut images = Vec::new();

    for item in body.images.iter().take(MAX_IMAGE_COUNT) {
        // Validate MIME type
        if !item.mime_type.starts_with("image/") {
            return Err(AppError::BadRequest(format!(
                "Invalid file type: {}. Only images are allowed.",
                item.mime_type
            )));
        }

        let data = base64::engine::general_purpose::STANDARD
            .decode(&item.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid base64 data: {e}")))?;

        if data.len() > MAX_IMAGE_SIZE {
            return Err(AppError::BadRequest(format!(
                "Image '{}' exceeds maximum size of 5MB",
                item.name
            )));
        }

        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
        let data_uri = format!("data:{};base64,{encoded}", item.mime_type);

        images.push(json!({
            "name": item.name,
            "data": data_uri,
            "size": data.len(),
            "mimeType": item.mime_type,
        }));

        debug!(name = %item.name, size = data.len(), "Image processed");
    }

    if body.images.len() > MAX_IMAGE_COUNT {
        warn!("Too many images, max {MAX_IMAGE_COUNT}");
    }

    info!(count = images.len(), "Images uploaded successfully");

    Ok(json!({ "images": images }))
}

// ─── File Save ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFileRequest {
    pub file_path: String,
    pub content: String,
}

#[tauri::command]
pub async fn save_file(
    _state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    body: SaveFileRequest,
) -> Result<Value, String> {
    _save_file(&project_name, body)
        .await
        .map_err(|e| e.to_string())
}

async fn _save_file(project_name: &str, body: SaveFileRequest) -> Result<Value, AppError> {
    let project_root = resolve_project_root(project_name).await?;
    let file_path = validate_within_project(&body.file_path, &project_root)?;

    tokio::fs::write(&file_path, &body.content)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to save file: {e}")))?;

    info!(path = %file_path.display(), "File saved");

    Ok(json!({ "success": true }))
}

// ─── File Reading ────────────────────────────────────────────────────────────

/// Resolve project name to its root directory path.
async fn resolve_project_root(project_name: &str) -> Result<PathBuf, AppError> {
    let dir = project_scanner::extract_project_directory(project_name).await;
    let path = PathBuf::from(&dir);
    if !path.exists() {
        return Err(AppError::NotFound(format!(
            "Project directory not found: {dir}"
        )));
    }
    Ok(path)
}

/// Validate that a file path is within the project root (prevent path traversal).
fn validate_within_project(file_path: &str, project_root: &PathBuf) -> Result<PathBuf, AppError> {
    if file_path.contains('\0') {
        return Err(AppError::BadRequest("Invalid file path".into()));
    }

    let resolved = if std::path::Path::new(file_path).is_absolute() {
        PathBuf::from(file_path)
    } else {
        project_root.join(file_path)
    };

    let canonical_root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.clone());
    let canonical_file = resolved
        .canonicalize()
        .map_err(|_| AppError::NotFound("File not found".into()))?;

    if !canonical_file.starts_with(&canonical_root) {
        return Err(AppError::BadRequest("Path traversal detected".into()));
    }

    Ok(canonical_file)
}

#[tauri::command]
pub async fn read_file(
    _state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    file_path: String,
) -> Result<Value, String> {
    _read_file(&project_name, &file_path)
        .await
        .map_err(|e| e.to_string())
}

async fn _read_file(project_name: &str, file_path: &str) -> Result<Value, AppError> {
    let project_root = resolve_project_root(project_name).await?;
    let resolved_path = validate_within_project(file_path, &project_root)?;

    let content = tokio::fs::read_to_string(&resolved_path).await.map_err(|e| {
        match e.kind() {
            std::io::ErrorKind::NotFound => {
                AppError::NotFound(format!("File not found: {}", resolved_path.display()))
            }
            std::io::ErrorKind::PermissionDenied => {
                AppError::BadRequest("Permission denied".into())
            }
            std::io::ErrorKind::InvalidData => {
                AppError::BadRequest("File is binary and cannot be read as text".into())
            }
            _ => AppError::BadRequest(format!("Cannot read file as text: {e}")),
        }
    })?;

    Ok(json!({
        "content": content,
        "path": resolved_path.display().to_string(),
    }))
}

/// Read file as base64 (replaces binary response for Tauri IPC).
#[tauri::command]
pub async fn read_file_content(
    _state: tauri::State<'_, Arc<AppState>>,
    project_name: String,
    path: String,
) -> Result<Value, String> {
    _read_file_content(&project_name, &path)
        .await
        .map_err(|e| e.to_string())
}

async fn _read_file_content(project_name: &str, path: &str) -> Result<Value, AppError> {
    let project_root = resolve_project_root(project_name).await?;
    let file_path = validate_within_project(path, &project_root)?;

    let data = tokio::fs::read(&file_path).await.map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => {
            AppError::NotFound(format!("File not found: {}", file_path.display()))
        }
        std::io::ErrorKind::PermissionDenied => {
            AppError::BadRequest("Permission denied".into())
        }
        _ => AppError::Internal(e.into()),
    })?;

    let mime = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .to_string();

    let encoded = base64::engine::general_purpose::STANDARD.encode(&data);

    Ok(json!({
        "data": encoded,
        "mimeType": mime,
        "size": data.len(),
    }))
}

/// Read a file by absolute path as base64 (replaces raw binary response).
#[tauri::command]
pub async fn read_raw_file(
    _state: tauri::State<'_, Arc<AppState>>,
    path: String,
) -> Result<Value, String> {
    _read_raw_file(&path).await.map_err(|e| e.to_string())
}

async fn _read_raw_file(path: &str) -> Result<Value, AppError> {
    if path.contains('\0') {
        return Err(AppError::BadRequest("Invalid file path".into()));
    }

    let file_path = std::path::Path::new(path);
    if !file_path.is_absolute() {
        return Err(AppError::BadRequest("Path must be absolute".into()));
    }

    let canonical = file_path
        .canonicalize()
        .map_err(|_| AppError::NotFound("File not found".into()))?;

    let data = tokio::fs::read(&canonical).await.map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => {
            AppError::NotFound(format!("File not found: {}", canonical.display()))
        }
        std::io::ErrorKind::PermissionDenied => {
            AppError::BadRequest("Permission denied".into())
        }
        _ => AppError::Internal(e.into()),
    })?;

    let mime = mime_guess::from_path(&canonical)
        .first_or_octet_stream()
        .to_string();

    let encoded = base64::engine::general_purpose::STANDARD.encode(&data);

    Ok(json!({
        "data": encoded,
        "mimeType": mime,
        "size": data.len(),
    }))
}
