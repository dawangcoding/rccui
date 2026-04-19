use leptos::prelude::*;
use leptos_fluent::tr;

use crate::state::git_state::GitTab;
use crate::state::{AppContext, GitContext};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::spinner::Spinner;

use super::branches_tab::BranchesTab;
use super::changes_tab::ChangesTab;
use super::history_tab::HistoryTab;

/// Main Git panel with Changes/History/Branches sub-tabs.
#[component]
pub fn GitPanel() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let git_ctx = expect_context::<GitContext>();

    // Load data when project changes
    Effect::new(move || {
        let project = ctx.selected_project.get();
        if let Some(project) = project {
            git_ctx.clear();
            git_ctx.load_status(project.name.clone());
            git_ctx.load_commits(project.name.clone());
            git_ctx.load_branches(project.name.clone());
        } else {
            git_ctx.clear();
        }
    });

    let tab_class = move |tab: GitTab| {
        let active = git_ctx.active_tab.get() == tab;
        format!(
            "px-3 py-1.5 text-xs font-medium rounded-md transition-colors {}",
            if active {
                "bg-primary text-primary-foreground shadow-sm"
            } else {
                "text-muted-foreground hover:text-foreground hover:bg-accent/50"
            }
        )
    };

    view! {
        <div class="flex flex-col flex-1 min-h-0 min-w-0 overflow-hidden">
            // Header: branch info + sub-tab switcher + actions
            <div class="flex items-center gap-2 px-3 py-2 border-b border-border shrink-0">
                // Branch info
                <div class="flex items-center gap-1.5 text-sm font-medium text-foreground mr-2">
                    <icons::GitBranch class="size-4 text-muted-foreground" />
                    {move || {
                        let status = git_ctx.status.get();
                        match status {
                            Some(s) => s.branch,
                            None => String::new(),
                        }
                    }}
                </div>

                // Change count
                {move || {
                    let status = git_ctx.status.get();
                    if let Some(s) = status {
                        let count = s.modified.len() + s.added.len() + s.deleted.len() + s.untracked.len();
                        if count > 0 {
                            return view! {
                                <span class="text-xs text-muted-foreground">
                                    {format!("{count} {}", tr!("git-files-changed"))}
                                </span>
                            }.into_any();
                        }
                    }
                    view! {}.into_any()
                }}

                <div class="ml-auto flex items-center gap-1">
                    // Sub-tab switcher
                    <div class="flex items-center gap-0.5 p-0.5 rounded-lg bg-muted/50">
                        <button class={move || tab_class(GitTab::Changes)} on:click=move |_| git_ctx.active_tab.set(GitTab::Changes)>
                            {tr!("git-changes")}
                        </button>
                        <button class={move || tab_class(GitTab::History)} on:click=move |_| {
                            git_ctx.active_tab.set(GitTab::History);
                            git_ctx.clear_diff();
                        }>
                            {tr!("git-history")}
                        </button>
                        <button class={move || tab_class(GitTab::Branches)} on:click=move |_| git_ctx.active_tab.set(GitTab::Branches)>
                            {tr!("git-branches")}
                        </button>
                    </div>

                    // Refresh button
                    <Button
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Icon
                        class="size-7"
                        on:click=move |_| {
                            if let Some(project) = ctx.selected_project.get_untracked() {
                                git_ctx.load_status(project.name.clone());
                                git_ctx.load_commits(project.name.clone());
                                git_ctx.load_branches(project.name.clone());
                            }
                        }
                    >
                        <icons::RefreshCw class="size-4" />
                    </Button>

                    // Loading indicator
                    {move || {
                        if git_ctx.status_loading.get() || git_ctx.op_loading.get() {
                            view! { <Spinner class="size-4" /> }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                </div>
            </div>

            // Error banner
            {move || {
                let err = git_ctx.error_message.get();
                if let Some(err) = err {
                    view! {
                        <div class="px-3 py-2 text-sm text-destructive bg-destructive/10 border-b border-border">
                            {err}
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}

            // Tab content
            {move || {
                let project = ctx.selected_project.get();
                if project.is_none() {
                    return view! {
                        <div class="flex items-center justify-center flex-1 text-sm text-muted-foreground">
                            {tr!("git-empty-state")}
                        </div>
                    }.into_any();
                }

                match git_ctx.active_tab.get() {
                    GitTab::Changes => view! { <ChangesTab /> }.into_any(),
                    GitTab::History => view! { <HistoryTab /> }.into_any(),
                    GitTab::Branches => view! { <BranchesTab /> }.into_any(),
                }
            }}
        </div>
    }
}
