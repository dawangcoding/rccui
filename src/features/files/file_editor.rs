use leptos::ev::{Event, KeyboardEvent};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::hooks::use_theme_mode::use_theme_mode;
use crate::state::{AppContext, FileContext};
use crate::state::file_state::{get_language_name, is_image_file};
use crate::tauri::commands;
use crate::ui::toast_custom::_context::ToasterContext;

const INDENT: &str = "    ";
const CODEMIRROR_EDITOR_ID: &str = "file_editor_main";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = create)]
    fn cm_create(
        id: &str,
        container: &web_sys::HtmlElement,
        opts: &JsValue,
    ) -> bool;

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = setValue)]
    fn cm_set_value(id: &str, content: &str);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = setLanguage)]
    fn cm_set_language(id: &str, lang_name: &str);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = onChange)]
    fn cm_on_change(id: &str, callback: &Closure<dyn FnMut(String)>);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = onSave)]
    fn cm_on_save(id: &str, callback: &Closure<dyn FnMut(String)>);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = dispose)]
    fn cm_dispose(id: &str);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = setTheme)]
    fn cm_set_theme(id: &str);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = setReadOnly)]
    fn cm_set_read_only(id: &str, read_only: bool);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = focus)]
    fn cm_focus(id: &str);

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = has)]
    fn cm_has(id: &str) -> bool;

    #[wasm_bindgen(js_namespace = ["window", "CodeMirrorBridge"], js_name = syncLayout)]
    fn cm_sync_layout(id: &str);
}

pub fn save_current_file(
    file_ctx: FileContext,
    app_ctx: AppContext,
    toaster: ToasterContext,
    success_message: String,
    error_prefix: String,
) {
    let Some(project) = app_ctx.selected_project.get_untracked() else {
        return;
    };
    let Some(file) = file_ctx.selected_file.get_untracked() else {
        return;
    };
    let Some(content) = file_ctx.editor_content.get_untracked() else {
        return;
    };

    let ctx = file_ctx;
    spawn_local(async move {
        match commands::save_file(&project.name, &file.path, &content).await {
            Ok(_) => {
                ctx.editor_dirty.set(false);
                ctx.error_message.set(None);
                toaster.success(success_message);
            }
            Err(e) => {
                let message = format!("{error_prefix}: {e}");
                ctx.error_message.set(Some(message.clone()));
                toaster.error(message);
            }
        }
    });
}

fn insert_indent_at_cursor(
    textarea: &web_sys::HtmlTextAreaElement,
    file_ctx: FileContext,
) {
    let value = textarea.value();
    let start = textarea
        .selection_start()
        .ok()
        .flatten()
        .unwrap_or(value.len() as u32) as usize;
    let end = textarea
        .selection_end()
        .ok()
        .flatten()
        .unwrap_or(start as u32) as usize;

    let mut next = String::with_capacity(value.len() + INDENT.len());
    next.push_str(&value[..start]);
    next.push_str(INDENT);
    next.push_str(&value[end..]);

    textarea.set_value(&next);
    let caret = (start + INDENT.len()) as u32;
    let _ = textarea.set_selection_range(caret, caret);

    file_ctx.editor_content.set(Some(next));
    file_ctx.editor_dirty.set(true);
}

fn has_codemirror_bridge() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(bridge) = js_sys::Reflect::get(
        &JsValue::from(window),
        &JsValue::from_str("CodeMirrorBridge"),
    ) else {
        return false;
    };
    bridge.is_object() && !bridge.is_null()
}

#[component]
fn TextareaFallback(
    textarea_ref: NodeRef<leptos::html::Textarea>,
    file_ctx: FileContext,
    app_ctx: AppContext,
    toaster: crate::ui::toast_custom::_context::ToasterContext,
) -> impl IntoView {
    let on_input = move |ev: Event| {
        let value = event_target_value(&ev);
        file_ctx.editor_content.set(Some(value));
        file_ctx.editor_dirty.set(true);
    };

    let save_success = leptos_fluent::tr!("toast-file-saved");
    let save_error = leptos_fluent::tr!("toast-file-save-failed");
    let on_keydown = move |ev: KeyboardEvent| {
        if ev.key() == "Tab" {
            ev.prevent_default();
            if let Some(textarea) = textarea_ref.get_untracked() {
                insert_indent_at_cursor(&textarea, file_ctx);
            }
            return;
        }

        if ev.key().eq_ignore_ascii_case("s") && (ev.ctrl_key() || ev.meta_key()) {
            ev.prevent_default();
            save_current_file(
                file_ctx,
                app_ctx,
                toaster.clone(),
                save_success.clone(),
                save_error.clone(),
            );
        }
    };

    view! {
        <textarea
            node_ref=textarea_ref
            class="absolute inset-0 h-full w-full resize-none border-0 bg-background px-4 py-3 font-mono text-[13px] leading-6 text-foreground outline-none overflow-auto whitespace-pre selection:bg-primary/20 selection:text-foreground [font-variant-ligatures:none]"
            spellcheck="false"
            autocapitalize="off"
            autocomplete="off"
            wrap="off"
            style="tab-size: 4;"
            prop:value=move || file_ctx.editor_content.get().unwrap_or_default()
            on:input=on_input
            on:keydown=on_keydown
        />
    }
}

/// File editor with CodeMirror as default and textarea fallback for safety.
#[component]
pub fn FileEditor() -> impl IntoView {
    let file_ctx = expect_context::<FileContext>();
    let app_ctx = expect_context::<AppContext>();
    let toaster = crate::ui::toast_custom::toaster::expect_toaster();
    let toaster_for_effect = toaster.clone();
    let toaster_for_fallback = toaster.clone();
    let theme_mode = use_theme_mode();

    let container_ref = NodeRef::<leptos::html::Div>::new();
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let use_fallback = RwSignal::new(false);

    let save_success = leptos_fluent::tr!("toast-file-saved");
    let save_error = leptos_fluent::tr!("toast-file-save-failed");

    // Initialize CodeMirror once the editor container exists. If initialization
    // fails, fallback to textarea instead of blocking file edits.
    Effect::new(move || {
        let selected = file_ctx.selected_file.get();
        let Some(file) = selected else {
            return;
        };
        if is_image_file(&file.name) || use_fallback.get() {
            return;
        }
        let Some(container) = container_ref.get() else {
            return;
        };

        if !has_codemirror_bridge() {
            use_fallback.set(true);
            return;
        }

        if !cm_has(CODEMIRROR_EDITOR_ID) {
            let opts = js_sys::Object::new();
            let content = file_ctx.editor_content.get().unwrap_or_default();
            let lang = get_language_name(&file.name);
            let _ = js_sys::Reflect::set(
                &opts,
                &JsValue::from_str("value"),
                &JsValue::from_str(&content),
            );
            let _ = js_sys::Reflect::set(
                &opts,
                &JsValue::from_str("language"),
                &JsValue::from_str(lang),
            );
            let _ = js_sys::Reflect::set(
                &opts,
                &JsValue::from_str("readOnly"),
                &JsValue::from_bool(false),
            );
            let _ = js_sys::Reflect::set(
                &opts,
                &JsValue::from_str("fontSize"),
                &JsValue::from_f64(13.0),
            );

            if !cm_create(CODEMIRROR_EDITOR_ID, &container, &JsValue::from(opts)) {
                use_fallback.set(true);
                return;
            }

            let file_ctx_on_change = file_ctx;
            let on_change =
                Closure::<dyn FnMut(String)>::new(move |content: String| {
                    file_ctx_on_change.editor_content.set(Some(content));
                    file_ctx_on_change.editor_dirty.set(true);
                });
            cm_on_change(CODEMIRROR_EDITOR_ID, &on_change);
            std::mem::forget(on_change);

            let file_ctx_on_save = file_ctx;
            let app_ctx_on_save = app_ctx;
            let toaster_on_save = toaster_for_effect.clone();
            let save_success_on_save = save_success.clone();
            let save_error_on_save = save_error.clone();
            let on_save = Closure::<dyn FnMut(String)>::new(move |content: String| {
                file_ctx_on_save.editor_content.set(Some(content));
                save_current_file(
                    file_ctx_on_save,
                    app_ctx_on_save,
                    toaster_on_save.clone(),
                    save_success_on_save.clone(),
                    save_error_on_save.clone(),
                );
            });
            cm_on_save(CODEMIRROR_EDITOR_ID, &on_save);
            std::mem::forget(on_save);
        }

        if let Some(content) = file_ctx.editor_content.get() {
            cm_set_value(CODEMIRROR_EDITOR_ID, &content);
        } else {
            cm_set_value(CODEMIRROR_EDITOR_ID, "");
        }

        let lang = get_language_name(&file.name);
        cm_set_language(CODEMIRROR_EDITOR_ID, lang);
        cm_set_read_only(CODEMIRROR_EDITOR_ID, false);
        cm_sync_layout(CODEMIRROR_EDITOR_ID);

        if !file_ctx.editor_loading.get() {
            cm_focus(CODEMIRROR_EDITOR_ID);
        }
    });

    // Sync CodeMirror theme when app theme changes.
    Effect::new(move || {
        let _ = theme_mode.get();
        if cm_has(CODEMIRROR_EDITOR_ID) {
            cm_set_theme(CODEMIRROR_EDITOR_ID);
        }
    });

    // Focus active editor after file finishes loading.
    Effect::new(move || {
        let content = file_ctx.editor_content.get();
        let loading = file_ctx.editor_loading.get();
        if loading || content.is_none() {
            return;
        }
        if use_fallback.get() {
            if let Some(textarea) = textarea_ref.get() {
                let _ = textarea.focus();
            }
        } else if cm_has(CODEMIRROR_EDITOR_ID) {
            cm_focus(CODEMIRROR_EDITOR_ID);
        }
    });

    on_cleanup(move || {
        if cm_has(CODEMIRROR_EDITOR_ID) {
            cm_dispose(CODEMIRROR_EDITOR_ID);
        }
    });

    view! {
        <Show
            when=move || !use_fallback.get()
            fallback=move || {
                view! {
                    <TextareaFallback
                        textarea_ref=textarea_ref
                        file_ctx=file_ctx
                        app_ctx=app_ctx
                        toaster=toaster_for_fallback.clone()
                    />
                }
            }
        >
            <div
                node_ref=container_ref
                class="absolute inset-0 h-full w-full bg-background font-mono text-[13px] leading-6 text-foreground [font-variant-ligatures:none]"
            />
        </Show>
    }
}
