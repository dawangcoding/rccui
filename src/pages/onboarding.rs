use leptos::prelude::*;
use leptos_fluent::tr;

/// Onboarding page placeholder - will be implemented in Phase 6.
#[component]
pub fn OnboardingPage() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center h-full text-sm text-muted-foreground">
            {move || tr!("onboarding-coming")}
        </div>
    }
}
