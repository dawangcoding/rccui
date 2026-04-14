use leptos::prelude::*;
use leptos_fluent::tr;

use crate::state::AppContext;
use crate::ui::sidenav::*;

use super::project_list::ProjectList;
use super::sidebar_footer::SidebarFooter;

/// Main sidebar component with project list and sessions.
#[component]
pub fn Sidebar() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    view! {
        <Sidenav variant=SidenavVariant::Sidenav data_collapsible=SidenavCollapsible::Offcanvas>
            <SidenavHeader>
                <div class="flex items-center gap-2">
                    <span class="text-sm font-semibold text-foreground truncate">{move || tr!("sidebar-projects")}</span>
                    <span class="ml-auto text-xs text-muted-foreground tabular-nums">
                        {move || ctx.projects.get().len()}
                    </span>
                </div>
            </SidenavHeader>

            <SidenavContent>
                <ProjectList />
            </SidenavContent>

            <SidebarFooter />
        </Sidenav>
    }
}
