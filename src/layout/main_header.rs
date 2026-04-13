use leptos::prelude::*;

use icons::{MessageSquare, SquareTerminal, FolderOpen, GitBranch};

use crate::state::AppContext;
use crate::tauri::types::AppTab;
use crate::ui::sidenav::SidenavTrigger;
use crate::ui::tooltip::{Tooltip, TooltipContent, TooltipPosition};

/// Tab icon component matching each AppTab variant.
#[component]
fn TabIcon(tab: AppTab) -> impl IntoView {
    match tab {
        AppTab::Chat => view! { <MessageSquare class="size-3.5" /> }.into_any(),
        AppTab::Shell => view! { <SquareTerminal class="size-3.5" /> }.into_any(),
        AppTab::Files => view! { <FolderOpen class="size-3.5" /> }.into_any(),
        AppTab::Git => view! { <GitBranch class="size-3.5" /> }.into_any(),
    }
}

/// Header bar for the main content area.
/// Shows: [SidenavTrigger] [Project Name] [Tab Switcher]
#[component]
pub fn MainHeader() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    let project_display_name = move || {
        ctx.selected_project
            .get()
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| "Select a project".to_string())
    };

    let tabs = [AppTab::Chat, AppTab::Shell, AppTab::Files, AppTab::Git];

    view! {
        <header class="flex items-center gap-3 h-12 px-4 border-b border-border shrink-0">
            // Sidebar toggle
            <SidenavTrigger>
                <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-muted-foreground">
                    <rect width="18" height="18" x="3" y="3" rx="2"/>
                    <path d="M9 3v18"/>
                </svg>
            </SidenavTrigger>

            // Project name
            <span class="text-sm font-medium text-foreground truncate max-w-48">
                {project_display_name}
            </span>

            // Spacer
            <div class="flex-1"/>

            // Tab buttons
            <nav class="flex items-center gap-0.5 bg-muted/60 rounded-lg p-0.5 h-8 border border-border/50">
                {tabs.into_iter().map(|tab| {
                    let label = tab.label();
                    let is_active = Memo::new(move |_| ctx.active_tab.get() == tab);
                    let implemented = tab.is_implemented();

                    let btn = view! {
                        <button
                            class=move || {
                                let base = "flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-medium rounded-md transition-all duration-200 cursor-pointer select-none";
                                if is_active.get() {
                                    format!("{base} bg-accent text-foreground")
                                } else if implemented {
                                    format!("{base} text-muted-foreground hover:text-foreground hover:bg-accent/50")
                                } else {
                                    format!("{base} text-muted-foreground/50 hover:text-muted-foreground")
                                }
                            }
                            on:click=move |_| ctx.active_tab.set(tab)
                        >
                            <TabIcon tab=tab />
                            {label}
                        </button>
                    };

                    if implemented {
                        view! { <span>{btn}</span> }.into_any()
                    } else {
                        view! {
                            <Tooltip>
                                {btn}
                                <TooltipContent position=TooltipPosition::Bottom>
                                    "Coming soon"
                                </TooltipContent>
                            </Tooltip>
                        }.into_any()
                    }
                }).collect_view()}
            </nav>
        </header>
    }
}
