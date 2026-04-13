use wasm_bindgen::prelude::*;

use super::types::*;

// ─── Tauri event binding ─────────────────────────────────────────────────────

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> JsValue;
}

/// A handle to an active event listener. Drop it or call `unlisten()` to stop.
pub struct EventUnlisten {
    _closure: Closure<dyn FnMut(JsValue)>,
    unlisten_fn: js_sys::Function,
}

impl EventUnlisten {
    pub fn unlisten(self) {
        let _ = self.unlisten_fn.call0(&JsValue::NULL);
    }
}

/// Raw Tauri event envelope: `{ event, payload }`.
#[derive(serde::Deserialize)]
struct TauriEvent<T> {
    #[allow(dead_code)]
    event: String,
    payload: T,
}

/// Listen to a Tauri event and deserialize the payload.
async fn listen_event<T, F>(event_name: &str, mut callback: F) -> EventUnlisten
where
    T: serde::de::DeserializeOwned + 'static,
    F: FnMut(T) + 'static,
{
    let closure = Closure::new(move |val: JsValue| {
        if let Ok(evt) = serde_wasm_bindgen::from_value::<TauriEvent<T>>(val) {
            callback(evt.payload);
        }
    });
    let unlisten_js = listen(event_name, &closure).await;
    let unlisten_fn: js_sys::Function = unlisten_js.into();
    EventUnlisten {
        _closure: closure,
        unlisten_fn,
    }
}

// ─── Typed listeners ─────────────────────────────────────────────────────────

pub async fn listen_chat_response(
    callback: impl FnMut(ChatResponse) + 'static,
) -> EventUnlisten {
    listen_event("chat_response", callback).await
}

pub async fn listen_shell_output(
    callback: impl FnMut(ShellOutputPayload) + 'static,
) -> EventUnlisten {
    listen_event("shell_output", callback).await
}

pub async fn listen_shell_auth_url(
    callback: impl FnMut(ShellAuthUrlPayload) + 'static,
) -> EventUnlisten {
    listen_event("shell_auth_url", callback).await
}

pub async fn listen_projects_updated(
    callback: impl FnMut(ProjectsUpdatedPayload) + 'static,
) -> EventUnlisten {
    listen_event("projects_updated", callback).await
}
