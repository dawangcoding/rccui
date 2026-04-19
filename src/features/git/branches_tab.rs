use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fluent::tr;

use crate::state::{AppContext, GitContext};
use crate::tauri::commands;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::input::Input;
use crate::ui::scroll_area::ScrollArea;
use crate::ui::spinner::Spinner;
use crate::ui::toast_custom::toaster::expect_toaster;

/// Branches sub-tab: list, switch, create, delete branches + remote operations.
#[component]
pub fn BranchesTab() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let git_ctx = expect_context::<GitContext>();
    let toaster = StoredValue::new(expect_toaster());

    // Create branch dialog state
    let create_open = RwSignal::new(false);
    let new_branch_name = RwSignal::new(String::new());

    // Get current branch from status
    let current_branch = move || {
        git_ctx
            .status
            .get()
            .map(|s| s.branch.clone())
            .unwrap_or_default()
    };

    // Remote operations
    let on_fetch = move |_: web_sys::MouseEvent| {
        let project = ctx.selected_project.get_untracked();
        let toaster = toaster.get_value();
        if let Some(project) = project {
            let project_name = project.name.clone();
            let project_name2 = project.name.clone();
            git_ctx.op_loading.set(true);
            spawn_local(async move {
                match commands::git_fetch(&project_name).await {
                    Ok(_) => {
                        toaster.success(tr!("toast-git-fetched"));
                        git_ctx.load_branches(project_name2);
                    }
                    Err(e) => toaster.error(format!("{}: {e}", tr!("toast-git-error"))),
                }
                git_ctx.op_loading.set(false);
            });
        }
    };

    let on_pull = move |_: web_sys::MouseEvent| {
        let project = ctx.selected_project.get_untracked();
        let toaster = toaster.get_value();
        if let Some(project) = project {
            let project_name = project.name.clone();
            let project_name2 = project.name.clone();
            git_ctx.op_loading.set(true);
            spawn_local(async move {
                match commands::git_pull(&project_name).await {
                    Ok(_) => {
                        toaster.success(tr!("toast-git-pulled"));
                        git_ctx.load_status(project_name2);
                    }
                    Err(e) => toaster.error(format!("{}: {e}", tr!("toast-git-error"))),
                }
                git_ctx.op_loading.set(false);
            });
        }
    };

    let on_push = move |_: web_sys::MouseEvent| {
        let project = ctx.selected_project.get_untracked();
        let toaster = toaster.get_value();
        if let Some(project) = project {
            let project_name = project.name.clone();
            git_ctx.op_loading.set(true);
            spawn_local(async move {
                match commands::git_push(&project_name).await {
                    Ok(_) => toaster.success(tr!("toast-git-pushed")),
                    Err(e) => toaster.error(format!("{}: {e}", tr!("toast-git-error"))),
                }
                git_ctx.op_loading.set(false);
            });
        }
    };

    // Create branch
    let on_create_confirm = move |_: web_sys::MouseEvent| {
        let name = new_branch_name.get_untracked();
        if name.trim().is_empty() {
            create_open.set(false);
            return;
        }
        let project = ctx.selected_project.get_untracked();
        let toaster = toaster.get_value();
        if let Some(project) = project {
            let project_name = project.name.clone();
            let project_name2 = project.name.clone();
            let branch_msg = name.clone();
            git_ctx.op_loading.set(true);
            spawn_local(async move {
                match commands::git_create_branch(&project_name, &name).await {
                    Ok(_) => {
                        toaster
                            .success(tr!("toast-git-branch-created", { "branch" => branch_msg }));
                        git_ctx.load_branches(project_name2.clone());
                        git_ctx.load_status(project_name2);
                    }
                    Err(e) => toaster.error(format!("{}: {e}", tr!("toast-git-error"))),
                }
                git_ctx.op_loading.set(false);
            });
        }
        create_open.set(false);
        new_branch_name.set(String::new());
    };

    view! {
        <div class="flex flex-col flex-1 min-h-0">
            // Toolbar
            <div class="flex items-center gap-2 px-3 py-2 border-b border-border shrink-0">
                // Remote status
                {move || {
                    let rs = git_ctx.remote_status.get();
                    match rs {
                        Some(rs) => {
                            let status_text = if !rs.has_remote {
                                tr!("git-no-remote")
                            } else if rs.is_up_to_date {
                                tr!("git-up-to-date")
                            } else {
                                let mut parts = Vec::new();
                                if rs.ahead > 0 {
                                    parts.push(tr!("git-ahead", { "count" => rs.ahead.to_string() }));
                                }
                                if rs.behind > 0 {
                                    parts.push(tr!("git-behind", { "count" => rs.behind.to_string() }));
                                }
                                parts.join(", ")
                            };
                            view! {
                                <span class="text-xs text-muted-foreground mr-auto">{status_text}</span>
                            }.into_any()
                        }
                        None => view! {
                            <span class="text-xs text-muted-foreground mr-auto" />
                        }.into_any(),
                    }
                }}

                <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm on:click=on_fetch>
                    <icons::RefreshCw class="size-3.5 mr-1" />
                    {tr!("git-fetch")}
                </Button>
                <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm on:click=on_pull>
                    <icons::ArrowDown class="size-3.5 mr-1" />
                    {tr!("git-pull")}
                </Button>
                <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm on:click=on_push>
                    <icons::ArrowUp class="size-3.5 mr-1" />
                    {tr!("git-push")}
                </Button>
                <Button variant=ButtonVariant::Outline size=ButtonSize::Sm
                    on:click=move |_| { create_open.set(true); new_branch_name.set(String::new()); }>
                    <icons::GitBranchPlus class="size-3.5 mr-1" />
                    {tr!("git-create-branch")}
                </Button>
            </div>

            // Branch list
            <ScrollArea class="flex-1 min-h-0">
                {move || {
                    let loading = git_ctx.branches_loading.get();
                    let branches = git_ctx.branches.get();
                    let current = current_branch();

                    if loading && branches.is_none() {
                        return view! {
                            <div class="flex items-center justify-center gap-2 py-8 text-sm text-muted-foreground">
                                <Spinner class="size-4" />
                                <span>{tr!("git-loading")}</span>
                            </div>
                        }.into_any();
                    }

                    match branches {
                        None => view! {
                            <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
                                {tr!("git-no-branches")}
                            </div>
                        }.into_any(),
                        Some(b) => {
                            let local = b.local_branches.clone();
                            let remote = b.remote_branches.clone();

                            view! {
                                <div class="py-1">
                                    // Local branches
                                    <div class="px-3 pt-2 pb-1">
                                        <div class="flex items-center gap-1.5 mb-1">
                                            <icons::GitBranch class="size-3.5 text-muted-foreground" />
                                            <span class="text-xs font-medium text-muted-foreground">{tr!("git-local")}</span>
                                            <span class="text-xs text-muted-foreground/60">{local.len().to_string()}</span>
                                        </div>
                                    </div>
                                    {local.into_iter().map(|branch| {
                                        let is_current = branch == current;
                                        let branch_checkout = branch.clone();
                                        let branch_delete = branch.clone();
                                        let branch_display = branch.clone();
                                        let toaster_co = toaster.get_value();
                                        let toaster_del = toaster.get_value();
                                        view! {
                                            <div class={format!(
                                                "flex items-center gap-2 px-4 py-1.5 text-sm hover:bg-accent/50 transition-colors group {}",
                                                if is_current { "bg-accent/30" } else { "" }
                                            )}>
                                                <span class="truncate flex-1">{branch_display}</span>
                                                {if is_current {
                                                    view! {
                                                        <Badge variant=BadgeVariant::Default class="text-[10px] px-1 py-0 shrink-0">
                                                            {tr!("git-current")}
                                                        </Badge>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
                                                            <Button
                                                                variant=ButtonVariant::Ghost
                                                                size=ButtonSize::Icon
                                                                class="size-6"
                                                                on:click=move |_| {
                                                                    let project = ctx.selected_project.get_untracked();
                                                                    let toaster = toaster_co.clone();
                                                                    if let Some(project) = project {
                                                                        let project_name = project.name.clone();
                                                                        let project_name2 = project.name.clone();
                                                                        let branch = branch_checkout.clone();
                                                                        let branch_msg = branch.clone();
                                                                        spawn_local(async move {
                                                                            match commands::git_checkout(&project_name, &branch).await {
                                                                                Ok(_) => {
                                                                                    toaster.success(tr!("toast-git-checked-out", { "branch" => branch_msg }));
                                                                                    git_ctx.load_status(project_name2.clone());
                                                                                    git_ctx.load_branches(project_name2);
                                                                                }
                                                                                Err(e) => toaster.error(format!("{}: {e}", tr!("toast-git-error"))),
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                            >
                                                                <icons::ArrowRightLeft class="size-3" />
                                                            </Button>
                                                            <Button
                                                                variant=ButtonVariant::Ghost
                                                                size=ButtonSize::Icon
                                                                class="size-6 text-destructive hover:text-destructive"
                                                                on:click=move |_| {
                                                                    let project = ctx.selected_project.get_untracked();
                                                                    let toaster = toaster_del.clone();
                                                                    if let Some(project) = project {
                                                                        let project_name = project.name.clone();
                                                                        let project_name2 = project.name.clone();
                                                                        let branch = branch_delete.clone();
                                                                        let branch_msg = branch.clone();
                                                                        spawn_local(async move {
                                                                            match commands::git_delete_branch(&project_name, &branch).await {
                                                                                Ok(_) => {
                                                                                    toaster.success(tr!("toast-git-branch-deleted", { "branch" => branch_msg }));
                                                                                    git_ctx.load_branches(project_name2);
                                                                                }
                                                                                Err(e) => toaster.error(format!("{}: {e}", tr!("toast-git-error"))),
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                            >
                                                                <icons::Trash2 class="size-3" />
                                                            </Button>
                                                        </div>
                                                    }.into_any()
                                                }}
                                            </div>
                                        }
                                    }).collect_view()}

                                    // Remote branches
                                    {if !remote.is_empty() {
                                        view! {
                                            <div class="px-3 pt-3 pb-1">
                                                <div class="flex items-center gap-1.5 mb-1">
                                                    <icons::Globe class="size-3.5 text-muted-foreground" />
                                                    <span class="text-xs font-medium text-muted-foreground">{tr!("git-remote")}</span>
                                                    <span class="text-xs text-muted-foreground/60">{remote.len().to_string()}</span>
                                                </div>
                                            </div>
                                            {remote.into_iter().map(|branch| {
                                                view! {
                                                    <div class="flex items-center gap-2 px-4 py-1.5 text-sm text-muted-foreground">
                                                        <span class="truncate">{branch}</span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        }.into_any()
                                    } else {
                                        view! {}.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </ScrollArea>

            // Create Branch Modal
            <Show when=move || create_open.get()>
                <div class="fixed inset-0 z-50 bg-black/50" on:click=move |_| create_open.set(false) />
                <div class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50 bg-background border rounded-2xl shadow-lg p-6 w-full max-w-md">
                    <h3 class="text-lg leading-none font-semibold mb-4">{tr!("git-create-branch-title")}</h3>
                    <Input placeholder=tr!("git-branch-name-placeholder") bind_value=new_branch_name />
                    <div class="flex flex-row gap-2 justify-end mt-4">
                        <Button variant=ButtonVariant::Outline on:click=move |_| create_open.set(false)>
                            {tr!("action-cancel")}
                        </Button>
                        <Button on:click=on_create_confirm>
                            {tr!("git-create-branch")}
                        </Button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
