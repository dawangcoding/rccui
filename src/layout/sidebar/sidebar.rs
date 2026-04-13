use leptos::ev;
use leptos::prelude::*;

use crate::state::AppContext;
use crate::ui::sidenav::*;

use super::sidebar_footer::SidebarFooter;

/// Main sidebar component with project list and sessions.
#[component]
pub fn Sidebar() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
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
        <Sidenav variant=SidenavVariant::Sidenav data_collapsible=SidenavCollapsible::Offcanvas>
            <SidenavHeader>
                <div class="flex items-center gap-2">
                    <span class="text-sm font-semibold text-foreground truncate">"Projects"</span>
                    <span class="ml-auto text-xs text-muted-foreground">
                        {move || ctx.projects.get().len()}
                    </span>
                </div>
                <input
                    type="text"
                    placeholder="Search projects..."
                    class="w-full h-8 shadow-none bg-background file:text-foreground placeholder:text-muted-foreground border-input flex min-w-0 rounded-md border bg-transparent px-3 py-1 text-sm transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-2"
                    prop:value=move || search_query.get()
                    on:input=move |e| {
                        search_query.set(event_target_value(&e));
                    }
                />
            </SidenavHeader>

            <SidenavContent>
                <div class="flex flex-col gap-0.5 p-2">
                    {move || {
                        let projects = filtered_projects();
                        if ctx.projects_loading.get() {
                            view! {
                                <div class="flex flex-col gap-2 p-2">
                                    <div class="h-8 rounded-md bg-muted animate-pulse"/>
                                    <div class="h-8 rounded-md bg-muted animate-pulse"/>
                                    <div class="h-8 rounded-md bg-muted animate-pulse"/>
                                </div>
                            }.into_any()
                        } else if projects.is_empty() {
                            view! {
                                <div class="px-2 py-8 text-center text-sm text-muted-foreground">
                                    {if search_query.get().is_empty() {
                                        "No projects found"
                                    } else {
                                        "No matching projects"
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <ul class="flex flex-col gap-0.5">
                                    {projects.into_iter().map(|project| {
                                        let name = project.name.clone();
                                        let display_name = project.display_name.clone();
                                        let session_count = project.sessions.len()
                                            + project.cursor_sessions.len()
                                            + project.codex_sessions.len()
                                            + project.gemini_sessions.len();
                                        let is_selected = {
                                            let name = name.clone();
                                            Memo::new(move |_| {
                                                ctx.selected_project.get()
                                                    .map(|p| p.name == name)
                                                    .unwrap_or(false)
                                            })
                                        };

                                        view! {
                                            <li class="relative group/menu-item">
                                                <SidenavMenuButton
                                                    class="justify-between"
                                                    attr:aria-current=move || if is_selected.get() { "page" } else { "false" }
                                                    on:click={
                                                        let name = name.clone();
                                                        move |_| ctx.select_project_by_name(&name)
                                                    }
                                                >
                                                    <span class="truncate">{display_name.clone()}</span>
                                                    {if session_count > 0 {
                                                        view! {
                                                            <span class="text-xs text-muted-foreground tabular-nums">
                                                                {session_count}
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {}.into_any()
                                                    }}
                                                </SidenavMenuButton>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                    }}
                </div>
            </SidenavContent>

            <SidebarFooter />
        </Sidenav>
    }
}
