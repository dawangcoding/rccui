mod app;
mod constants;
mod features;
mod hooks;
mod layout;
mod pages;
mod state;
mod tauri;
mod ui;
mod utils;

use app::*;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();

    // Remove loading indicator after WASM is ready
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("app-loading"))
    {
        el.remove();
    }

    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
