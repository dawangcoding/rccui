use leptos::prelude::*;
use leptos_fluent::tr;

use crate::state::{AppContext, ChatContext};
use crate::tauri::types::SessionProvider;

use super::chat_input::ChatInput;
use super::message_list::MessageList;
use super::permission_dialog::PermissionDialog;
use super::quick_settings::QuickSettings;

/// Main chat panel — assembles message list, input, settings, and permission dialog.
#[component]
pub fn ChatPanel() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let chat_ctx = expect_context::<ChatContext>();

    // When selected_session changes, load messages
    Effect::new(move || {
        let session = ctx.selected_session.get();
        let project = ctx.selected_project.get();

        if let (Some(session), Some(project)) = (session, project) {
            let provider = match session.provider.as_str() {
                "cursor" => SessionProvider::Cursor,
                "codex" => SessionProvider::Codex,
                "gemini" => SessionProvider::Gemini,
                _ => SessionProvider::Claude,
            };
            chat_ctx.provider.set(provider);
            chat_ctx.load_messages(
                session.id.clone(),
                provider,
                Some(project.name.clone()),
            );
        } else {
            chat_ctx.clear();
        }
    });

    view! {
        <div class="flex flex-col flex-1 min-h-0 min-w-0">
            // Quick settings bar (provider selection)
            <QuickSettings />

            // Main content area
            {move || {
                let has_session = ctx.selected_session.get().is_some();
                let has_messages = !chat_ctx.messages.get().is_empty()
                    || chat_ctx.is_streaming.get()
                    || chat_ctx.messages_loading.get();

                if !has_session && !has_messages {
                    // Empty state — no session selected, show welcome
                    view! {
                        <div class="flex-1 flex flex-col items-center justify-center gap-3 text-muted-foreground px-4">
                            <icons::MessageSquare class="size-10 opacity-20" />
                            <p class="text-sm text-center">{tr!("chat-empty-state")}</p>
                            <p class="text-xs text-center text-muted-foreground/60">{tr!("chat-empty-hint")}</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <MessageList />
                    }.into_any()
                }
            }}

            // Chat input
            <ChatInput />

            // Permission dialog (overlay)
            <PermissionDialog />
        </div>
    }
}
