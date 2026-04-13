use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::tauri::commands;
use crate::tauri::types::SessionInfo;

// ─── SessionContext ──────────────────────────────────────────────────────────

/// Per-project session state. Provided when a project is selected.
#[derive(Clone, Copy)]
pub struct SessionContext {
    /// Sessions for the currently selected project
    pub sessions: RwSignal<Vec<SessionInfo>>,
    /// Total number of sessions available (server-side)
    pub total: RwSignal<usize>,
    /// Whether more sessions can be loaded (pagination)
    pub has_more: RwSignal<bool>,
    /// Current pagination offset
    pub offset: RwSignal<u32>,
    /// Whether sessions are currently loading
    pub loading: RwSignal<bool>,
    /// Search query for filtering sessions
    pub search_query: RwSignal<String>,
}

const PAGE_SIZE: u32 = 20;

impl SessionContext {
    pub fn new() -> Self {
        Self {
            sessions: RwSignal::new(Vec::new()),
            total: RwSignal::new(0),
            has_more: RwSignal::new(false),
            offset: RwSignal::new(0),
            loading: RwSignal::new(false),
            search_query: RwSignal::new(String::new()),
        }
    }

    /// Load sessions for a given project (resets pagination).
    pub fn load_sessions(&self, project_name: String) {
        let ctx = *self;
        ctx.loading.set(true);
        ctx.offset.set(0);
        ctx.sessions.set(Vec::new());

        spawn_local(async move {
            match commands::list_sessions(&project_name, None, Some(PAGE_SIZE), Some(0)).await {
                Ok(resp) => {
                    ctx.sessions.set(resp.sessions);
                    ctx.total.set(resp.total);
                    ctx.has_more.set(resp.has_more);
                    ctx.offset.set(PAGE_SIZE);
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to load sessions: {e}").into(),
                    );
                }
            }
            ctx.loading.set(false);
        });
    }

    /// Load more sessions (pagination).
    pub fn load_more(&self, project_name: String) {
        if self.loading.get_untracked() || !self.has_more.get_untracked() {
            return;
        }

        let ctx = *self;
        let current_offset = ctx.offset.get_untracked();
        ctx.loading.set(true);

        spawn_local(async move {
            match commands::list_sessions(&project_name, None, Some(PAGE_SIZE), Some(current_offset))
                .await
            {
                Ok(resp) => {
                    ctx.sessions.update(|sessions| {
                        sessions.extend(resp.sessions);
                    });
                    ctx.total.set(resp.total);
                    ctx.has_more.set(resp.has_more);
                    ctx.offset.set(current_offset + PAGE_SIZE);
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to load more sessions: {e}").into(),
                    );
                }
            }
            ctx.loading.set(false);
        });
    }

    /// Clear all session state.
    pub fn clear(&self) {
        self.sessions.set(Vec::new());
        self.total.set(0);
        self.has_more.set(false);
        self.offset.set(0);
        self.search_query.set(String::new());
    }
}
