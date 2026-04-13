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
        <div class="flex flex-col items-center justify-center h-full p-8 gap-10 relative overflow-hidden">
            // Decorative background glow orbs — Raycast Blue
            <div class="absolute inset-0 overflow-hidden pointer-events-none select-none" aria-hidden="true">
                <div class="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full opacity-[0.04]"
                    style="background: radial-gradient(circle, oklch(0.73 0.14 240) 0%, transparent 70%);"
                />
                <div class="absolute bottom-1/4 right-1/4 w-[300px] h-[300px] rounded-full opacity-[0.03]"
                    style="background: radial-gradient(circle, oklch(0.68 0.12 260) 0%, transparent 70%);"
                />
            </div>

            // Header section
            <div class="flex flex-col items-center gap-3 text-center max-w-md relative z-10">
                <h1 class="text-4xl font-bold tracking-tight gradient-text">"RCCUI"</h1>
                <p class="text-sm text-muted-foreground leading-relaxed">
                    "Multi AI Coding Assistant Manager"
                </p>
            </div>

            // Provider stats grid
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 w-full max-w-lg relative z-10">
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
            <div class="text-center relative z-10">
                {move || {
                    let count = ctx.projects.get().len();
                    if count == 0 {
                        view! {
                            <div class="flex flex-col items-center gap-3 px-4">
                                <div class="size-12 rounded-xl bg-muted/50 border border-border flex items-center justify-center">
                                    <FolderOpen class="size-5 text-muted-foreground" />
                                </div>
                                <p class="text-sm font-medium text-foreground">"No projects yet"</p>
                                <p class="text-xs text-muted-foreground max-w-sm leading-relaxed">
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
        <div class="group flex flex-col items-center gap-2.5 rounded-xl border border-border bg-card/50 p-5 transition-all duration-300 hover:border-primary/20 hover:bg-card">
            <span class="size-5 flex items-center justify-center opacity-80 group-hover:opacity-100 transition-opacity">
                <ProviderIcon provider=provider.to_string() />
            </span>
            <span class="text-[11px] text-muted-foreground uppercase tracking-widest font-medium">{label}</span>
            <span class=format!("text-2xl font-semibold tabular-nums tracking-tight {color_class}")>{count}</span>
        </div>
    }
}
