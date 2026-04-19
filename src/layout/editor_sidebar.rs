use leptos::prelude::*;
use leptos_fluent::tr;

use crate::features::files::file_editor::FileEditor;
use crate::features::files::image_preview::ImagePreview;
use crate::state::file_state::{is_image_file, FileContext};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::spinner::Spinner;
use crate::ui::toast_custom::toaster::expect_toaster;

/// Right-side editor sidebar panel.
/// Uses Show + individual reactive closures to avoid destroying FileEditor
/// when unrelated signals (dirty, loading) change.
#[component]
pub fn EditorSidebar() -> impl IntoView {
    let file_ctx = expect_context::<FileContext>();
    let ctx = expect_context::<crate::state::AppContext>();

    // Wrap toaster in StoredValue so on_save closure is Copy
    let toaster_save = StoredValue::new(expect_toaster());
    let on_save = StoredValue::new(move |_: web_sys::MouseEvent| {
        let project = ctx.selected_project.get_untracked();
        let toaster = toaster_save.get_value();
        if let Some(project) = project {
            file_ctx.save_file(project.name.clone());
            toaster.success(tr!("toast-file-saved"));
        }
    });

    let on_close = StoredValue::new(move |_: web_sys::MouseEvent| {
        file_ctx.close_editor();
    });

    // Derived memo: is current file an image?
    let is_image = Memo::new(move |_| {
        file_ctx
            .selected_file
            .get()
            .as_ref()
            .map(|f| is_image_file(&f.name))
            .unwrap_or(false)
    });

    view! {
        <Show when=move || {
            file_ctx.editor_open.get()
                && ctx.active_tab.get() == crate::tauri::types::AppTab::Files
        }>
            <div class="flex flex-col h-full w-[40%] min-w-80 max-w-[50%] border-l border-border bg-background">
                // Header bar
                <div class="flex items-center gap-2 px-3 py-2 border-b border-border shrink-0">
                    <div class="flex items-center gap-1.5 mr-auto min-w-0">
                        // File icon - reactive on is_image
                        {move || {
                            if is_image.get() {
                                view! { <icons::Image class="size-4 shrink-0 text-muted-foreground" /> }.into_any()
                            } else {
                                view! { <icons::FileCode class="size-4 shrink-0 text-muted-foreground" /> }.into_any()
                            }
                        }}
                        // File name - reactive on selected_file only
                        <span class="text-sm font-medium truncate">
                            {move || file_ctx.selected_file.get().as_ref().map(|f| f.name.clone()).unwrap_or_default()}
                        </span>
                        // Dirty badge - reactive on editor_dirty only
                        {move || {
                            if file_ctx.editor_dirty.get() {
                                view! {
                                    <span class="text-xs text-warning shrink-0">{tr!("files-editor-unsaved")}</span>
                                }.into_any()
                            } else {
                                view! {}.into_any()
                            }
                        }}
                    </div>

                    // Save button (only for text files)
                    {move || {
                        if !is_image.get() {
                            view! {
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Icon
                                    class="size-7"
                                    attr:title=tr!("files-editor-save")
                                    on:click=move |e| on_save.with_value(|f| f(e))
                                >
                                    <icons::Save class="size-4" />
                                </Button>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}

                    // Close button
                    <Button
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Icon
                        class="size-7"
                        attr:title=tr!("files-editor-close")
                        on:click=move |e| on_close.with_value(|f| f(e))
                    >
                        <icons::X class="size-4" />
                    </Button>
                </div>

                // Content area — flex column so FileEditor can use flex-1 to fill space
                <div class="flex-1 min-h-0 relative overflow-hidden flex flex-col">
                    // Loading overlay (on top of everything)
                    <Show when=move || file_ctx.editor_loading.get()>
                        <div class="absolute inset-0 flex items-center justify-center bg-background z-10">
                            <Spinner class="size-6" />
                        </div>
                    </Show>

                    // Error overlay
                    {move || {
                        if let Some(err) = file_ctx.error_message.get() {
                            view! {
                                <div class="absolute inset-0 flex flex-col items-center justify-center gap-2 p-4 text-destructive bg-background z-10">
                                    <icons::CircleAlert class="size-6" />
                                    <p class="text-sm text-center">{err}</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}

                    // Image preview (shown only for images, after loading)
                    <Show when=move || {
                        is_image.get()
                            && !file_ctx.editor_loading.get()
                            && file_ctx.error_message.get().is_none()
                    }>
                        <ImagePreview />
                    </Show>

                    // FileEditor — always mounted, hidden via CSS when image
                    <div class=move || {
                        if is_image.get() { "hidden" } else { "flex-1 min-h-0 w-full relative" }
                    }>
                        <FileEditor />
                    </div>
                </div>

                // Status bar
                <div class="flex items-center gap-3 px-3 py-1 border-t border-border text-xs text-muted-foreground shrink-0">
                    <span class="truncate">
                        {move || file_ctx.selected_file.get().as_ref().map(|f| f.path.clone()).unwrap_or_default()}
                    </span>
                    {move || {
                        let lang = file_ctx.selected_file.get().as_ref().map(|f| {
                            crate::state::file_state::get_language_name(&f.name).to_string()
                        }).unwrap_or_default();
                        if !lang.is_empty() {
                            view! {
                                <span class="ml-auto shrink-0">{lang}</span>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                </div>
            </div>
        </Show>
    }
}
