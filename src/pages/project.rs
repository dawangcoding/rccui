use leptos::prelude::*;
use leptos_fluent::tr;

use crate::features::chat::chat_panel::ChatPanel;
use crate::features::files::files_panel::FilesPanel;
use crate::features::git::git_panel::GitPanel;
use crate::features::shell::shell_panel::ShellPanel;
use crate::state::{AppContext, FileContext};
use crate::tauri::types::AppTab;

/// Project view page - shows the active tab content for the selected project.
#[component]
pub fn ProjectPage() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let file_ctx = expect_context::<FileContext>();

    // Clear file context when project changes
    Effect::new(move || {
        let _project = ctx.selected_project.get();
        file_ctx.close_editor();
    });

    view! {
        <div class="flex flex-col flex-1 min-h-0 min-w-0 overflow-hidden">
            {move || {
                let tab = ctx.active_tab.get();
                let project = ctx.selected_project.get();

                match project {
                    None => view! {
                        <div class="flex items-center justify-center flex-1 text-sm text-muted-foreground animate-in fade-in duration-150">
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
                                <FilesPanel />
                            }.into_any(),
                            AppTab::Git => view! {
                                <GitPanel />
                            }.into_any(),
                        }
                    }
                }
            }}
        </div>
    }
}
