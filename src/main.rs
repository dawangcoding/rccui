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
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
