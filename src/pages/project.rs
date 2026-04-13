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
                        match tab {
                            AppTab::Chat => view! {
                                <div class="flex items-center justify-center h-full text-sm text-muted-foreground">
                                    <div class="text-center">
                                        <p class="font-medium text-foreground">{project.display_name.clone()}</p>
                                        <p class="mt-1">"Chat - Coming in Phase 2"</p>
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
