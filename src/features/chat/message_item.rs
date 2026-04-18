use leptos::prelude::*;
use leptos_fluent::tr;

use crate::tauri::types::NormalizedMessage;
use crate::ui::alert::{Alert, AlertDescription, AlertTitle};
use crate::ui::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};

/// Render a single message based on its kind.
#[component]
pub fn MessageItem(
    message: NormalizedMessage,
    /// Whether this collapsible item should start expanded (used for the current step during streaming).
    #[prop(default = false)]
    auto_expand: bool,
) -> impl IntoView {
    let kind = message.kind.as_str();
    let is_user = message.role.as_deref() == Some("user");

    match kind {
        "text" => render_text_message(message, is_user).into_any(),
        "tool_use" => render_tool_use(message, auto_expand).into_any(),
        "tool_result" => render_tool_result(message, auto_expand).into_any(),
        "thinking" => render_thinking(message, auto_expand).into_any(),
        "error" => render_error(message).into_any(),
        _ => view! {}.into_any(),
    }
}

fn render_text_message(msg: NormalizedMessage, is_user: bool) -> impl IntoView {
    let content = msg.content.unwrap_or_default();

    if is_user {
        view! {
            <div class="flex justify-end mb-3">
                <div class="max-w-[85%] rounded-2xl rounded-br-md px-4 py-2.5 bg-primary text-primary-foreground text-sm overflow-hidden">
                    <pre class="whitespace-pre-wrap break-words font-sans text-sm leading-relaxed">{content}</pre>
                </div>
            </div>
        }
        .into_any()
    } else {
        view! {
            <div class="flex justify-start mb-3">
                <div class="max-w-[85%] min-w-0 text-sm overflow-hidden">
                    <pre class="whitespace-pre-wrap break-words font-sans text-sm leading-relaxed text-foreground">{content}</pre>
                </div>
            </div>
        }
        .into_any()
    }
}

/// Chevron icon that rotates when open. Uses a wrapper span since icon class is static.
fn chevron_icon(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <span class=move || {
            if open.get() {
                "inline-flex transition-transform duration-200 rotate-90"
            } else {
                "inline-flex transition-transform duration-200"
            }
        }>
            <icons::ChevronRight class="size-3.5" />
        </span>
    }
}

fn render_tool_use(msg: NormalizedMessage, auto_expand: bool) -> impl IntoView {
    let tool_name = msg.tool_name.unwrap_or_else(|| "Unknown".to_string());
    let tool_input = msg
        .tool_input
        .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| v.to_string()))
        .unwrap_or_default();

    let open = RwSignal::new(auto_expand);

    view! {
        <div class="mb-2 pl-2">
            <Collapsible open=open>
                <CollapsibleTrigger class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors w-full text-left py-1">
                    {chevron_icon(open)}
                    <icons::Wrench class="size-3.5" />
                    <span class="font-medium">{tool_name.clone()}</span>
                </CollapsibleTrigger>
                <CollapsibleContent class="pl-7 pt-1">
                    <pre class="text-xs text-muted-foreground bg-muted/50 rounded-md p-2.5 overflow-x-auto max-h-60 font-mono">{tool_input}</pre>
                </CollapsibleContent>
            </Collapsible>
        </div>
    }
}

fn render_tool_result(msg: NormalizedMessage, auto_expand: bool) -> impl IntoView {
    let content = msg.content.unwrap_or_default();
    let is_error = msg.is_error.unwrap_or(false);
    let open = RwSignal::new(auto_expand);

    let label = if is_error {
        tr!("chat-tool-error")
    } else {
        tr!("chat-tool-result")
    };

    let pre_class = if is_error {
        "text-xs rounded-md p-2.5 overflow-x-auto max-h-60 font-mono bg-destructive/10 text-destructive"
    } else {
        "text-xs rounded-md p-2.5 overflow-x-auto max-h-60 font-mono bg-muted/50 text-muted-foreground"
    };

    view! {
        <div class="mb-2 pl-2">
            <Collapsible open=open>
                <CollapsibleTrigger class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors w-full text-left py-1">
                    {chevron_icon(open)}
                    {if is_error {
                        view! { <icons::CircleX class="size-3.5 text-destructive" /> }.into_any()
                    } else {
                        view! { <icons::CircleCheck class="size-3.5 text-success" /> }.into_any()
                    }}
                    <span class="font-medium">{label}</span>
                </CollapsibleTrigger>
                <CollapsibleContent class="pl-7 pt-1">
                    <pre class=pre_class>{content}</pre>
                </CollapsibleContent>
            </Collapsible>
        </div>
    }
}

fn render_thinking(msg: NormalizedMessage, auto_expand: bool) -> impl IntoView {
    let content = msg.content.unwrap_or_default();
    let open = RwSignal::new(auto_expand);

    view! {
        <div class="mb-2 pl-2">
            <Collapsible open=open>
                <CollapsibleTrigger class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors w-full text-left py-1">
                    {chevron_icon(open)}
                    <icons::Brain class="size-3.5" />
                    <span class="font-medium italic">{tr!("chat-thinking")}</span>
                </CollapsibleTrigger>
                <CollapsibleContent class="pl-7 pt-1">
                    <pre class="text-xs text-muted-foreground/70 bg-muted/30 rounded-md p-2.5 overflow-x-auto max-h-60 font-mono italic">{content}</pre>
                </CollapsibleContent>
            </Collapsible>
        </div>
    }
}

fn render_error(msg: NormalizedMessage) -> impl IntoView {
    let content = msg.content.unwrap_or_else(|| tr!("chat-unknown-error"));

    view! {
        <div class="mb-3">
            <Alert class="border-destructive/50 bg-destructive/5 text-destructive">
                <icons::CircleAlert class="size-4" />
                <AlertTitle>{tr!("chat-error-title")}</AlertTitle>
                <AlertDescription>
                    <pre class="whitespace-pre-wrap break-words font-sans text-xs">{content}</pre>
                </AlertDescription>
            </Alert>
        </div>
    }
}
