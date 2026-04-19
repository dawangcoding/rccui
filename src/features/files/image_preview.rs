use leptos::prelude::*;

use crate::state::FileContext;

/// Image preview component — renders an image from base64 data URI.
#[component]
pub fn ImagePreview() -> impl IntoView {
    let file_ctx = expect_context::<FileContext>();

    view! {
        <div class="flex-1 flex flex-col items-center justify-center p-4 min-h-0 overflow-auto">
            {move || {
                let data = file_ctx.image_data.get();
                let file = file_ctx.selected_file.get();

                match data {
                    Some(data_uri) => {
                        let name = file.as_ref().map(|f| f.name.clone()).unwrap_or_default();
                        let size = file.as_ref().map(|f| format_file_size(f.size)).unwrap_or_default();
                        view! {
                            <div class="flex flex-col items-center gap-3 max-w-full max-h-full">
                                <img
                                    src=data_uri
                                    alt=name.clone()
                                    class="max-w-full max-h-[calc(100vh-12rem)] object-contain rounded border border-border"
                                />
                                <div class="flex items-center gap-2 text-xs text-muted-foreground">
                                    <span class="font-medium">{name}</span>
                                    <span>{size}</span>
                                </div>
                            </div>
                        }.into_any()
                    }
                    None => view! {
                        <div class="text-sm text-muted-foreground">
                            "No image to display"
                        </div>
                    }.into_any(),
                }
            }}
        </div>
    }
}

fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
