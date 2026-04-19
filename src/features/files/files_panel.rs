use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fluent::tr;

use crate::state::{AppContext, FileContext};
use crate::tauri::commands;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::input::Input;
use crate::ui::scroll_area::ScrollArea;
use crate::ui::spinner::Spinner;
use crate::ui::toast_custom::toaster::expect_toaster;

use super::file_tree::FileTree;

/// Main files panel — toolbar + file tree for the Files tab.
#[component]
pub fn FilesPanel() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let file_ctx = expect_context::<FileContext>();
    // Load tree when project changes
    Effect::new(move || {
        let project = ctx.selected_project.get();
        if let Some(project) = project {
            file_ctx.load_tree(project.name.clone());
        } else {
            file_ctx.clear();
        }
    });

    // Create file/folder dialog state
    let create_open = RwSignal::new(false);
    let create_name = RwSignal::new(String::new());
    let create_type = RwSignal::new(String::from("file"));

    // Wrap toaster in StoredValue so closure is Copy
    let toaster_create = StoredValue::new(expect_toaster());
    let on_create_confirm = StoredValue::new(move |_: web_sys::MouseEvent| {
        let name = create_name.get_untracked();
        if name.is_empty() {
            create_open.set(false);
            return;
        }
        let project = ctx.selected_project.get_untracked();
        let item_type = create_type.get_untracked();
        let toaster = toaster_create.get_value();
        if let Some(project) = project {
            let project_name = project.name.clone();
            let project_name2 = project.name.clone();
            let name_msg = name.clone();
            spawn_local(async move {
                match commands::create_file(&project_name, "", &item_type, &name).await {
                    Ok(_) => {
                        toaster.success(tr!("toast-file-created", { "name" => name_msg }));
                        file_ctx.refresh_tree(project_name2);
                    }
                    Err(e) => {
                        toaster.error(format!("{}: {e}", tr!("toast-file-create-failed")));
                    }
                }
            });
        }
        create_open.set(false);
        create_name.set(String::new());
    });

    view! {
        <div class="flex flex-col flex-1 min-h-0 min-w-0 overflow-hidden">
            // Toolbar
            <div class="flex items-center gap-1 px-3 py-2 border-b border-border shrink-0">
                <div class="flex items-center gap-1.5 text-sm font-medium text-foreground mr-auto">
                    <icons::FolderTree class="size-4 text-muted-foreground" />
                    <span>{tr!("tab-files")}</span>
                </div>

                // New File button
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Icon
                    class="size-7"
                    attr:title=tr!("files-toolbar-new-file")
                    on:click=move |_| {
                        create_type.set("file".to_string());
                        create_name.set(String::new());
                        create_open.set(true);
                    }
                >
                    <icons::FilePlus class="size-4" />
                </Button>

                // New Folder button
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Icon
                    class="size-7"
                    attr:title=tr!("files-toolbar-new-folder")
                    on:click=move |_| {
                        create_type.set("directory".to_string());
                        create_name.set(String::new());
                        create_open.set(true);
                    }
                >
                    <icons::FolderPlus class="size-4" />
                </Button>

                // Refresh button
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Icon
                    class="size-7"
                    attr:title=tr!("files-toolbar-refresh")
                    on:click=move |_| {
                        let project = ctx.selected_project.get_untracked();
                        if let Some(project) = project {
                            file_ctx.refresh_tree(project.name.clone());
                        }
                    }
                >
                    <icons::RefreshCw class="size-4" />
                </Button>

                // Loading indicator
                {move || {
                    if file_ctx.tree_loading.get() {
                        view! { <Spinner class="size-4" /> }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}
            </div>

            // File tree area
            <ScrollArea class="flex-1 min-h-0">
                <div class="py-1">
                    {move || {
                        let tree = file_ctx.tree.get();
                        let loading = file_ctx.tree_loading.get();
                        let project = ctx.selected_project.get();
                        let error = file_ctx.error_message.get();

                        if project.is_none() {
                            view! {
                                <div class="flex flex-col items-center justify-center gap-2 py-12 text-muted-foreground">
                                    <icons::FolderTree class="size-8 opacity-20" />
                                    <p class="text-sm">{tr!("files-empty-state")}</p>
                                </div>
                            }.into_any()
                        } else if loading && tree.is_empty() {
                            view! {
                                <div class="flex items-center justify-center gap-2 py-12 text-sm text-muted-foreground">
                                    <Spinner class="size-4" />
                                    <span>{tr!("files-loading")}</span>
                                </div>
                            }.into_any()
                        } else if let Some(err) = error {
                            view! {
                                <div class="flex flex-col items-center justify-center gap-2 py-12 text-destructive">
                                    <icons::CircleAlert class="size-6" />
                                    <p class="text-sm">{err}</p>
                                </div>
                            }.into_any()
                        } else if tree.is_empty() {
                            view! {
                                <div class="flex flex-col items-center justify-center gap-2 py-12 text-muted-foreground">
                                    <icons::FolderOpen class="size-8 opacity-20" />
                                    <p class="text-sm">{tr!("files-no-files")}</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <FileTree nodes=tree depth=0 />
                            }.into_any()
                        }
                    }}
                </div>
            </ScrollArea>

            // Create File/Folder Modal
            <Show when=move || create_open.get()>
                <div class="fixed inset-0 z-50 bg-black/50" on:click=move |_| create_open.set(false) />
                <div class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50 bg-background border rounded-2xl shadow-lg p-6 w-full max-w-md">
                    <div class="flex flex-col gap-2 text-left mb-4">
                        <h3 class="text-lg leading-none font-semibold">
                            {move || {
                                if create_type.get() == "directory" {
                                    tr!("files-create-folder-title")
                                } else {
                                    tr!("files-create-file-title")
                                }
                            }}
                        </h3>
                    </div>
                    <Input placeholder=tr!("files-name-placeholder") bind_value=create_name />
                    <div class="flex flex-row gap-2 justify-end mt-4">
                        <Button variant=ButtonVariant::Outline on:click=move |_| create_open.set(false)>
                            {tr!("action-cancel")}
                        </Button>
                        <Button on:click=move |e| on_create_confirm.with_value(|f| f(e))>
                            {tr!("action-save")}
                        </Button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
