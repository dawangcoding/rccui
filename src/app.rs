use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;

use crate::layout::AppLayout;
use crate::pages::dashboard::DashboardPage;
use crate::pages::onboarding::OnboardingPage;
use crate::pages::project::ProjectPage;
use crate::pages::settings::SettingsPage;
use crate::hooks::use_theme_mode::ThemeMode;
use crate::state::{AppContext, SessionContext};
use crate::tauri::{commands, events};
use crate::ui::toast_custom::toaster::{expect_toaster, provide_toaster, Toaster};

#[component]
pub fn App() -> impl IntoView {
    // Initialize theme mode (must be before any component that uses ThemeToggle)
    let _theme = ThemeMode::init();

    // Initialize toast notification system
    provide_toaster();

    // Create global application state
    let ctx = AppContext::new();
    provide_context(ctx);

    // Create session context
    let session_ctx = SessionContext::new();
    provide_context(session_ctx);

    // Load projects on mount
    spawn_local(async move {
        ctx.projects_loading.set(true);
        match commands::list_projects(false).await {
            Ok(projects) => {
                ctx.projects.set(projects);
            }
            Err(e) => {
                expect_toaster().error(format!("Failed to load projects: {e}"));
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
                expect_toaster().error(format!("Failed to check onboarding: {e}"));
            }
        }
    });

    // Listen for projects_updated events from the file watcher
    spawn_local(async move {
        let _unlisten = events::listen_projects_updated(move |_payload| {
            // Re-fetch projects when the file watcher detects changes
            spawn_local(async move {
                match commands::list_projects(true).await {
                    Ok(projects) => {
                        ctx.projects.set(projects);
                    }
                    Err(e) => {
                        expect_toaster().error(format!("Failed to refresh projects: {e}"));
                    }
                }
            });
        })
        .await;
        // Keep the unlisten handle alive for the lifetime of the app
        // by leaking it (it should never be dropped)
        std::mem::forget(_unlisten);
    });

    view! {
        <Toaster />
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
