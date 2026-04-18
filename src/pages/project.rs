use leptos::prelude::*;
use leptos_fluent::tr;

use crate::features::chat::chat_panel::ChatPanel;
use crate::features::shell::shell_panel::ShellPanel;
use crate::state::AppContext;
use crate::tauri::types::AppTab;

/// Project view page - shows the active tab content for the selected project.
#[component]
pub fn ProjectPage() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    view! {
        <div class="flex flex-col h-full min-w-0 overflow-hidden">
            {move || {
                let tab = ctx.active_tab.get();
                let project = ctx.selected_project.get();

                match project {
                    None => view! {
                        <div class="flex items-center justify-center h-full text-sm text-muted-foreground animate-in fade-in duration-150">
                            {tr!("project-select-prompt")}
                        </div>
                    }.into_any(),
                    Some(_project) => {
                        match tab {
                            AppTab::Chat => view! {
                                <ChatPanel />
                            }.into_any(),
                            AppTab::Shell => view! {
                                <ShellPanel />
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
