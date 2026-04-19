use leptos::ev::{Event, KeyboardEvent};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::state::{AppContext, FileContext};
use crate::tauri::commands;
use crate::ui::toast_custom::_context::ToasterContext;

const INDENT: &str = "    ";

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

/// Stable textarea-based editor used as a fallback for Tauri/WebKit.
#[component]
pub fn FileEditor() -> impl IntoView {
    let file_ctx = expect_context::<FileContext>();
    let app_ctx = expect_context::<AppContext>();
    let toaster = crate::ui::toast_custom::toaster::expect_toaster();
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

    // Focus after a file finishes loading so keyboard editing feels immediate.
    Effect::new(move || {
        let content = file_ctx.editor_content.get();
        let loading = file_ctx.editor_loading.get();
        if !loading && content.is_some() {
            if let Some(textarea) = textarea_ref.get() {
                let _ = textarea.focus();
            }
        }
    });

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
