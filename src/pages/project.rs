use leptos::prelude::*;

use crate::state::AppContext;
use crate::tauri::types::AppTab;

/// Project view page - shows the active tab content for the selected project.
#[component]
pub fn ProjectPage() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    view! {
        <div class="flex flex-col h-full overflow-hidden">
            {move || {
                let tab = ctx.active_tab.get();
                let project = ctx.selected_project.get();

                match project {
                    None => view! {
                        <div class="flex items-center justify-center h-full text-sm text-muted-foreground">
                            "Select a project from the sidebar"
                        </div>
                    }.into_any(),
                    Some(project) => {
                        let selected_session = ctx.selected_session.get();
                        let session_display = selected_session
                            .as_ref()
                            .map(|s| {
                                s.name
                                    .clone()
                                    .unwrap_or_else(|| s.summary.clone())
                            })
                            .unwrap_or_default();

                        match tab {
                            AppTab::Chat => view! {
                                <div class="flex flex-col items-center justify-center h-full gap-3 text-sm text-muted-foreground">
                                    <div class="text-center">
                                        <p class="font-medium text-foreground text-base">{project.display_name.clone()}</p>
                                        {if !session_display.is_empty() {
                                            Some(view! {
                                                <p class="mt-1 text-xs text-muted-foreground max-w-sm truncate">
                                                    "Session: " {session_display}
                                                </p>
                                            })
                                        } else {
                                            None
                                        }}
                                        <p class="mt-3 text-xs">"Chat - Coming in Phase 2"</p>
                                    </div>
                                </div>
                            }.into_any(),
                            AppTab::Shell => view! {
                                <div class="flex items-center justify-center h-full text-sm text-muted-foreground">
                                    "Shell - Coming in Phase 3"
                                </div>
                            }.into_any(),
                            AppTab::Files => view! {
                                <div class="flex items-center justify-center h-full text-sm text-muted-foreground">
                                    "Files - Coming in Phase 4"
                                </div>
                            }.into_any(),
                            AppTab::Git => view! {
                                <div class="flex items-center justify-center h-full text-sm text-muted-foreground">
                                    "Git - Coming in Phase 5"
                                </div>
                            }.into_any(),
                        }
                    }
                }
            }}
        </div>
    }
}
