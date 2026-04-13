use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::ui::{
    button::Button,
    card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle},
    input::Input,
};

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
    let name = RwSignal::new(String::new());
    let (greet_msg, set_greet_msg) = signal(String::new());

    let greet = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let name = name.get_untracked();
            if name.is_empty() {
                return;
            }

            let args = serde_wasm_bindgen::to_value(&GreetArgs { name: &name }).unwrap();
            let new_msg = invoke("greet", args).await.as_string().unwrap();
            set_greet_msg.set(new_msg);
        });
    };

    view! {
        <main class="flex flex-col items-center justify-center min-h-screen p-8 gap-6 bg-background">
            <h1 class="text-3xl font-bold text-foreground">"Welcome to Tauri + Leptos"</h1>

            <div class="flex items-center gap-8">
                <a href="https://tauri.app" target="_blank" class="transition-opacity hover:opacity-80">
                    <img src="public/tauri.svg" class="w-24 h-24" alt="Tauri logo"/>
                </a>
                <a href="https://docs.rs/leptos/" target="_blank" class="transition-opacity hover:opacity-80">
                    <img src="public/leptos.svg" class="w-24 h-24" alt="Leptos logo"/>
                </a>
            </div>

            <p class="text-muted-foreground">"Click on the Tauri and Leptos logos to learn more."</p>

            <Card class="w-full max-w-md border-transparent shadow-none">
                <CardContent class="pt-6">
                    <form class="flex items-center gap-4" on:submit=greet>
                        <Input
                            id="greet-input"
                            placeholder="Enter a name..."
                            bind_value=name
                            class="flex-1"
                        />
                        <Button attr:r#type="submit">
                            "Greet"
                        </Button>
                    </form>
                </CardContent>
                <CardFooter>
                    <p class="text-lg text-foreground">{ move || greet_msg.get() }</p>
                </CardFooter>
            </Card>
        </main>
    }
}
