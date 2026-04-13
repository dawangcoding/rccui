use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[component]
pub fn App() -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (greet_msg, set_greet_msg) = signal(String::new());

    let update_name = move |ev| {
        let v = event_target_value(&ev);
        set_name.set(v);
    };

    let greet = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let name = name.get_untracked();
            if name.is_empty() {
                return;
            }

            let args = serde_wasm_bindgen::to_value(&GreetArgs { name: &name }).unwrap();
            // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
            let new_msg = invoke("greet", args).await.as_string().unwrap();
            set_greet_msg.set(new_msg);
        });
    };

    view! {
        <main class="flex flex-col items-center justify-center min-h-screen p-8 gap-6">
            <h1 class="text-3xl font-bold">"Welcome to Tauri + Leptos"</h1>

            <div class="flex items-center gap-8">
                <a href="https://tauri.app" target="_blank" class="transition-opacity hover:opacity-80">
                    <img src="public/tauri.svg" class="w-24 h-24" alt="Tauri logo"/>
                </a>
                <a href="https://docs.rs/leptos/" target="_blank" class="transition-opacity hover:opacity-80">
                    <img src="public/leptos.svg" class="w-24 h-24" alt="Leptos logo"/>
                </a>
            </div>
            <p class="text-muted-foreground">"Click on the Tauri and Leptos logos to learn more."</p>

            <form class="flex items-center gap-4" on:submit=greet>
                <input
                    id="greet-input"
                    placeholder="Enter a name..."
                    on:input=update_name
                    class="px-4 py-2 border border-border rounded-md bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                />
                <button
                    type="submit"
                    class="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
                >
                    "Greet"
                </button>
            </form>
            <p class="text-lg">{ move || greet_msg.get() }</p>
        </main>
    }
}
