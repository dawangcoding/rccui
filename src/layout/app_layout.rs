use leptos::prelude::*;
use leptos_router::components::Outlet;

use crate::ui::sidenav::{SidenavInset, SidenavWrapper};

use super::editor_sidebar::EditorSidebar;
use super::main_header::MainHeader;
use super::sidebar::Sidebar;

/// Root application layout: Sidebar + Main Content area + Editor Sidebar.
/// The main content swaps via `<Outlet/>` based on the current route.
#[component]
pub fn AppLayout() -> impl IntoView {
    view! {
        <SidenavWrapper class="h-screen overflow-hidden" attr:style="--sidenav-width:16rem;">
            <Sidebar />
            <SidenavInset class="min-w-0">
                <MainHeader />
                <div class="flex flex-1 min-h-0 min-w-0 overflow-hidden">
                    <main class="flex-1 min-w-0 h-full overflow-hidden flex flex-col">
                        <Outlet />
                    </main>
                    <EditorSidebar />
                </div>
            </SidenavInset>
        </SidenavWrapper>
    }
}
