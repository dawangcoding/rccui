use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::state::file_state::{get_language_name, FileContext};

// ─── CodeMirror Bridge bindings ──────────────────────────────────────────────

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"])]
    fn create(id: &str, container: &web_sys::HtmlElement, opts: &JsValue) -> bool;

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"])]
    fn setValue(id: &str, content: &str);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"])]
    fn getValue(id: &str) -> String;

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"])]
    fn setLanguage(id: &str, lang_name: &str);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"])]
    fn onChange(id: &str, callback: &Closure<dyn FnMut(String)>);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"])]
    fn onSave(id: &str, callback: &Closure<dyn FnMut(String)>);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"])]
    fn dispose(id: &str);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"])]
    fn setTheme(id: &str);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"])]
    fn focus(id: &str);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"])]
    fn has(id: &str) -> bool;
}

// ─── FileEditor Component ────────────────────────────────────────────────────

const EDITOR_ID: &str = "cm_editor";

/// CodeMirror 6 editor component with wasm_bindgen interop.
///
/// Always disposes and recreates the editor when content changes (new file loaded).
/// This ensures the editor is always attached to the CURRENT container div,
/// even if Leptos's Show component destroyed and recreated this component.
///
/// The onChange → editor_content.set → Effect loop is broken by comparing
/// getValue() with the signal value: if they match, we skip entirely.
#[component]
pub fn FileEditor() -> impl IntoView {
    let file_ctx = expect_context::<FileContext>();
    let app_ctx = expect_context::<crate::state::AppContext>();
    let container_ref = NodeRef::<leptos::html::Div>::new();

    // Effect: create/update editor when content or selected file changes
    Effect::new(move || {
        // Track all dependencies reactively
        let content = file_ctx.editor_content.get();
        let selected = file_ctx.selected_file.get();
        let loading = file_ctx.editor_loading.get();
        let container = container_ref.get();

        web_sys::console::log_1(
            &format!(
                "[FileEditor] Effect fired: loading={}, content={}, file={}, container={}, has_editor={}",
                loading,
                match &content {
                    Some(c) => format!("Some({}chars)", c.len()),
                    None => "None".to_string(),
                },
                selected.as_ref().map(|f| f.name.as_str()).unwrap_or("None"),
                if container.is_some() { "YES" } else { "NO" },
                has(EDITOR_ID),
            )
            .into(),
        );

        // Wait until loading finishes
        if loading {
            web_sys::console::log_1(&"[FileEditor] → skipped (loading=true)".into());
            return;
        }

        if let (Some(content), Some(file), Some(container)) =
            (content, selected, container)
        {
            let lang = get_language_name(&file.name);

            // Guard: if editor exists AND content matches, skip (breaks onChange cycle)
            if has(EDITOR_ID) {
                let current = getValue(EDITOR_ID);
                web_sys::console::log_1(
                    &format!(
                        "[FileEditor] Editor exists, getValue={}chars, content={}chars, match={}",
                        current.len(), content.len(), current == content
                    ).into(),
                );
                if current == content {
                    web_sys::console::log_1(&"[FileEditor] → skipped (content match)".into());
                    return;
                }
            }

            // Dispose any existing instance
            if has(EDITOR_ID) {
                web_sys::console::log_1(&"[FileEditor] Disposing old editor".into());
                dispose(EDITOR_ID);
            }

            // Create fresh editor in the CURRENT container
            web_sys::console::log_1(
                &format!(
                    "[FileEditor] Creating editor: {}chars, lang='{}', container_tag={}",
                    content.len(), lang,
                    container.tag_name(),
                ).into(),
            );
            let opts = js_sys::Object::new();
            let _ = js_sys::Reflect::set(
                &opts,
                &JsValue::from_str("value"),
                &JsValue::from_str(&content),
            );
            if !lang.is_empty() {
                let _ = js_sys::Reflect::set(
                    &opts,
                    &JsValue::from_str("language"),
                    &JsValue::from_str(lang),
                );
            }

            let success = create(EDITOR_ID, &container, &opts.into());
            web_sys::console::log_1(
                &format!("[FileEditor] create() returned: {}", success).into(),
            );
            if success {
                // Register onChange callback — updates content & dirty state
                let file_ctx_change = file_ctx;
                let on_change_closure =
                    Closure::<dyn FnMut(String)>::new(move |new_content: String| {
                        file_ctx_change.editor_content.set(Some(new_content));
                        file_ctx_change.editor_dirty.set(true);
                    });
                onChange(EDITOR_ID, &on_change_closure);
                std::mem::forget(on_change_closure);

                // Register onSave callback (Cmd/Ctrl+S)
                let file_ctx_save = file_ctx;
                let app_ctx_save = app_ctx;
                let on_save_closure =
                    Closure::<dyn FnMut(String)>::new(move |_content: String| {
                        let project = app_ctx_save.selected_project.get_untracked();
                        if let Some(project) = project {
                            file_ctx_save.save_file(project.name.clone());
                        }
                    });
                onSave(EDITOR_ID, &on_save_closure);
                std::mem::forget(on_save_closure);

                file_ctx.editor_dirty.set(false);
                focus(EDITOR_ID);
            }
        }
    });

    // Cleanup on unmount
    on_cleanup(move || {
        if has(EDITOR_ID) {
            dispose(EDITOR_ID);
        }
    });

    view! {
        <div node_ref=container_ref class="absolute inset-0 overflow-hidden" />
    }
}
