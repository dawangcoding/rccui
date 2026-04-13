use leptos::prelude::*;
use leptos::task::spawn_local;

use icons::Search;

use crate::state::{AppContext, SessionContext};
use crate::tauri::commands;
use crate::tauri::types::{Project, SessionInfo};
use crate::ui::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
use crate::ui::skeleton::Skeleton;
use crate::ui::toast_custom::toaster::expect_toaster;

use super::session_item::SessionItem;

/// Project list in the sidebar. Each project can be expanded to show its sessions.
#[component]
pub fn ProjectList() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let _session_ctx = expect_context::<SessionContext>();

    // Search query for filtering projects
    let search_query = RwSignal::new(String::new());

    // Filtered projects based on search
    let filtered_projects = move || {
        let query = search_query.get().to_lowercase();
        let projects = ctx.projects.get();
        if query.is_empty() {
            projects
        } else {
            projects
                .into_iter()
                .filter(|p| p.display_name.to_lowercase().contains(&query))
                .collect()
        }
    };

    view! {
        <div class="flex flex-col h-full">
            // Search input with icon
            <div class="px-2 pb-2">
                <div class="relative">
                    <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground pointer-events-none" />
                    <input
                        type="text"
                        placeholder="Search projects..."
                        class="w-full h-8 shadow-none bg-background file:text-foreground placeholder:text-muted-foreground border-input flex min-w-0 rounded-md border bg-transparent pl-8 pr-3 py-1 text-xs transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-2"
                        prop:value=move || search_query.get()
                        on:input=move |e| {
                            search_query.set(event_target_value(&e));
                        }
                    />
                </div>
            </div>

            // Project list
            <div class="flex-1 overflow-auto scrollbar__on_hover">
                <div class="flex flex-col gap-0.5 px-2 pb-2">
                    {move || {
                        let projects = filtered_projects();
                        if ctx.projects_loading.get() {
                            view! {
                                <div class="flex flex-col gap-2 p-2">
                                    <Skeleton class="h-8 w-full"/>
                                    <Skeleton class="h-8 w-full"/>
                                    <Skeleton class="h-8 w-full"/>
                                </div>
                            }.into_any()
                        } else if projects.is_empty() {
                            view! {
                                <div class="px-2 py-8 text-center text-xs text-muted-foreground">
                                    {if search_query.get().is_empty() {
                                        "No projects found. Sessions will appear as you use Claude, Cursor, Codex, or Gemini."
                                    } else {
                                        "No matching projects"
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <ul class="flex flex-col gap-0.5">
                                    {projects.into_iter().map(|project| {
                                        view! { <ProjectItem project=project /> }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

/// A single project item that can be expanded to show sessions.
#[component]
fn ProjectItem(project: Project) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let session_ctx = expect_context::<SessionContext>();

    let name = project.name.clone();
    let display_name = project.display_name.clone();

    // Count all sessions across providers
    let total_sessions = project.sessions.len()
        + project.cursor_sessions.len()
        + project.codex_sessions.len()
        + project.gemini_sessions.len();

    // Collect all sessions from all providers into one flat list
    let all_sessions: Vec<SessionInfo> = {
        let mut all = Vec::new();
        all.extend(project.sessions.clone());
        all.extend(project.cursor_sessions.clone());
        all.extend(project.codex_sessions.clone());
        all.extend(project.gemini_sessions.clone());
        // Sort by last_activity descending
        all.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
        all
    };

    let is_selected = {
        let name = name.clone();
        Memo::new(move |_| {
            ctx.selected_project
                .get()
                .map(|p| p.name == name)
                .unwrap_or(false)
        })
    };

    // Expand/collapse signal for showing sessions under this project
    let is_expanded = RwSignal::new(false);

    // When the project becomes selected, auto-expand it
    Effect::new(move |_| {
        if is_selected.get() {
            is_expanded.set(true);
        }
    });

    let name_for_click = name.clone();
    let _project_for_select = project.clone();

    view! {
        <li class="relative">
            <Collapsible open=is_expanded>
                // Project header row
                <CollapsibleTrigger class="w-full">
                    <div
                        class=move || {
                            let base = "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition-colors cursor-pointer select-none";
                            if is_selected.get() {
                                format!("{base} bg-sidenav-accent text-sidenav-accent-foreground font-medium")
                            } else {
                                format!("{base} text-sidenav-foreground hover:bg-sidenav-accent hover:text-sidenav-accent-foreground")
                            }
                        }
                        on:click={
                            let name = name_for_click.clone();
                            move |_| {
                                ctx.select_project_by_name(&name);
                                session_ctx.load_sessions(name.clone());
                            }
                        }
                    >
                        // Expand/collapse chevron
                        <span class=move || {
                            let base = "flex-shrink-0 transition-transform duration-200";
                            if is_expanded.get() {
                                format!("{base} rotate-90")
                            } else {
                                base.to_string()
                            }
                        }>
                            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="m9 18 6-6-6-6"/>
                            </svg>
                        </span>

                        // Project name
                        <span class="flex-1 truncate min-w-0 text-sm">{display_name.clone()}</span>

                        // Session count badge
                        {if total_sessions > 0 {
                            Some(view! {
                                <span class="flex-shrink-0 text-[11px] text-muted-foreground tabular-nums">
                                    {total_sessions}
                                </span>
                            })
                        } else {
                            None
                        }}
                    </div>
                </CollapsibleTrigger>

                // Session list (collapsed content)
                <CollapsibleContent class="pl-2">
                    <SessionList
                        project_name=name.clone()
                        sessions=all_sessions
                    />
                </CollapsibleContent>
            </Collapsible>
        </li>
    }
}

/// List of sessions under a project. Shows inline sessions from the project
/// and can load more from the backend via SessionContext.
#[component]
fn SessionList(project_name: String, sessions: Vec<SessionInfo>) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let session_ctx = expect_context::<SessionContext>();

    // Rename dialog state
    let renaming_session_id = RwSignal::new(Option::<String>::None);
    let rename_value = RwSignal::new(String::new());

    // Delete confirmation dialog state
    let deleting_session_id = RwSignal::new(Option::<String>::None);
    let deleting_provider = RwSignal::new(Option::<String>::None);

    let project_name_more = project_name.clone();
    let project_name_del = project_name.clone();

    view! {
        <ul class="flex flex-col gap-0.5 py-1">
            {sessions.into_iter().map(|session| {
                let session_id = session.id.clone();
                let session_clone = session.clone();
                let provider = session.provider.clone();

                let is_session_selected = {
                    let session_id = session_id.clone();
                    Memo::new(move |_| {
                        ctx.selected_session.get()
                            .map(|s| s.id == session_id)
                            .unwrap_or(false)
                    })
                };

                let on_click = {
                    let session = session_clone.clone();
                    Callback::new(move |_: ()| {
                        ctx.selected_session.set(Some(session.clone()));
                    })
                };

                let on_delete = {
                    let provider = provider.clone();
                    Callback::new(move |id: String| {
                        deleting_session_id.set(Some(id));
                        deleting_provider.set(Some(provider.clone()));
                    })
                };

                let on_rename = Callback::new(move |id: String| {
                    renaming_session_id.set(Some(id));
                    rename_value.set(String::new());
                });

                view! {
                    <SessionItem
                        session=session
                        is_selected=is_session_selected
                        on_click=on_click
                        on_delete=on_delete
                        on_rename=on_rename
                    />
                }
            }).collect_view()}

            // Load more button
            {
                let session_ctx_has_more = session_ctx;
                let project_name_more = project_name_more.clone();
                move || {
                    if session_ctx_has_more.has_more.get() {
                        let project_name = project_name_more.clone();
                        Some(view! {
                            <li>
                                <button
                                    class="w-full px-2 py-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors text-center"
                                    on:click=move |_| {
                                        session_ctx_has_more.load_more(project_name.clone());
                                    }
                                    disabled=move || session_ctx_has_more.loading.get()
                                >
                                    {move || if session_ctx_has_more.loading.get() {
                                        "Loading..."
                                    } else {
                                        "Load more sessions"
                                    }}
                                </button>
                            </li>
                        })
                    } else {
                        None
                    }
                }
            }
        </ul>

        // Delete confirmation dialog
        {move || {
            deleting_session_id.get().map(|session_id| {
                let project_name = project_name_del.clone();
                let session_id_confirm = session_id.clone();
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80">
                        <div class="bg-background border border-border rounded-lg p-4 shadow-lg w-80">
                            <p class="text-sm font-medium mb-1">"Delete Session"</p>
                            <p class="text-xs text-muted-foreground mb-4">
                                "This session will be permanently deleted. This action cannot be undone."
                            </p>
                            <div class="flex justify-end gap-2">
                                <button
                                    class="px-3 py-1.5 text-xs rounded-md border border-border text-muted-foreground hover:text-foreground transition-colors"
                                    on:click=move |_| {
                                        deleting_session_id.set(None);
                                        deleting_provider.set(None);
                                    }
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="px-3 py-1.5 text-xs rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors"
                                    on:click={
                                        let session_id = session_id_confirm.clone();
                                        move |_| {
                                            let provider = deleting_provider.get_untracked().unwrap_or_default();
                                            let pn = project_name.clone();
                                            let sid = session_id.clone();
                                            deleting_session_id.set(None);
                                            deleting_provider.set(None);
                                            spawn_local(async move {
                                                if let Err(e) = commands::delete_session(
                                                    &sid,
                                                    Some(&provider),
                                                    Some(&pn),
                                                ).await {
                                                    expect_toaster().error(format!("Failed to delete session: {e}"));
                                                } else {
                                                    expect_toaster().success("Session deleted");
                                                }
                                                // Reload projects to reflect the deletion
                                                if let Ok(projects) = commands::list_projects(true).await {
                                                    let ctx = expect_context::<AppContext>();
                                                    ctx.projects.set(projects);
                                                }
                                            });
                                        }
                                    }
                                >
                                    "Delete"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })
        }}

        // Inline rename input (shown when renaming)
        {move || {
            renaming_session_id.get().map(|session_id| {
                let session_id_submit = session_id.clone();
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80">
                        <div class="bg-background border border-border rounded-lg p-4 shadow-lg w-80">
                            <p class="text-sm font-medium mb-2">"Rename Session"</p>
                            <input
                                type="text"
                                placeholder="Enter new name..."
                                class="w-full h-8 bg-background border-input flex rounded-md border px-3 py-1 text-sm outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-2"
                                prop:value=move || rename_value.get()
                                on:input=move |e| {
                                    rename_value.set(event_target_value(&e));
                                }
                                on:keydown=move |e: web_sys::KeyboardEvent| {
                                    if e.key() == "Enter" {
                                        let val = rename_value.get_untracked();
                                        if !val.is_empty() {
                                            let sid = session_id_submit.clone();
                                            spawn_local(async move {
                                                if let Err(e) = commands::set_session_name(&sid, &val, None).await {
                                                    expect_toaster().error(format!("Failed to rename session: {e}"));
                                                } else {
                                                    expect_toaster().success("Session renamed");
                                                }
                                                // Reload projects
                                                if let Ok(projects) = commands::list_projects(true).await {
                                                    let ctx = expect_context::<AppContext>();
                                                    ctx.projects.set(projects);
                                                }
                                            });
                                        }
                                        renaming_session_id.set(None);
                                    } else if e.key() == "Escape" {
                                        renaming_session_id.set(None);
                                    }
                                }
                            />
                            <div class="flex justify-end gap-2 mt-3">
                                <button
                                    class="px-3 py-1.5 text-xs rounded-md border border-border text-muted-foreground hover:text-foreground transition-colors"
                                    on:click=move |_| renaming_session_id.set(None)
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="px-3 py-1.5 text-xs rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
                                    on:click={
                                        let session_id = session_id.clone();
                                        move |_| {
                                            let val = rename_value.get_untracked();
                                            if !val.is_empty() {
                                                let sid = session_id.clone();
                                                spawn_local(async move {
                                                    if let Err(e) = commands::set_session_name(&sid, &val, None).await {
                                                        expect_toaster().error(format!("Failed to rename session: {e}"));
                                                    } else {
                                                        expect_toaster().success("Session renamed");
                                                    }
                                                    // Reload projects
                                                    if let Ok(projects) = commands::list_projects(true).await {
                                                        let ctx = expect_context::<AppContext>();
                                                        ctx.projects.set(projects);
                                                    }
                                                });
                                            }
                                            renaming_session_id.set(None);
                                        }
                                    }
                                >
                                    "Save"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })
        }}
    }
}
