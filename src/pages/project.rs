use leptos::prelude::*;
use leptos_fluent::tr;

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
                        <div class="flex items-center justify-center h-full text-sm text-muted-foreground animate-in fade-in duration-150">
                            {tr!("project-select-prompt")}
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
                                <div class="flex flex-col items-center justify-center h-full gap-3 text-sm text-muted-foreground animate-in fade-in duration-150">
                                    <div class="text-center">
                                        <p class="font-medium text-foreground text-base">{project.display_name.clone()}</p>
                                        {if !session_display.is_empty() {
                                            Some(view! {
                                                <p class="mt-1 text-xs text-muted-foreground max-w-sm truncate">
                                                    {tr!("project-session-label")} {session_display}
                                                </p>
                                            })
                                        } else {
                                            None
                                        }}
                                        <p class="mt-3 text-xs">{tr!("project-chat-coming")}</p>
                                    </div>
                                </div>
                            }.into_any(),
                            AppTab::Shell => view! {
                                <div class="flex items-center justify-center h-full text-sm text-muted-foreground animate-in fade-in duration-150">
                                    {tr!("project-shell-coming")}
                                </div>
                            }.into_any(),
                            AppTab::Files => view! {
                                <div class="flex items-center justify-center h-full text-sm text-muted-foreground animate-in fade-in duration-150">
                                    {tr!("project-files-coming")}
                                </div>
                            }.into_any(),
                            AppTab::Git => view! {
                                <div class="flex items-center justify-center h-full text-sm text-muted-foreground animate-in fade-in duration-150">
                                    {tr!("project-git-coming")}
                                </div>
                            }.into_any(),
                        }
                    }
                }
            }}
        </div>
    }
}
