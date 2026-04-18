use leptos::prelude::*;
use leptos_fluent::tr;

use crate::state::{AppContext, ChatContext};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};

/// Chat input component with send button and streaming controls.
#[component]
pub fn ChatInput() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let chat_ctx = expect_context::<ChatContext>();

    let input_value = RwSignal::new(String::new());
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

    let is_streaming = chat_ctx.is_streaming;
    let can_send = Memo::new(move |_| {
        !input_value.get().trim().is_empty() && !is_streaming.get()
    });

    // Send handler
    let send_message = move || {
        let text = input_value.get_untracked().trim().to_string();
        if text.is_empty() || is_streaming.get_untracked() {
            return;
        }

        let project = ctx.selected_project.get_untracked();
        let project_path = project.as_ref().map(|p| p.full_path.clone());
        let session = ctx.selected_session.get_untracked();
        let session_id = session.map(|s| s.id);

        chat_ctx.send_message(text, project_path, session_id);
        input_value.set(String::new());

        // Refocus the textarea
        if let Some(el) = textarea_ref.get_untracked() {
            let _ = el.focus();
        }
    };

    let send_message_clone = send_message.clone();

    // Handle Enter key (send on Enter, newline on Shift+Enter)
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            send_message_clone();
        }
    };

    let provider_display = move || {
        let provider = chat_ctx.provider.get();
        format!("{}", provider)
    };

    view! {
        <div class="border-t bg-background px-4 py-3">
            <div class="flex flex-col gap-2">
                // Input area
                <div class="flex gap-2 items-end">
                    <div class="flex-1 relative">
                        <textarea
                            node_ref=textarea_ref
                            class="flex w-full rounded-lg border border-input bg-transparent px-3 py-2.5 text-sm shadow-xs placeholder:text-muted-foreground focus-visible:outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-2 disabled:cursor-not-allowed disabled:opacity-50 resize-none min-h-[42px] max-h-[200px] field-sizing-content"
                            placeholder=move || tr!("chat-input-placeholder")
                            prop:value=move || input_value.get()
                            on:input=move |ev| {
                                input_value.set(event_target_value(&ev));
                            }
                            on:keydown=on_keydown
                            disabled=move || is_streaming.get()
                        />
                    </div>

                    // Send / Stop button
                    {move || {
                        if is_streaming.get() {
                            view! {
                                <Button
                                    variant=ButtonVariant::Destructive
                                    size=ButtonSize::Icon
                                    on:click=move |_| chat_ctx.abort_session()
                                    attr:title=tr!("chat-stop-btn")
                                >
                                    <icons::Square class="size-4" />
                                </Button>
                            }.into_any()
                        } else {
                            let send = send_message.clone();
                            view! {
                                <Button
                                    size=ButtonSize::Icon
                                    on:click=move |_| send()
                                    attr:disabled=move || !can_send.get()
                                >
                                    <icons::ArrowUp class="size-4" />
                                </Button>
                            }.into_any()
                        }
                    }}
                </div>

                // Bottom bar: provider indicator
                <div class="flex items-center justify-between text-xs text-muted-foreground">
                    <div class="flex items-center gap-1.5">
                        <span class=move || {
                            let provider = chat_ctx.provider.get();
                            let color_class = match provider {
                                crate::tauri::types::SessionProvider::Claude => "text-provider-claude",
                                crate::tauri::types::SessionProvider::Codex => "text-provider-codex",
                                crate::tauri::types::SessionProvider::Gemini => "text-provider-gemini",
                                crate::tauri::types::SessionProvider::Cursor => "text-muted-foreground",
                            };
                            format!("font-medium {color_class}")
                        }>
                            {provider_display}
                        </span>
                        {move || {
                            chat_ctx.model.get().map(|m| {
                                view! { <span class="text-muted-foreground/70">{format!(" / {m}")}</span> }
                            })
                        }}
                    </div>
                    <span class="text-muted-foreground/50">
                        {tr!("chat-enter-to-send")}
                    </span>
                </div>
            </div>
        </div>
    }
}
