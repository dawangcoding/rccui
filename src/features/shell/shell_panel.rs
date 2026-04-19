use leptos::prelude::*;
use leptos_fluent::tr;

use crate::state::{AppContext, ShellContext};

use super::terminal::ShellTerminal;

/// Main shell panel — assembles the terminal view.
#[component]
pub fn ShellPanel() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let shell_ctx = expect_context::<ShellContext>();

    // When selected_project changes, we need to clear the previous terminal
    // and let the terminal component handle the new initialization.
    Effect::new(move || {
        let project = ctx.selected_project.get();
        if project.is_none() {
            shell_ctx.clear();
        }
    });

    view! {
        <div class="flex flex-col flex-1 min-h-0 min-w-0 overflow-hidden">
            {move || {
                let project = ctx.selected_project.get();
                match project {
                    None => view! {
                        <div class="flex-1 flex flex-col items-center justify-center gap-3 text-muted-foreground px-4">
                            <icons::SquareTerminal class="size-10 opacity-20" />
                            <p class="text-sm text-center">{tr!("shell-empty-state")}</p>
                        </div>
                    }.into_any(),
                    Some(_) => view! {
                        <ShellTerminal />
                    }.into_any(),
                }
            }}
        </div>
    }
}
