use leptos::prelude::*;
use leptos_fluent::tr;

use crate::state::{AppContext, GitContext};
use crate::ui::scroll_area::ScrollArea;
use crate::ui::spinner::Spinner;

use super::diff_viewer::DiffViewer;

/// History sub-tab: shows commit log with expandable diffs.
#[component]
pub fn HistoryTab() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let git_ctx = expect_context::<GitContext>();

    view! {
        <div class="flex flex-1 min-h-0">
            // Left: commit list
            <div class="flex flex-col w-80 shrink-0 border-r border-border min-h-0">
                <ScrollArea class="flex-1 min-h-0">
                    {move || {
                        let loading = git_ctx.commits_loading.get();
                        let commits = git_ctx.commits.get();

                        if loading && commits.is_empty() {
                            return view! {
                                <div class="flex items-center justify-center gap-2 py-8 text-sm text-muted-foreground">
                                    <Spinner class="size-4" />
                                    <span>{tr!("git-loading")}</span>
                                </div>
                            }.into_any();
                        }

                        if commits.is_empty() {
                            return view! {
                                <div class="flex flex-col items-center justify-center gap-2 py-8 text-muted-foreground">
                                    <icons::GitCommitHorizontal class="size-6 opacity-30" />
                                    <p class="text-sm">{tr!("git-no-commits")}</p>
                                </div>
                            }.into_any();
                        }

                        let project_name = ctx.selected_project.get_untracked().map(|p| p.name.clone()).unwrap_or_default();

                        view! {
                            <div class="py-1">
                                {commits.into_iter().map(|commit| {
                                    let hash = commit.hash.clone();
                                    let short_hash = hash[..7.min(hash.len())].to_string();
                                    let message = commit.message.clone();
                                    let author = commit.author.clone();
                                    let date = format_commit_date(&commit.date);
                                    let stats = commit.stats.clone();
                                    let hash_click = hash.clone();
                                    let hash_selected = hash.clone();
                                    let project_name = project_name.clone();

                                    let is_selected = move || {
                                        git_ctx.selected_commit.get().as_deref() == Some(hash_selected.as_str())
                                    };

                                    view! {
                                        <button
                                            class={move || format!(
                                                "flex flex-col gap-0.5 w-full px-3 py-2 text-left border-b border-border/50 hover:bg-accent/50 transition-colors {}",
                                                if is_selected() { "bg-accent" } else { "" }
                                            )}
                                            on:click={
                                                let project_name = project_name.clone();
                                                let hash_click = hash_click.clone();
                                                move |_| {
                                                    git_ctx.load_commit_diff(project_name.clone(), hash_click.clone());
                                                }
                                            }
                                        >
                                            <div class="flex items-center gap-2">
                                                <span class="text-xs font-mono text-primary">{short_hash}</span>
                                                {stats.map(|s| view! {
                                                    <span class="text-[10px] text-muted-foreground ml-auto">{s}</span>
                                                })}
                                            </div>
                                            <p class="text-sm truncate">{message}</p>
                                            <div class="flex items-center gap-2 text-[11px] text-muted-foreground">
                                                <span>{author}</span>
                                                <span class="ml-auto">{date}</span>
                                            </div>
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }}
                </ScrollArea>
            </div>

            // Right: commit diff viewer
            <div class="flex flex-col flex-1 min-w-0 min-h-0">
                <ScrollArea class="flex-1 min-h-0">
                    {move || {
                        let loading = git_ctx.diff_loading.get();
                        let diff = git_ctx.commit_diff_content.get();
                        let selected = git_ctx.selected_commit.get();

                        if selected.is_none() {
                            return view! {
                                <div class="flex items-center justify-center h-full text-sm text-muted-foreground py-12">
                                    {tr!("git-select-commit-diff")}
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
    }
}

/// Format ISO date to a short display form.
fn format_commit_date(iso: &str) -> String {
    // Just take the date part (YYYY-MM-DD) from ISO 8601
    if iso.len() >= 10 {
        iso[..10].to_string()
    } else {
        iso.to_string()
    }
}
