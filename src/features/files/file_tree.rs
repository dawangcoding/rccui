use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fluent::tr;

use crate::state::file_state::{is_image_file, FileContext};
use crate::state::AppContext;
use crate::tauri::commands;
use crate::tauri::types::FileNode;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::input::Input;
use crate::ui::toast_custom::toaster::expect_toaster;

/// Recursive file tree — renders a list of file/directory nodes.
#[component]
pub fn FileTree(
    #[prop(into)] nodes: Vec<FileNode>,
    #[prop(default = 0)] depth: usize,
) -> impl IntoView {
    nodes
        .into_iter()
        .map(|node| {
            view! { <FileTreeNode node=node depth=depth /> }
        })
        .collect_view()
}

/// A single file or directory node in the tree.
#[component]
fn FileTreeNode(node: FileNode, depth: usize) -> impl IntoView {
    let file_ctx = expect_context::<FileContext>();
    let ctx = expect_context::<AppContext>();
    let is_dir = node.file_type == "directory";
    let node_path = node.path.clone();
    let node_name = node.name.clone();
    let node_clone = node.clone();
    let children = node.children.clone().unwrap_or_default();

    // Rename dialog state
    let rename_open = RwSignal::new(false);
    let rename_value = RwSignal::new(node.name.clone());

    // Delete dialog state
    let delete_open = RwSignal::new(false);

    // Create file/folder dialog state (for directories)
    let create_open = RwSignal::new(false);
    let create_name = RwSignal::new(String::new());
    let create_type = RwSignal::new(String::from("file"));

    let padding_left = format!("{}rem", depth as f64 * 1.0);

    // Icon for file type
    let file_icon = if is_dir {
        view! {
            {move || {
                let expanded = file_ctx.expanded_dirs.get();
                if expanded.contains(&node_path) {
                    view! { <icons::FolderOpen class="size-4 shrink-0 text-muted-foreground" /> }.into_any()
                } else {
                    view! { <icons::Folder class="size-4 shrink-0 text-muted-foreground" /> }.into_any()
                }
            }}
        }
        .into_any()
    } else {
        let icon_class = "size-4 shrink-0 text-muted-foreground";
        if is_image_file(&node_name) {
            view! { <icons::Image class=icon_class /> }.into_any()
        } else {
            view! { <icons::FileCode class=icon_class /> }.into_any()
        }
    };

    // Chevron for directories
    let chevron = if is_dir {
        let path_for_chevron = node.path.clone();
        view! {
            {move || {
                let expanded = file_ctx.expanded_dirs.get();
                if expanded.contains(&path_for_chevron) {
                    view! { <icons::ChevronDown class="size-3.5 shrink-0 text-muted-foreground" /> }.into_any()
                } else {
                    view! { <icons::ChevronRight class="size-3.5 shrink-0 text-muted-foreground" /> }.into_any()
                }
            }}
        }
        .into_any()
    } else {
        view! { <span class="w-3.5 shrink-0"></span> }.into_any()
    };

    // Click handler
    let path_for_click = node.path.clone();
    let node_for_select = node_clone.clone();
    let on_click = move |_: web_sys::MouseEvent| {
        if is_dir {
            file_ctx.toggle_dir(path_for_click.clone());
        } else {
            let project = ctx.selected_project.get_untracked();
            if let Some(project) = project {
                file_ctx.select_file(node_for_select.clone(), project.name.clone());
            }
        }
    };

    // Wrap non-Copy values in StoredValue for Copy closures
    let sv_path_rename = StoredValue::new(node.path.clone());
    let sv_name_rename = StoredValue::new(node.name.clone());
    let sv_type_str = StoredValue::new(node.file_type.clone());
    let sv_path_delete = StoredValue::new(node.path.clone());
    let sv_path_create = StoredValue::new(node.path.clone());
    let sv_name_delete = StoredValue::new(node.name.clone());
    let sv_toaster_r = StoredValue::new(expect_toaster());
    let sv_toaster_d = StoredValue::new(expect_toaster());
    let sv_toaster_c = StoredValue::new(expect_toaster());

    // Wrap handlers in StoredValue so they are Copy for Show children
    let on_rename_confirm = StoredValue::new(move |_: web_sys::MouseEvent| {
        let new_name = rename_value.get_untracked();
        let original_name = sv_name_rename.get_value();
        if new_name.is_empty() || new_name == original_name {
            rename_open.set(false);
            return;
        }
        let project = ctx.selected_project.get_untracked();
        let old_path = sv_path_rename.get_value();
        let toaster = sv_toaster_r.get_value();
        if let Some(project) = project {
            let project_name = project.name.clone();
            let project_name2 = project.name.clone();
            let new_name_msg = new_name.clone();
            spawn_local(async move {
                match commands::rename_file(&project_name, &old_path, &new_name).await {
                    Ok(_) => {
                        toaster.success(tr!("toast-file-renamed", { "name" => new_name_msg }));
                        file_ctx.refresh_tree(project_name2);
                    }
                    Err(e) => {
                        toaster.error(format!("{}: {e}", tr!("toast-file-rename-failed")));
                    }
                }
            });
        }
        rename_open.set(false);
    });

    let on_delete_confirm = StoredValue::new(move |_: web_sys::MouseEvent| {
        let project = ctx.selected_project.get_untracked();
        let path = sv_path_delete.get_value();
        let item_type = sv_type_str.get_value();
        let toaster = sv_toaster_d.get_value();
        let name_msg = sv_name_delete.get_value();
        if let Some(project) = project {
            let project_name = project.name.clone();
            let project_name2 = project.name.clone();
            spawn_local(async move {
                match commands::delete_file(&project_name, &path, &item_type).await {
                    Ok(_) => {
                        toaster.success(tr!("toast-file-deleted", { "name" => name_msg }));
                        file_ctx.close_editor();
                        file_ctx.refresh_tree(project_name2);
                    }
                    Err(e) => {
                        toaster.error(format!("{}: {e}", tr!("toast-file-delete-failed")));
                    }
                }
            });
        }
        delete_open.set(false);
    });

    let on_create_confirm = StoredValue::new(move |_: web_sys::MouseEvent| {
        let name = create_name.get_untracked();
        if name.is_empty() {
            create_open.set(false);
            return;
        }
        let project = ctx.selected_project.get_untracked();
        let parent_path = sv_path_create.get_value();
        let item_type = create_type.get_untracked();
        let toaster = sv_toaster_c.get_value();
        if let Some(project) = project {
            let project_name = project.name.clone();
            let project_name2 = project.name.clone();
            let name_msg = name.clone();
            spawn_local(async move {
                match commands::create_file(&project_name, &parent_path, &item_type, &name).await {
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

    // Selected state
    let path_for_selected = node.path.clone();
    let delete_confirm_name = node.name.clone();

    view! {
        // Node row
        <div
            class="flex items-center gap-1 py-0.5 px-1 rounded-sm cursor-pointer text-sm hover:bg-accent/50 transition-colors"
            class:bg-accent={move || {
                file_ctx.selected_file.get().as_ref().map(|f| f.path == path_for_selected).unwrap_or(false)
            }}
            style:padding-left=padding_left.clone()
            on:click=on_click
            on:contextmenu=move |e: web_sys::MouseEvent| {
                e.prevent_default();
                if is_dir {
                    create_type.set("file".to_string());
                    create_name.set(String::new());
                }
            }
        >
            {chevron}
            {file_icon}
            <span class="truncate select-none">{node.name.clone()}</span>
        </div>

        // Directory children (when expanded)
        {move || {
            if is_dir {
                let expanded = file_ctx.expanded_dirs.get();
                let path_key = node_clone.path.clone();
                if expanded.contains(&path_key) && !children.is_empty() {
                    let child_nodes = children.clone();
                    view! {
                        <FileTree nodes=child_nodes depth=depth + 1 />
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            } else {
                view! {}.into_any()
            }
        }}

        // Rename Modal
        <Show when=move || rename_open.get()>
            <div class="fixed inset-0 z-50 bg-black/50" on:click=move |_| rename_open.set(false) />
            <div class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50 bg-background border rounded-2xl shadow-lg p-6 w-full max-w-md">
                <div class="flex flex-col gap-2 text-left mb-4">
                    <h3 class="text-lg leading-none font-semibold">{tr!("files-rename-title")}</h3>
                </div>
                <Input placeholder=tr!("files-name-placeholder") bind_value=rename_value />
                <div class="flex flex-row gap-2 justify-end mt-4">
                    <Button variant=ButtonVariant::Outline on:click=move |_| rename_open.set(false)>
                        {tr!("action-cancel")}
                    </Button>
                    <Button on:click=move |e| on_rename_confirm.with_value(|f| f(e))>
                        {tr!("action-rename")}
                    </Button>
                </div>
            </div>
        </Show>

        // Delete Confirmation Modal
        <Show when=move || delete_open.get()>
            <div class="fixed inset-0 z-50 bg-black/50" on:click=move |_| delete_open.set(false) />
            <div class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50 bg-background border rounded-2xl shadow-lg p-6 w-full max-w-md">
                <div class="flex flex-col gap-2 text-left mb-4">
                    <h3 class="text-lg leading-none font-semibold">{tr!("files-delete-title")}</h3>
                </div>
                <p class="text-sm text-muted-foreground">
                    {tr!("files-delete-confirm", { "name" => delete_confirm_name.clone() })}
                </p>
                <div class="flex flex-row gap-2 justify-end mt-4">
                    <Button variant=ButtonVariant::Outline on:click=move |_| delete_open.set(false)>
                        {tr!("action-cancel")}
                    </Button>
                    <Button variant=ButtonVariant::Destructive on:click=move |e| on_delete_confirm.with_value(|f| f(e))>
                        {tr!("action-delete")}
                    </Button>
                </div>
            </div>
        </Show>

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
    }
}
