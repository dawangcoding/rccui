use leptos::prelude::*;
use leptos_fluent::tr;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::state::{AppContext, ShellContext};

// ─── JS Interop bindings ─────────────────────────────────────────────────────

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "XtermBridge"])]
    fn create(id: &str, container: &web_sys::HtmlElement, opts: &JsValue) -> bool;

    #[wasm_bindgen(js_namespace = ["window", "XtermBridge"])]
    fn onData(id: &str, callback: &Closure<dyn FnMut(String)>);

    #[wasm_bindgen(js_namespace = ["window", "XtermBridge"])]
    fn onResize(id: &str, callback: &Closure<dyn FnMut(JsValue)>);

    #[wasm_bindgen(js_namespace = ["window", "XtermBridge"])]
    fn fit(id: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "XtermBridge"])]
    fn focus(id: &str);

    #[wasm_bindgen(js_namespace = ["window", "XtermBridge"])]
    fn dispose(id: &str);

    #[wasm_bindgen(js_namespace = ["window", "XtermBridge"])]
    fn has(id: &str) -> bool;

    #[wasm_bindgen(js_namespace = ["window", "XtermBridge"])]
    fn updateTheme(id: &str);

    #[wasm_bindgen(js_namespace = ["window", "XtermBridge"])]
    fn clear(id: &str);

    #[wasm_bindgen(js_namespace = ["window", "XtermBridge"])]
    fn getDimensions(id: &str) -> JsValue;
}

/// Helper to extract { cols, rows } from a JsValue.
fn extract_dims(dims: &JsValue) -> (u16, u16) {
    if dims.is_null() || dims.is_undefined() {
        return (80, 24);
    }
    let cols = js_sys::Reflect::get(dims, &JsValue::from_str("cols"))
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(80.0) as u16;
    let rows = js_sys::Reflect::get(dims, &JsValue::from_str("rows"))
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(24.0) as u16;
    (cols, rows)
}

/// The terminal component. Manages xterm.js lifecycle and PTY connection.
#[component]
pub fn ShellTerminal() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let shell_ctx = expect_context::<ShellContext>();
    let container_ref = NodeRef::<leptos::html::Div>::new();

    // Stable terminal ID per project so it persists across tab switches
    let terminal_id = Memo::new(move |_| {
        ctx.selected_project
            .get()
            .map(|p| format!("term_{}", p.name))
            .unwrap_or_else(|| "term_default".to_string())
    });

    // Initialize terminal when container is available and project is selected
    Effect::new(move || {
        let project = ctx.selected_project.get();
        let term_id = terminal_id.get();

        if project.is_none() {
            return;
        }
        let project = project.unwrap();

        let Some(container) = container_ref.get() else {
            return;
        };

        // Don't re-init if already created
        if has(&term_id) {
            fit(&term_id);
            focus(&term_id);
            return;
        }

        // Create xterm instance
        let opts = js_sys::Object::new();
        create(&term_id, &container, &JsValue::from(opts));

        // Set up onData callback — sends user input to PTY
        let shell_ctx_input = shell_ctx;
        let on_data_closure = Closure::<dyn FnMut(String)>::new(move |data: String| {
            shell_ctx_input.send_input(data);
        });
        onData(&term_id, &on_data_closure);
        // Leak closure to keep it alive (same pattern as app.rs event listeners)
        std::mem::forget(on_data_closure);

        // Set up onResize callback — sends resize to PTY
        let shell_ctx_resize = shell_ctx;
        let on_resize_closure = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
            let (cols, rows) = extract_dims(&event);
            shell_ctx_resize.resize(cols, rows);
        });
        onResize(&term_id, &on_resize_closure);
        std::mem::forget(on_resize_closure);

        // Fit terminal and get dimensions
        let dims = fit(&term_id);
        let (cols, rows) = extract_dims(&dims);

        // Initialize PTY session (pass term_id so ShellContext knows which xterm instance to write to)
        shell_ctx.init_session(
            term_id.clone(),
            project.full_path.clone(),
            None,
            "claude".to_string(),
            cols,
            rows,
        );

        focus(&term_id);
    });

    // Set up ResizeObserver to re-fit terminal when container size changes
    Effect::new(move || {
        let _ = terminal_id.get(); // Track changes
        let Some(container) = container_ref.get() else {
            return;
        };

        let term_id_for_resize = terminal_id.get_untracked();
        let shell_ctx_fit = shell_ctx;

        let resize_cb = Closure::<dyn FnMut(JsValue)>::new(move |_entries: JsValue| {
            let tid = term_id_for_resize.clone();
            if has(&tid) {
                let dims = fit(&tid);
                let (cols, rows) = extract_dims(&dims);
                shell_ctx_fit.resize(cols, rows);
            }
        });

        if let Ok(observer) =
            web_sys::ResizeObserver::new(resize_cb.as_ref().unchecked_ref())
        {
            observer.observe(&container);
            // Leak to keep alive for component lifetime
            std::mem::forget(observer);
        }
        std::mem::forget(resize_cb);
    });

    // Cleanup when component is unmounted
    on_cleanup(move || {
        let term_id = terminal_id.get_untracked();
        shell_ctx.detach();
        dispose(&term_id);
    });

    // Status bar signals
    let is_init = shell_ctx.is_initializing;
    let is_conn = shell_ctx.is_connected;
    let error = shell_ctx.error_message;
    let auth_url_sig = shell_ctx.auth_url;

    view! {
        <div class="flex flex-col h-full min-w-0">
            // Status bar
            <div class="flex items-center justify-between px-3 py-1.5 border-b border-border bg-muted/30 text-xs text-muted-foreground shrink-0">
                <div class="flex items-center gap-2">
                    <icons::SquareTerminal class="size-3.5" />
                    <span>{tr!("tab-shell")}</span>
                    {move || {
                        if is_init.get() {
                            view! {
                                <span class="flex items-center gap-1 text-info">
                                    <span class="size-1.5 rounded-full bg-info animate-pulse" />
                                    {tr!("shell-connecting")}
                                </span>
                            }.into_any()
                        } else if is_conn.get() {
                            view! {
                                <span class="flex items-center gap-1 text-success">
                                    <span class="size-1.5 rounded-full bg-success" />
                                    {tr!("shell-connected")}
                                </span>
                            }.into_any()
                        } else {
                            view! {
                                <span class="flex items-center gap-1 text-muted-foreground">
                                    <span class="size-1.5 rounded-full bg-muted-foreground/50" />
                                    {tr!("shell-disconnected")}
                                </span>
                            }.into_any()
                        }
                    }}
                </div>

                // Reconnect button when disconnected
                {move || {
                    if !is_conn.get() && !is_init.get() && shell_ctx.session_key.get().is_some() {
                        view! {
                            <button
                                class="text-xs text-primary hover:underline"
                                on:click=move |_| {
                                    if let Some(project) = ctx.selected_project.get_untracked() {
                                        let term_id = format!("term_{}", project.name);
                                        let dims = getDimensions(&term_id);
                                        let (cols, rows) = extract_dims(&dims);
                                        shell_ctx.init_session(
                                            term_id.clone(),
                                            project.full_path,
                                            None,
                                            "claude".to_string(),
                                            cols,
                                            rows,
                                        );
                                    }
                                }
                            >
                                {tr!("shell-reconnect")}
                            </button>
                        }.into_any()
                    } else {
                        view! { <span /> }.into_any()
                    }
                }}
            </div>

            // Error banner
            {move || {
                error.get().map(|e| view! {
                    <div class="px-3 py-2 text-xs text-destructive bg-destructive/10 border-b border-destructive/20">
                        {e}
                    </div>
                })
            }}

            // Auth URL banner
            {move || {
                auth_url_sig.get().map(|url| {
                    let url_clone = url.clone();
                    view! {
                        <div class="px-3 py-2 text-xs text-info bg-info/10 border-b border-info/20 flex items-center gap-2">
                            <icons::ExternalLink class="size-3.5 shrink-0" />
                            <span class="truncate">{tr!("shell-auth-required")}</span>
                            <a
                                href={url_clone}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-primary hover:underline shrink-0"
                            >
                                {tr!("shell-open-link")}
                            </a>
                        </div>
                    }
                })
            }}

            // Terminal container — xterm.js mounts here
            <div
                node_ref=container_ref
                class="flex-1 min-h-0 bg-background p-1"
                on:click=move |_| {
                    let tid = terminal_id.get_untracked();
                    if has(&tid) {
                        focus(&tid);
                    }
                }
            />
        </div>
    }
}
