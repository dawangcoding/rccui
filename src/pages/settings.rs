use leptos::prelude::*;
use leptos_fluent::tr;

use crate::ui::card::{Card, CardHeader, CardTitle, CardDescription, CardContent};

/// Settings page with language switcher.
#[component]
pub fn SettingsPage() -> impl IntoView {
    let i18n = expect_context::<leptos_fluent::I18n>();

    view! {
        <div class="flex flex-col h-full overflow-auto p-8">
            <div class="max-w-2xl mx-auto w-full">
                <h1 class="text-2xl font-bold tracking-tight mb-6">{move || tr!("settings-title")}</h1>

                <Card class="border-transparent shadow-none">
                    <CardHeader>
                        <CardTitle>{move || tr!("settings-language-title")}</CardTitle>
                        <CardDescription>{move || tr!("settings-language-desc")}</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div class="flex flex-col gap-2">
                            {move || {
                                let languages = i18n.languages;
                                let current_lang = i18n.language.get();
                                languages.iter().map(|lang| {
                                    let lang_id = lang.id.to_string();
                                    let is_active = lang.id == current_lang.id;
                                    let display_name = match lang_id.as_str() {
                                        "zh-CN" => "中文（简体）",
                                        "en" => "English",
                                        _ => lang.name,
                                    };
                                    let lang_clone = *lang;
                                    view! {
                                        <button
                                            class=if is_active {
                                                "flex items-center justify-between w-full px-4 py-3 rounded-lg border-2 border-primary bg-primary/5 text-sm font-medium text-foreground transition-all duration-200"
                                            } else {
                                                "flex items-center justify-between w-full px-4 py-3 rounded-lg border border-border text-sm text-muted-foreground hover:text-foreground hover:border-primary/30 hover:bg-accent/50 transition-all duration-200 cursor-pointer"
                                            }
                                            on:click=move |_| {
                                                i18n.language.set(lang_clone);
                                            }
                                        >
                                            <div class="flex items-center gap-3">
                                                <span class="text-base">{if lang_id == "zh-CN" { "🇨🇳" } else { "🇺🇸" }}</span>
                                                <span>{display_name}</span>
                                            </div>
                                            {if is_active {
                                                Some(view! {
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary">
                                                        <path d="M20 6 9 17l-5-5"/>
                                                    </svg>
                                                })
                                            } else {
                                                None
                                            }}
                                        </button>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </CardContent>
                </Card>
            </div>
        </div>
    }
}
