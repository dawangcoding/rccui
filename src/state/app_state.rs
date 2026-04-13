use leptos::prelude::*;

use crate::tauri::types::{AppTab, Project, SessionInfo};

// ─── AppContext ──────────────────────────────────────────────────────────────

/// Global application state, provided at the root App component.
#[derive(Clone, Copy)]
pub struct AppContext {
    /// All discovered projects
    pub projects: RwSignal<Vec<Project>>,
    /// Currently selected project
    pub selected_project: RwSignal<Option<Project>>,
    /// Currently active tab in the main content area
    pub active_tab: RwSignal<AppTab>,
    /// Whether the sidebar is expanded
    pub sidebar_open: RwSignal<bool>,
    /// Whether onboarding has been completed
    pub onboarding_complete: RwSignal<bool>,
    /// Loading state for projects
    pub projects_loading: RwSignal<bool>,
    /// Currently selected session (within the selected project)
    pub selected_session: RwSignal<Option<SessionInfo>>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            projects: RwSignal::new(Vec::new()),
            selected_project: RwSignal::new(None),
            active_tab: RwSignal::new(AppTab::Chat),
            sidebar_open: RwSignal::new(true),
            onboarding_complete: RwSignal::new(true),
            projects_loading: RwSignal::new(false),
            selected_session: RwSignal::new(None),
        }
    }

    /// Select a project by name from the loaded projects list.
    pub fn select_project_by_name(&self, name: &str) {
        let projects = self.projects.get_untracked();
        if let Some(project) = projects.iter().find(|p| p.name == name) {
            self.selected_project.set(Some(project.clone()));
            // Reset session when changing project
            self.selected_session.set(None);
            // Reset to Chat tab
            self.active_tab.set(AppTab::Chat);
        }
    }
}
