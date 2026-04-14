use leptos::prelude::*;
use leptos_fluent::tr;

use crate::tauri::types::SessionInfo;

/// Renders a single session entry in the sidebar.
/// Shows provider icon, session title/summary, message count, and relative timestamp.
#[component]
pub fn SessionItem(
    session: SessionInfo,
    is_selected: Memo<bool>,
    #[prop(into)] on_click: Callback<()>,
    #[prop(into)] on_delete: Callback<String>,
    #[prop(into)] on_rename: Callback<(String, String)>,
) -> impl IntoView {
    let session_id = session.id.clone();
    let session_id_del = session_id.clone();
    let session_id_ren = session_id.clone();
    let provider = session.provider.clone();
    let summary = session
        .name
        .clone()
        .unwrap_or_else(|| session.summary.clone());
    let message_count = session.message_count;
    let last_activity = session.last_activity.clone();
    let summary_for_rename = summary.clone();

    let time_data = parse_relative_time(&last_activity);

    view! {
        <li class="group/session relative min-w-0">
            <button
                class=move || {
                    let base = "flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-xs transition-all duration-200 cursor-pointer select-none";
                    if is_selected.get() {
                        format!("{base} bg-sidenav-accent text-sidenav-accent-foreground font-medium")
                    } else {
                        format!("{base} text-sidenav-foreground/70 hover:bg-sidenav-accent/50 hover:text-sidenav-accent-foreground")
                    }
                }
                on:click=move |_| on_click.run(())
            >
                // Provider icon
                <span class="flex-shrink-0 size-4 flex items-center justify-center">
                    <ProviderIcon provider=provider.clone() />
                </span>

                // Session summary
                <span class="flex-1 truncate min-w-0">
                    {summary.clone()}
                </span>

                // Message count + time
                <span class="flex-shrink-0 flex items-center gap-1.5 text-[10px] text-muted-foreground tabular-nums">
                    {if message_count > 0 {
                        Some(view! {
                            <span class="inline-flex items-center justify-center min-w-4 h-4 px-1 rounded-full bg-muted/80 text-muted-foreground text-[10px] font-medium">
                                {message_count}
                            </span>
                        })
                    } else {
                        None
                    }}
                    <span>{move || match time_data {
                        ("time-just-now", _) => tr!("time-just-now"),
                        ("time-minutes", Some(v)) => tr!("time-minutes", { "value" => v }),
                        ("time-hours", Some(v)) => tr!("time-hours", { "value" => v }),
                        ("time-days", Some(v)) => tr!("time-days", { "value" => v }),
                        ("time-weeks", Some(v)) => tr!("time-weeks", { "value" => v }),
                        _ => tr!("time-just-now"),
                    }}</span>
                </span>
            </button>

            // Action buttons (visible on hover)
            <div class="absolute top-0.5 right-1 hidden group-hover/session:flex items-center gap-0.5 rounded-md px-0.5" style="background: var(--sidenav); opacity: 0.95;">
                // Rename button
                <button
                    class="size-5 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-all duration-150"
                    title={move || tr!("action-rename")}
                    on:click=move |e| {
                        e.stop_propagation();
                        on_rename.run((session_id_ren.clone(), summary_for_rename.clone()));
                    }
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/>
                    </svg>
                </button>
                // Delete button
                <button
                    class="size-5 flex items-center justify-center rounded-md text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-all duration-150"
                    title={move || tr!("action-delete")}
                    on:click=move |e| {
                        e.stop_propagation();
                        on_delete.run(session_id_del.clone());
                    }
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
                        <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
                    </svg>
                </button>
            </div>
        </li>
    }
}

/// Provider icon SVG based on provider string.
#[component]
pub fn ProviderIcon(provider: String) -> impl IntoView {
    match provider.as_str() {
        "claude" => view! {
            <svg viewBox="0 0 24 24" class="size-3.5 text-provider-claude" fill="currentColor">
                <path d="M16.1 11.3c0-3.2-2.5-5.3-5.3-5.3a5.5 5.5 0 0 0-5.5 5.5c0 3 2.3 5.5 5.5 5.5 2.8 0 5.3-2.1 5.3-5.3V11.3zM12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2z"/>
            </svg>
        }.into_any(),
        "cursor" => view! {
            <svg viewBox="0 0 24 24" class="size-3.5 text-foreground" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2"/>
                <path d="M8 12h8"/>
            </svg>
        }.into_any(),
        "codex" => view! {
            <svg viewBox="0 0 24 24" class="size-3.5 text-provider-codex" fill="currentColor">
                <circle cx="12" cy="12" r="10"/>
                <path d="M12 8v8M8 12h8" stroke="white" stroke-width="2"/>
            </svg>
        }.into_any(),
        "gemini" => view! {
            <svg viewBox="0 0 24 24" class="size-3.5 text-provider-gemini" fill="currentColor">
                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
            </svg>
        }.into_any(),
        _ => view! {
            <svg viewBox="0 0 24 24" class="size-3.5 text-muted-foreground" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
            </svg>
        }.into_any(),
    }
}

/// Parse an ISO timestamp string into a translation key and optional value.
fn parse_relative_time(timestamp: &str) -> (&'static str, Option<i64>) {
    let now = js_sys::Date::new_0();
    let now_ms = now.get_time();

    let parsed = js_sys::Date::new(&timestamp.into());
    let parsed_ms = parsed.get_time();

    if parsed_ms.is_nan() {
        return ("time-just-now", None);
    }

    let diff_ms = now_ms - parsed_ms;
    let diff_secs = (diff_ms / 1000.0) as i64;

    if diff_secs < 60 {
        ("time-just-now", None)
    } else if diff_secs < 3600 {
        ("time-minutes", Some(diff_secs / 60))
    } else if diff_secs < 86400 {
        ("time-hours", Some(diff_secs / 3600))
    } else if diff_secs < 604800 {
        ("time-days", Some(diff_secs / 86400))
    } else {
        ("time-weeks", Some(diff_secs / 604800))
    }
}
