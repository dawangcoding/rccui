use leptos::prelude::*;

use icons::FolderOpen;

use crate::layout::sidebar::session_item::ProviderIcon;
use crate::state::AppContext;

/// Dashboard page - shown when no project is selected.
/// Displays a welcome message, provider stats, and project overview.
#[component]
pub fn DashboardPage() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    // Compute provider session stats reactively
    let stats = move || {
        let projects = ctx.projects.get();
        let claude: usize = projects.iter().map(|p| p.sessions.len()).sum();
        let cursor: usize = projects.iter().map(|p| p.cursor_sessions.len()).sum();
        let codex: usize = projects.iter().map(|p| p.codex_sessions.len()).sum();
        let gemini: usize = projects.iter().map(|p| p.gemini_sessions.len()).sum();
        (claude, cursor, codex, gemini)
    };

    view! {
        <div class="flex flex-col items-center justify-center h-full p-8 gap-8">
            // Header section
            <div class="flex flex-col items-center gap-2 text-center max-w-md">
                <h1 class="text-3xl font-bold tracking-tight text-foreground">"RCCUI"</h1>
                <p class="text-base text-muted-foreground">
                    "Multi AI Coding Assistant Manager"
                </p>
            </div>

            // Provider stats grid
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 w-full max-w-lg">
                {move || {
                    let (claude, cursor, codex, gemini) = stats();
                    view! {
                        <ProviderCard provider="claude" label="Claude" count=claude color_class="text-provider-claude" />
                        <ProviderCard provider="cursor" label="Cursor" count=cursor color_class="text-foreground" />
                        <ProviderCard provider="codex" label="Codex" count=codex color_class="text-provider-codex" />
                        <ProviderCard provider="gemini" label="Gemini" count=gemini color_class="text-provider-gemini" />
                    }
                }}
            </div>

            // Bottom section
            <div class="text-center">
                {move || {
                    let count = ctx.projects.get().len();
                    if count == 0 {
                        view! {
                            <div class="flex flex-col items-center gap-3 px-4">
                                <div class="size-12 rounded-lg bg-muted flex items-center justify-center">
                                    <FolderOpen class="size-6 text-muted-foreground" />
                                </div>
                                <p class="text-sm font-medium text-foreground">"No projects yet"</p>
                                <p class="text-xs text-muted-foreground max-w-sm">
                                    "Sessions will appear automatically as you use Claude, Cursor, Codex, or Gemini in your projects."
                                </p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <p class="text-sm text-muted-foreground">
                                {format!("{count} project(s) found. Select one from the sidebar to get started.")}
                            </p>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// A single provider stat card for the dashboard.
#[component]
fn ProviderCard(
    provider: &'static str,
    label: &'static str,
    count: usize,
    color_class: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center gap-2 rounded-lg bg-muted/50 p-4">
            <span class="size-5 flex items-center justify-center">
                <ProviderIcon provider=provider.to_string() />
            </span>
            <span class="text-xs text-muted-foreground uppercase tracking-wider">{label}</span>
            <span class=format!("text-2xl font-bold tabular-nums {color_class}")>{count}</span>
        </div>
    }
}
