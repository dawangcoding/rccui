use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::tauri::commands;
use crate::tauri::types::FileNode;

// ─── FileContext ─────────────────────────────────────────────────────────────

/// State management for the Files feature.
#[derive(Clone, Copy)]
pub struct FileContext {
    /// Full file tree from `list_files`
    pub tree: RwSignal<Vec<FileNode>>,
    /// Whether the tree is currently loading
    pub tree_loading: RwSignal<bool>,
    /// Set of expanded directory paths (controls tree expand/collapse)
    pub expanded_dirs: RwSignal<HashSet<String>>,
    /// Currently selected file node
    pub selected_file: RwSignal<Option<FileNode>>,
    /// Text content of the currently open file
    pub editor_content: RwSignal<Option<String>>,
    /// Whether the editor has unsaved changes
    pub editor_dirty: RwSignal<bool>,
    /// Whether a file is currently being loaded
    pub editor_loading: RwSignal<bool>,
    /// Whether the editor sidebar panel is visible
    pub editor_open: RwSignal<bool>,
    /// Base64 data URI for image preview (e.g., "data:image/png;base64,...")
    pub image_data: RwSignal<Option<String>>,
    /// Error message from last file operation
    pub error_message: RwSignal<Option<String>>,
}

impl FileContext {
    pub fn new() -> Self {
        Self {
            tree: RwSignal::new(Vec::new()),
            tree_loading: RwSignal::new(false),
            expanded_dirs: RwSignal::new(HashSet::new()),
            selected_file: RwSignal::new(None),
            editor_content: RwSignal::new(None),
            editor_dirty: RwSignal::new(false),
            editor_loading: RwSignal::new(false),
            editor_open: RwSignal::new(false),
            image_data: RwSignal::new(None),
            error_message: RwSignal::new(None),
        }
    }

    /// Load the full file tree for a project.
    pub fn load_tree(&self, project_name: String) {
        let ctx = *self;
        ctx.tree_loading.set(true);
        ctx.error_message.set(None);
        spawn_local(async move {
            match commands::list_files(&project_name).await {
                Ok(nodes) => {
                    ctx.tree.set(nodes);
                }
                Err(e) => {
                    ctx.error_message.set(Some(format!("Failed to load files: {e}")));
                }
            }
            ctx.tree_loading.set(false);
        });
    }

    /// Toggle a directory's expanded/collapsed state.
    pub fn toggle_dir(&self, path: String) {
        self.expanded_dirs.update(|dirs| {
            if dirs.contains(&path) {
                dirs.remove(&path);
            } else {
                dirs.insert(path);
            }
        });
    }

    /// Select a file and load its content (text or image).
    pub fn select_file(&self, node: FileNode, project_name: String) {
        let ctx = *self;
        web_sys::console::log_1(
            &format!("[select_file] file={}, editor_open={}", node.name, ctx.editor_open.get_untracked()).into(),
        );
        ctx.selected_file.set(Some(node.clone()));
        ctx.editor_dirty.set(false);
        ctx.editor_content.set(None);
        ctx.image_data.set(None);
        ctx.error_message.set(None);
        // Only set if not already open — .set(true) always notifies subscribers,
        // which causes Show to destroy and recreate FileEditor (losing the editor).
        if !ctx.editor_open.get_untracked() {
            web_sys::console::log_1(&"[select_file] setting editor_open=true".into());
            ctx.editor_open.set(true);
        }
        ctx.editor_loading.set(true);

        let file_path = node.path.clone();
        let file_name = node.name.clone();

        spawn_local(async move {
            if is_image_file(&file_name) {
                match commands::read_file_content(&project_name, &file_path).await {
                    Ok(resp) => {
                        let data_uri =
                            format!("data:{};base64,{}", resp.mime_type, resp.data);
                        ctx.image_data.set(Some(data_uri));
                    }
                    Err(e) => {
                        ctx.error_message
                            .set(Some(format!("Failed to load image: {e}")));
                    }
                }
            } else if !is_binary_file(&file_name) {
                match commands::read_file(&project_name, &file_path).await {
                    Ok(resp) => {
                        web_sys::console::log_1(
                            &format!("[select_file] read_file OK: {}chars", resp.content.len()).into(),
                        );
                        ctx.editor_content.set(Some(resp.content));
                    }
                    Err(e) => {
                        web_sys::console::log_1(
                            &format!("[select_file] read_file ERROR: {e}").into(),
                        );
                        ctx.error_message
                            .set(Some(format!("Failed to load file: {e}")));
                    }
                }
            } else {
                ctx.error_message
                    .set(Some("Cannot preview binary files".to_string()));
            }
            web_sys::console::log_1(&"[select_file] setting editor_loading=false".into());
            ctx.editor_loading.set(false);
        });
    }

    /// Save the currently open file.
    pub fn save_file(&self, project_name: String) {
        let ctx = *self;
        let file = ctx.selected_file.get_untracked();
        let content = ctx.editor_content.get_untracked();

        if let (Some(file), Some(content)) = (file, content) {
            spawn_local(async move {
                match commands::save_file(&project_name, &file.path, &content).await {
                    Ok(_) => {
                        ctx.editor_dirty.set(false);
                    }
                    Err(e) => {
                        ctx.error_message
                            .set(Some(format!("Failed to save: {e}")));
                    }
                }
            });
        }
    }

    /// Close the editor sidebar panel.
    pub fn close_editor(&self) {
        web_sys::console::log_1(&"[close_editor] called".into());
        self.selected_file.set(None);
        self.editor_content.set(None);
        self.image_data.set(None);
        self.editor_dirty.set(false);
        self.editor_open.set(false);
        self.error_message.set(None);
    }

    /// Refresh the file tree, preserving expanded directories.
    pub fn refresh_tree(&self, project_name: String) {
        let ctx = *self;
        ctx.tree_loading.set(true);
        ctx.error_message.set(None);
        spawn_local(async move {
            match commands::list_files(&project_name).await {
                Ok(nodes) => {
                    ctx.tree.set(nodes);
                }
                Err(e) => {
                    ctx.error_message.set(Some(format!("Failed to refresh: {e}")));
                }
            }
            ctx.tree_loading.set(false);
        });
    }

    /// Reset all state (called when switching projects).
    pub fn clear(&self) {
        self.tree.set(Vec::new());
        self.tree_loading.set(false);
        self.expanded_dirs.set(HashSet::new());
        self.selected_file.set(None);
        self.editor_content.set(None);
        self.editor_dirty.set(false);
        self.editor_loading.set(false);
        self.editor_open.set(false);
        self.image_data.set(None);
        self.error_message.set(None);
    }
}

// ─── File type helpers ──────────────────────────────────────────────────────

/// Check if a filename refers to an image file.
pub fn is_image_file(name: &str) -> bool {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp"
    )
}

/// Check if a filename refers to a known binary file type.
pub fn is_binary_file(name: &str) -> bool {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(
        ext.as_str(),
        "zip" | "tar"
            | "gz"
            | "bz2"
            | "xz"
            | "7z"
            | "rar"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "wasm"
            | "pdf"
            | "doc"
            | "docx"
            | "xls"
            | "xlsx"
            | "ppt"
            | "pptx"
            | "ttf"
            | "otf"
            | "woff"
            | "woff2"
            | "eot"
            | "mp3"
            | "mp4"
            | "avi"
            | "mov"
            | "mkv"
            | "wav"
            | "flac"
            | "ogg"
            | "class"
            | "o"
            | "a"
            | "pyc"
            | "pyo"
    )
}

/// Map a filename to the CodeMirror language name for syntax highlighting.
pub fn get_language_name(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "jsx",
        "ts" | "mts" => "typescript",
        "tsx" => "tsx",
        "html" | "htm" | "svelte" | "vue" => "html",
        "css" | "scss" | "less" => "css",
        "json" | "jsonc" => "json",
        "md" | "mdx" => "markdown",
        "py" | "pyw" => "python",
        "rs" => "rust",
        "c" | "h" | "cpp" | "hpp" | "cc" | "cxx" => "cpp",
        "java" | "kt" | "kts" => "java",
        "xml" | "svg" | "plist" => "xml",
        "sql" => "sql",
        "yaml" | "yml" => "yaml",
        _ => "",
    }
}
