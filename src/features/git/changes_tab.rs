use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fluent::tr;

use crate::state::{AppContext, GitContext};
use crate::tauri::commands;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize};
use crate::ui::input::Input;
use crate::ui::scroll_area::ScrollArea;
use crate::ui::spinner::Spinner;
use crate::ui::toast_custom::toaster::expect_toaster;

use super::diff_viewer::DiffViewer;

/// Changes sub-tab: shows git status (modified/added/deleted/untracked files),
/// allows viewing diffs and committing.
#[component]
pub fn ChangesTab() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let git_ctx = expect_context::<GitContext>();
    let commit_message = RwSignal::new(String::new());
    let toaster = StoredValue::new(expect_toaster());

    // Commit handler
    let on_commit = move |_: web_sys::MouseEvent| {
        let msg = commit_message.get_untracked();
        if msg.trim().is_empty() {
            return;
        }
        let project = ctx.selected_project.get_untracked();
        let toaster = toaster.get_value();
        if let Some(project) = project {
            let project_name = project.name.clone();
            let project_name2 = project.name.clone();
            let has_commits = git_ctx
                .status
                .get_untracked()
                .map(|s| s.has_commits)
                .unwrap_or(true);
            git_ctx.op_loading.set(true);
            spawn_local(async move {
                let result = if has_commits {
                    commands::git_commit(&project_name, &msg, vec![]).await
                } else {
                    commands::git_initial_commit(&project_name, &msg, vec![]).await
                };
                match result {
                    Ok(_) => {
                        toaster.success(tr!("toast-git-committed"));
                        commit_message.set(String::new());
                        git_ctx.load_status(project_name2);
                    }
                    Err(e) => {
                        toaster.error(format!("{}: {e}", tr!("toast-git-error")));
                    }
                }
                git_ctx.op_loading.set(false);
            });
        }
    };

    view! {
        <div class="flex flex-col flex-1 min-h-0">
            // Commit input area
            <div class="flex items-center gap-2 px-3 py-2 border-b border-border shrink-0">
                <Input
                    placeholder=tr!("git-commit-placeholder")
                    bind_value=commit_message
                    class="flex-1 h-8 text-sm"
                />
                <Button
                    size=ButtonSize::Sm
                    on:click=on_commit
                    attr:disabled=move || {
                        if commit_message.get().trim().is_empty() || git_ctx.op_loading.get() {
                            Some(true)
                        } else {
                            None
                        }
                    }
                >
                    {move || {
                        let status = git_ctx.status.get();
                        if status.as_ref().is_some_and(|s| !s.has_commits) {
                            tr!("git-initial-commit")
                        } else {
                            tr!("git-commit-all")
                        }
                    }}
                </Button>
            </div>

            // File list + diff split view
            <div class="flex flex-1 min-h-0">
                // Left: file list
                <div class="flex flex-col w-64 shrink-0 border-r border-border min-h-0">
                    <ScrollArea class="flex-1 min-h-0">
                        {move || {
                            let loading = git_ctx.status_loading.get();
                            let status = git_ctx.status.get();

                            if loading && status.is_none() {
                                return view! {
                                    <div class="flex items-center justify-center gap-2 py-8 text-sm text-muted-foreground">
                                        <Spinner class="size-4" />
                                        <span>{tr!("git-loading")}</span>
                                    </div>
                                }.into_any();
                            }

                            match status {
                                None => view! {
                                    <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
                                        {tr!("git-no-changes")}
                                    </div>
                                }.into_any(),
                                Some(s) => {
                                    let total = s.modified.len() + s.added.len() + s.deleted.len() + s.untracked.len();
                                    if total == 0 {
                                        return view! {
                                            <div class="flex flex-col items-center justify-center gap-2 py-8 text-muted-foreground">
                                                <icons::Check class="size-6 text-success" />
                                                <p class="text-sm">{tr!("git-no-changes")}</p>
                                            </div>
                                        }.into_any();
                                    }

                                    let project_name = ctx.selected_project.get_untracked().map(|p| p.name.clone()).unwrap_or_default();

                                    let render_section = |files: Vec<String>, label: String, variant: BadgeVariant, icon_class: &'static str| {
                                        if files.is_empty() {
                                            return view! {}.into_any();
                                        }
                                        let project_name = project_name.clone();
                                        view! {
                                            <div class="px-2 pt-2 pb-1">
                                                <div class="flex items-center gap-1.5 px-1 mb-1">
                                                    <Badge variant=variant class="text-[10px] px-1 py-0">
                                                        {label}
                                                    </Badge>
                                                    <span class="text-xs text-muted-foreground">{files.len().to_string()}</span>
                                                </div>
                                                {files.into_iter().map(|file| {
                                                    let file_clone = file.clone();
                                                    let file_display = file.clone();
                                                    let project_name = project_name.clone();
                                                    let is_selected = {
                                                        let file_clone = file_clone.clone();
                                                        move || git_ctx.selected_diff_file.get().as_deref() == Some(file_clone.as_str())
                                                    };
                                                    view! {
                                                        <button
                                                            class={move || format!(
                                                                "flex items-center gap-2 w-full px-2 py-1 text-xs text-left rounded-md hover:bg-accent/50 transition-colors {}",
                                                                if is_selected() { "bg-accent text-accent-foreground" } else { "" }
                                                            )}
                                                            on:click={
                                                                let project_name = project_name.clone();
                                                                let file_clone = file_clone.clone();
                                                                move |_| {
                                                                    git_ctx.load_file_diff(project_name.clone(), file_clone.clone());
                                                                }
                                                            }
                                                        >
                                                            <span class={format!("size-1.5 rounded-full shrink-0 {icon_class}")} />
                                                            <span class="truncate">{file_display}</span>
                                                        </button>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    };

                                    let modified = s.modified.clone();
                                    let added = s.added.clone();
                                    let deleted = s.deleted.clone();
                                    let untracked = s.untracked.clone();

                                    view! {
                                        <div>
                                            {render_section(modified, tr!("git-modified"), BadgeVariant::Warning, "bg-warning")}
                                            {render_section(added, tr!("git-added"), BadgeVariant::Success, "bg-success")}
                                            {render_section(deleted, tr!("git-deleted"), BadgeVariant::Destructive, "bg-destructive")}
                                            {render_section(untracked, tr!("git-untracked"), BadgeVariant::Secondary, "bg-muted-foreground")}
                                        </div>
                                    }.into_any()
                                }
                            }
                        }}
                    </ScrollArea>
                </div>

                // Right: diff viewer
                <div class="flex flex-col flex-1 min-w-0 min-h-0">
                    <ScrollArea class="flex-1 min-h-0">
                        {move || {
                            let loading = git_ctx.diff_loading.get();
                            let diff = git_ctx.diff_content.get();
                            let selected = git_ctx.selected_diff_file.get();

                            if selected.is_none() {
                                return view! {
                                    <div class="flex items-center justify-center h-full text-sm text-muted-foreground py-12">
                                        {tr!("git-select-file-diff")}
                                    </div>
                                }.into_any();
                            }

                            if loading {
                                return view! {
                                    <div class="flex items-center justify-center gap-2 py-8 text-sm text-muted-foreground">
                                        <Spinner class="size-4" />
                                        <span>{tr!("git-loading")}</span>
                                    </div>
                                }.into_any();
                            }

                            match diff {
                                Some(d) if !d.is_empty() => {
                                    view! { <DiffViewer diff=d /> }.into_any()
                                }
                                _ => view! {
                                    <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
                                        {tr!("git-no-changes")}
                                    </div>
                                }.into_any(),
                            }
                        }}
                    </ScrollArea>
                </div>
            </div>
        </div>
    }
}
