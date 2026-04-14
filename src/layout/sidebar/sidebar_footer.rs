use leptos::prelude::*;
use leptos_fluent::tr;

use crate::ui::separator::Separator;
use crate::ui::sidenav::SidenavFooter as SidenavFooterBase;
use crate::ui::theme_toggle::ThemeToggle;
use crate::ui::tooltip::{Tooltip, TooltipContent, TooltipPosition};

/// Sidebar footer with settings access and theme toggle.
#[component]
pub fn SidebarFooter() -> impl IntoView {
    view! {
        <SidenavFooterBase>
            <Separator />
            <div class="flex items-center justify-between px-1">
                // Settings link
                <Tooltip>
                    <a
                        href="/settings"
                        class="inline-flex items-center justify-center size-7 rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
                            <circle cx="12" cy="12" r="3"/>
                        </svg>
                    </a>
                    <TooltipContent position=TooltipPosition::Top>
                        {move || tr!("tooltip-settings")}
                    </TooltipContent>
                </Tooltip>

                // Theme toggle
                <Tooltip>
                    <ThemeToggle />
                    <TooltipContent position=TooltipPosition::Top>
                        {move || tr!("tooltip-toggle-theme")}
                    </TooltipContent>
                </Tooltip>
            </div>
        </SidenavFooterBase>
    }
}
