use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;

use crate::layout::AppLayout;
use crate::pages::dashboard::DashboardPage;
use crate::pages::onboarding::OnboardingPage;
use crate::pages::project::ProjectPage;
use crate::pages::settings::SettingsPage;
use crate::state::AppContext;
use crate::tauri::commands;

#[component]
pub fn App() -> impl IntoView {
    // Create global application state
    let ctx = AppContext::new();
    provide_context(ctx);

    // Load projects on mount
    spawn_local(async move {
        ctx.projects_loading.set(true);
        match commands::list_projects(false).await {
            Ok(projects) => {
                ctx.projects.set(projects);
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to load projects: {e}").into());
            }
        }
        ctx.projects_loading.set(false);
    });

    // Check onboarding status
    spawn_local(async move {
        match commands::get_onboarding_status().await {
            Ok(status) => {
                ctx.onboarding_complete.set(status.has_completed_onboarding);
            }
            Err(e) => {
                web_sys::console::error_1(
                    &format!("Failed to check onboarding: {e}").into(),
                );
            }
        }
    });

    view! {
        <Router>
            <Routes fallback=|| view! {
                <div class="flex items-center justify-center h-screen text-sm text-muted-foreground">
                    "Page not found"
                </div>
            }>
                // Main layout with sidebar wraps all pages
                <ParentRoute path=path!("") view=AppLayout>
                    <Route path=path!("/") view=DashboardPage />
                    <Route path=path!("/project/:name") view=ProjectPage />
                    <Route path=path!("/settings") view=SettingsPage />
                    <Route path=path!("/onboarding") view=OnboardingPage />
                </ParentRoute>
            </Routes>
        </Router>
    }
}
