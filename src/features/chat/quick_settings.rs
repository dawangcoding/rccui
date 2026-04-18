use leptos::prelude::*;
use leptos_fluent::tr;

use crate::state::ChatContext;
use crate::tauri::types::SessionProvider;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};

/// Quick settings bar for selecting provider and permission mode.
#[component]
pub fn QuickSettings() -> impl IntoView {
    let chat_ctx = expect_context::<ChatContext>();

    let providers = [
        SessionProvider::Claude,
        SessionProvider::Cursor,
        SessionProvider::Codex,
        SessionProvider::Gemini,
    ];

    view! {
        <div class="flex items-center gap-1 px-4 py-1.5 border-b bg-background/50">
            // Provider buttons
            <div class="flex items-center gap-0.5">
                {providers
                    .into_iter()
                    .map(|provider| {
                        let label = match provider {
                            SessionProvider::Claude => "Claude",
                            SessionProvider::Cursor => "Cursor",
                            SessionProvider::Codex => "Codex",
                            SessionProvider::Gemini => "Gemini",
                        };
                        let color_class = match provider {
                            SessionProvider::Claude => "text-provider-claude",
                            SessionProvider::Codex => "text-provider-codex",
                            SessionProvider::Gemini => "text-provider-gemini",
                            SessionProvider::Cursor => "text-muted-foreground",
                        };

                        // Use a reactive wrapper so the button re-renders with correct style
                        view! {
                            {move || {
                                let is_active = chat_ctx.provider.get() == provider;
                                let class = if is_active {
                                    format!("text-xs h-7 px-2 {color_class} bg-accent font-semibold")
                                } else {
                                    format!("text-xs h-7 px-2 {color_class} opacity-60 hover:opacity-100")
                                };
                                view! {
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Sm
                                        class=class
                                        on:click=move |_| {
                                            chat_ctx.provider.set(provider);
                                            chat_ctx.model.set(None);
                                        }
                                    >
                                        {label}
                                    </Button>
                                }
                            }}
                        }
                    })
                    .collect_view()}
            </div>

            <div class="flex-1" />

            // Permission mode indicator
            <div class="flex items-center gap-1">
                {move || {
                    let mode = chat_ctx.permission_mode.get();
                    let (icon, label_text) = match mode.as_str() {
                        "auto-accept" => (
                            view! { <icons::ShieldCheck class="size-3 text-success" /> }.into_any(),
                            tr!("chat-perm-auto-accept"),
                        ),
                        "auto-deny" => (
                            view! { <icons::ShieldX class="size-3 text-destructive" /> }.into_any(),
                            tr!("chat-perm-auto-deny"),
                        ),
                        _ => (
                            view! { <icons::ShieldQuestion class="size-3 text-muted-foreground" /> }.into_any(),
                            tr!("chat-perm-ask"),
                        ),
                    };

                    let next_mode = match mode.as_str() {
                        "default" => "auto-accept",
                        "auto-accept" => "auto-deny",
                        _ => "default",
                    };
                    let next_mode = next_mode.to_string();

                    view! {
                        <Button
                            variant=ButtonVariant::Ghost
                            size=ButtonSize::Sm
                            class="text-xs h-7 px-2 gap-1"
                            on:click=move |_| {
                                chat_ctx.permission_mode.set(next_mode.clone());
                            }
                        >
                            {icon}
                            <span class="text-muted-foreground">{label_text}</span>
                        </Button>
                    }
                }}
            </div>
        </div>
    }
}
