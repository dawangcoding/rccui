use leptos::prelude::*;
use leptos_fluent::tr;

use crate::state::ChatContext;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::card::{Card, CardContent, CardFooter, CardHeader, CardTitle};

/// Permission request overlay. Shown when the assistant wants to use a tool.
#[component]
pub fn PermissionDialog() -> impl IntoView {
    let chat_ctx = expect_context::<ChatContext>();

    view! {
        {move || {
            chat_ctx.pending_permission.get().map(|perm| {
                let tool_name = perm.tool_name.clone();
                let tool_input_display = perm.tool_input
                    .as_ref()
                    .map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string()))
                    .unwrap_or_default();

                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 animate-in fade-in duration-150">
                        <Card class="w-full max-w-md border shadow-lg animate-in zoom-in-95 duration-200">
                            <CardHeader>
                                <CardTitle class="flex items-center gap-2 text-base">
                                    <icons::ShieldQuestion class="size-5 text-warning" />
                                    {tr!("chat-permission-title")}
                                </CardTitle>
                            </CardHeader>
                            <CardContent>
                                <div class="space-y-3">
                                    <p class="text-sm text-muted-foreground">
                                        {tr!("chat-permission-desc")}
                                    </p>
                                    <div class="rounded-md bg-muted/50 p-3">
                                        <p class="text-sm font-medium text-foreground mb-1">{tool_name}</p>
                                        {if !tool_input_display.is_empty() {
                                            Some(view! {
                                                <pre class="text-xs text-muted-foreground overflow-x-auto max-h-40 font-mono">{tool_input_display}</pre>
                                            })
                                        } else {
                                            None
                                        }}
                                    </div>
                                </div>
                            </CardContent>
                            <CardFooter class="flex justify-end gap-2">
                                <Button
                                    variant=ButtonVariant::Outline
                                    size=ButtonSize::Sm
                                    on:click=move |_| chat_ctx.respond_permission(false)
                                >
                                    {tr!("chat-permission-deny")}
                                </Button>
                                <Button
                                    size=ButtonSize::Sm
                                    on:click=move |_| chat_ctx.respond_permission(true)
                                >
                                    {tr!("chat-permission-allow")}
                                </Button>
                            </CardFooter>
                        </Card>
                    </div>
                }
            })
        }}
    }
}
