use leptos::prelude::*;

use crate::state::AppContext;

/// Dashboard page - shown when no project is selected.
/// Displays a welcome message and the project list overview.
#[component]
pub fn DashboardPage() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    view! {
        <div class="flex flex-col items-center justify-center h-full p-8 gap-6">
            <div class="flex flex-col items-center gap-2 text-center max-w-md">
                <h1 class="text-2xl font-bold text-foreground">"RCCUI"</h1>
                <p class="text-sm text-muted-foreground">
                    "Multi AI Coding Assistant Manager"
                </p>
            </div>

            <div class="text-center text-sm text-muted-foreground">
                {move || {
                    let count = ctx.projects.get().len();
                    if count == 0 {
                        "No projects discovered yet. Sessions will appear as you use Claude, Cursor, Codex, or Gemini.".to_string()
                    } else {
                        format!("{count} project(s) found. Select one from the sidebar to get started.")
                    }
                }}
            </div>
        </div>
    }
}
