use leptos::prelude::*;
use leptos_fluent::tr;

use crate::state::ChatContext;
use crate::ui::spinner::Spinner;

use super::message_item::MessageItem;

/// Scrollable message list container.
#[component]
pub fn MessageList() -> impl IntoView {
    let chat_ctx = expect_context::<ChatContext>();

    // Ref for auto-scroll
    let bottom_ref = NodeRef::<leptos::html::Div>::new();

    // Auto-scroll to bottom when messages change or stream updates
    Effect::new(move || {
        let _ = chat_ctx.messages.get();
        let _ = chat_ctx.stream_content.get();
        if let Some(el) = bottom_ref.get() {
            el.scroll_into_view();
        }
    });

    view! {
        <div class="flex-1 min-w-0 overflow-y-auto overflow-x-hidden px-4 py-4 no__scrollbar">
            // Loading state
            {move || {
                if chat_ctx.messages_loading.get() {
                    Some(view! {
                        <div class="flex items-center justify-center py-8 gap-2 text-sm text-muted-foreground">
                            <Spinner class="size-4" />
                            <span>{tr!("loading")}</span>
                        </div>
                    })
                } else {
                    None
                }
            }}

            // Message items
            {move || {
                let messages = chat_ctx.messages.get();
                let is_streaming = chat_ctx.is_streaming.get();

                // Find the index of the last collapsible message (thinking/tool_use/tool_result)
                // During streaming, auto-expand it so user sees the current step
                let last_collapsible_idx = if is_streaming {
                    messages.iter().rposition(|m| {
                        matches!(m.kind.as_str(), "thinking" | "tool_use" | "tool_result")
                    })
                } else {
                    None
                };

                messages.into_iter().enumerate().map(|(i, msg)| {
                    let auto_expand = last_collapsible_idx == Some(i);
                    view! {
                        <MessageItem message=msg auto_expand=auto_expand />
                    }
                }).collect_view()
            }}

            // Streaming content (in-progress assistant response)
            {move || {
                let content = chat_ctx.stream_content.get();
                let is_streaming = chat_ctx.is_streaming.get();
                if !content.is_empty() {
                    view! {
                        <div class="flex justify-start mb-3">
                            <div class="max-w-[85%] min-w-0 text-sm overflow-hidden">
                                <pre class="whitespace-pre-wrap break-words font-sans text-sm leading-relaxed text-foreground">{content}</pre>
                                {if is_streaming {
                                    Some(view! {
                                        <span class="inline-block w-2 h-4 bg-foreground/60 animate-pulse ml-0.5 align-text-bottom" />
                                    })
                                } else {
                                    None
                                }}
                            </div>
                        </div>
                    }.into_any()
                } else if is_streaming {
                    view! {
                        <div class="flex justify-start mb-3">
                            <div class="flex items-center gap-2 text-sm text-muted-foreground">
                                <Spinner class="size-3.5" />
                                <span class="text-xs">{tr!("chat-thinking-indicator")}</span>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}

            // Scroll anchor
            <div node_ref=bottom_ref />
        </div>
    }
}
