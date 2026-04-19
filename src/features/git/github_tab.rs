use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fluent::tr;

use crate::state::{AppContext, GitContext};
use crate::tauri::commands;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::input::Input;
use crate::ui::scroll_area::ScrollArea;
use crate::ui::separator::Separator;
use crate::ui::spinner::Spinner;
use crate::ui::toast_custom::toaster::expect_toaster;

/// Format disk usage from KB to human-readable string.
fn format_disk_usage(kb: u64) -> String {
    if kb >= 1_048_576 {
        format!("{:.1} GB", kb as f64 / 1_048_576.0)
    } else if kb >= 1024 {
        format!("{:.1} MB", kb as f64 / 1024.0)
    } else {
        format!("{} KB", kb)
    }
}

/// Format ISO 8601 date to a shorter display form (YYYY-MM-DD).
fn format_date(iso: &str) -> String {
    iso.split('T').next().unwrap_or(iso).to_string()
}

/// GitHub sub-tab: display repository info from `gh` CLI.
#[component]
pub fn GitHubTab() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let git_ctx = expect_context::<GitContext>();
    let toaster = StoredValue::new(expect_toaster());

    // Create repo form state
    let repo_name = RwSignal::new(String::new());
    let repo_desc = RwSignal::new(String::new());
    let repo_private = RwSignal::new(false);
    let creating = RwSignal::new(false);

    // Pre-capture i18n strings for spawn_local
    let msg_created = StoredValue::new(tr!("toast-github-created"));
    let msg_error = StoredValue::new(tr!("toast-git-error"));

    let on_create = move |_: leptos::ev::MouseEvent| {
        let name = repo_name.get_untracked();
        if name.trim().is_empty() {
            return;
        }
        let Some(project) = ctx.selected_project.get_untracked() else {
            return;
        };
        let desc = repo_desc.get_untracked();
        let is_private = repo_private.get_untracked();
        let project_name = project.name.clone();
        let msg_created = msg_created.get_value();
        let msg_error = msg_error.get_value();
        let toaster = toaster.get_value();

        creating.set(true);
        git_ctx.error_message.set(None);

        spawn_local(async move {
            match commands::gh_repo_create(&project_name, &name, &desc, is_private, true).await {
                Ok(_) => {
                    toaster.success(msg_created);
                    repo_name.set(String::new());
                    repo_desc.set(String::new());
                    git_ctx.load_github_info(project_name);
                }
                Err(e) => {
                    toaster.error(format!("{msg_error}: {e}"));
                }
            }
            creating.set(false);
        });
    };

    view! {
        <div class="flex flex-col flex-1 min-h-0 min-w-0 overflow-hidden">
            {move || {
                let loading = git_ctx.github_loading.get();
                let info = git_ctx.github_info.get();
                let no_remote = git_ctx.github_no_remote.get();
                let is_creating = creating.get();

                if loading || is_creating {
                    let msg = if is_creating { tr!("git-github-creating") } else { tr!("git-loading") };
                    return view! {
                        <div class="flex items-center justify-center flex-1 gap-2 text-sm text-muted-foreground">
                            <Spinner class="size-4" />
                            {msg}
                        </div>
                    }.into_any();
                }

                if no_remote {

                    return view! {
                        <ScrollArea class="flex-1">
                            <div class="flex flex-col items-center justify-center p-6 gap-6">
                                <div class="flex flex-col items-center gap-2 text-center">
                                    <icons::Github class="size-10 text-muted-foreground/40" />
                                    <h3 class="text-sm font-medium text-foreground">{tr!("git-github-no-remote-title")}</h3>
                                    <p class="text-xs text-muted-foreground max-w-xs">{tr!("git-github-no-remote-hint")}</p>
                                </div>

                                <div class="w-full max-w-sm space-y-3">
                                    // Repo name
                                    <div>
                                        <label class="text-xs font-medium text-foreground mb-1 block">{tr!("git-github-repo-name")}</label>
                                        <Input
                                            placeholder=tr!("git-github-repo-name-placeholder")
                                            bind_value=repo_name
                                        />
                                    </div>
                                    // Description
                                    <div>
                                        <label class="text-xs font-medium text-foreground mb-1 block">{tr!("git-github-repo-desc")}</label>
                                        <Input
                                            placeholder=tr!("git-github-repo-desc-placeholder")
                                            bind_value=repo_desc
                                        />
                                    </div>
                                    // Visibility toggle
                                    <div class="flex items-center gap-2">
                                        <button
                                            class=move || {
                                                let private = repo_private.get();
                                                format!(
                                                    "flex-1 px-3 py-1.5 text-xs font-medium rounded-md border transition-colors {}",
                                                    if !private {
                                                        "bg-primary text-primary-foreground border-primary"
                                                    } else {
                                                        "bg-background text-muted-foreground border-border hover:bg-accent/50"
                                                    }
                                                )
                                            }
                                            on:click=move |_| repo_private.set(false)
                                        >
                                            <div class="flex items-center justify-center gap-1.5">
                                                <icons::Globe class="size-3.5" />
                                                {tr!("git-github-visibility-public")}
                                            </div>
                                        </button>
                                        <button
                                            class=move || {
                                                let private = repo_private.get();
                                                format!(
                                                    "flex-1 px-3 py-1.5 text-xs font-medium rounded-md border transition-colors {}",
                                                    if private {
                                                        "bg-primary text-primary-foreground border-primary"
                                                    } else {
                                                        "bg-background text-muted-foreground border-border hover:bg-accent/50"
                                                    }
                                                )
                                            }
                                            on:click=move |_| repo_private.set(true)
                                        >
                                            <div class="flex items-center justify-center gap-1.5">
                                                <icons::Lock class="size-3.5" />
                                                {tr!("git-github-visibility-private")}
                                            </div>
                                        </button>
                                    </div>
                                    // Create button
                                    <Button
                                        class="w-full"
                                        attr:disabled=move || repo_name.get().trim().is_empty()
                                        on:click=on_create
                                    >
                                        <icons::Plus class="size-4" />
                                        {tr!("git-github-create-repo")}
                                    </Button>
                                </div>
                            </div>
                        </ScrollArea>
                    }.into_any();
                }

                let Some(info) = info else {
                    return view! {
                        <div class="flex flex-col items-center justify-center flex-1 gap-2 text-sm text-muted-foreground">
                            <icons::Github class="size-8 text-muted-foreground/50" />
                            <p>{tr!("git-github-empty")}</p>
                            <p class="text-xs">{tr!("git-github-empty-hint")}</p>
                        </div>
                    }.into_any();
                };

                // Clone values for use in closures
                let url = info.url.clone();
                let url_for_btn = url.clone();

                view! {
                    <ScrollArea class="flex-1">
                        <div class="p-4 space-y-4">
                            // ── Header: owner/name + visibility ──
                            <div class="flex items-start justify-between gap-2">
                                <div class="min-w-0">
                                    <div class="flex items-center gap-2">
                                        <h2 class="text-base font-semibold text-foreground truncate">
                                            {format!("{}/{}", info.owner.login, info.name)}
                                        </h2>
                                        <Badge variant={if info.is_private { BadgeVariant::Secondary } else { BadgeVariant::Outline }}>
                                            {if info.is_private {
                                                tr!("git-github-visibility-private")
                                            } else {
                                                tr!("git-github-visibility-public")
                                            }}
                                        </Badge>
                                    </div>
                                    <p class="mt-1 text-sm text-muted-foreground">
                                        {if info.description.is_empty() {
                                            tr!("git-github-no-description")
                                        } else {
                                            info.description.clone()
                                        }}
                                    </p>
                                </div>
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Icon
                                    class="size-7 shrink-0"
                                    on:click=move |_| {
                                        if let Some(project) = ctx.selected_project.get_untracked() {
                                            git_ctx.load_github_info(project.name.clone());
                                        }
                                    }
                                >
                                    <icons::RefreshCw class="size-3.5" />
                                </Button>
                            </div>

                            <Separator />

                            // ── Stats row ──
                            <div class="flex flex-wrap items-center gap-4 text-sm">
                                // Stars
                                <div class="flex items-center gap-1.5 text-muted-foreground">
                                    <icons::Star class="size-4" />
                                    <span class="font-medium text-foreground">{info.stargazer_count}</span>
                                    <span>{tr!("git-github-stars")}</span>
                                </div>
                                // Forks
                                <div class="flex items-center gap-1.5 text-muted-foreground">
                                    <icons::GitFork class="size-4" />
                                    <span class="font-medium text-foreground">{info.fork_count}</span>
                                    <span>{tr!("git-github-forks")}</span>
                                </div>
                                // Language
                                {info.primary_language.as_ref().map(|lang| {
                                    let name = lang.name.clone();
                                    view! {
                                        <div class="flex items-center gap-1.5 text-muted-foreground">
                                            <span class="size-3 rounded-full bg-primary" />
                                            <span>{name}</span>
                                        </div>
                                    }
                                })}
                                // License
                                {info.license_info.as_ref().map(|lic| {
                                    let name = lic.spdx_id.clone().unwrap_or_else(|| lic.name.clone());
                                    view! {
                                        <div class="flex items-center gap-1.5 text-muted-foreground">
                                            <icons::Scale class="size-4" />
                                            <span>{name}</span>
                                        </div>
                                    }
                                })}
                            </div>

                            <Separator />

                            // ── Details ──
                            <div class="grid grid-cols-2 gap-3 text-sm">
                                // Default branch
                                {info.default_branch_ref.as_ref().map(|br| {
                                    let name = br.name.clone();
                                    view! {
                                        <div>
                                            <div class="text-xs text-muted-foreground mb-0.5">{tr!("git-github-default-branch")}</div>
                                            <div class="flex items-center gap-1.5 text-foreground">
                                                <icons::GitBranch class="size-3.5 text-muted-foreground" />
                                                <span class="font-mono text-xs">{name}</span>
                                            </div>
                                        </div>
                                    }
                                })}
                                // Disk usage
                                {info.disk_usage.map(|kb| {
                                    let size = format_disk_usage(kb);
                                    view! {
                                        <div>
                                            <div class="text-xs text-muted-foreground mb-0.5">{tr!("git-github-disk-usage")}</div>
                                            <div class="flex items-center gap-1.5 text-foreground">
                                                <icons::HardDrive class="size-3.5 text-muted-foreground" />
                                                <span>{size}</span>
                                            </div>
                                        </div>
                                    }
                                })}
                                // Homepage
                                {info.homepage_url.as_ref().filter(|u| !u.is_empty()).map(|homepage| {
                                    let homepage = homepage.clone();
                                    view! {
                                        <div class="col-span-2">
                                            <div class="text-xs text-muted-foreground mb-0.5">{tr!("git-github-homepage")}</div>
                                            <div class="flex items-center gap-1.5 text-foreground">
                                                <icons::Globe class="size-3.5 text-muted-foreground" />
                                                <span class="text-xs truncate">{homepage}</span>
                                            </div>
                                        </div>
                                    }
                                })}
                            </div>

                            // ── Topics ──
                            {if !info.repository_topics.is_empty() {
                                let topics = info.repository_topics.clone();
                                view! {
                                    <>
                                        <Separator />
                                        <div>
                                            <div class="text-xs text-muted-foreground mb-2">{tr!("git-github-topics")}</div>
                                            <div class="flex flex-wrap gap-1.5">
                                                {topics.into_iter().map(|t| {
                                                    view! {
                                                        <Badge variant=BadgeVariant::Secondary class="text-xs">
                                                            {t.name}
                                                        </Badge>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    </>
                                }.into_any()
                            } else {
                                view! {}.into_any()
                            }}

                            <Separator />

                            // ── Footer: dates + open link ──
                            <div class="flex items-center justify-between text-xs text-muted-foreground">
                                <div class="flex items-center gap-4">
                                    {if !info.created_at.is_empty() {
                                        let date = format_date(&info.created_at);
                                        view! {
                                            <span>{move || format!("{}: {}", tr!("git-github-created"), date)}</span>
                                        }.into_any()
                                    } else {
                                        view! {}.into_any()
                                    }}
                                    {if !info.updated_at.is_empty() {
                                        let date = format_date(&info.updated_at);
                                        view! {
                                            <span>{move || format!("{}: {}", tr!("git-github-updated"), date)}</span>
                                        }.into_any()
                                    } else {
                                        view! {}.into_any()
                                    }}
                                </div>
                                <Button
                                    variant=ButtonVariant::Outline
                                    size=ButtonSize::Sm
                                    class="h-7 text-xs gap-1.5"
                                    on:click=move |_| {
                                        let _ = web_sys::window()
                                            .and_then(|w| w.open_with_url_and_target(&url_for_btn, "_blank").ok());
                                    }
                                >
                                    <icons::ExternalLink class="size-3" />
                                    {tr!("git-github-open")}
                                </Button>
                            </div>
                        </div>
                    </ScrollArea>
                }.into_any()
            }}
        </div>
    }
}
