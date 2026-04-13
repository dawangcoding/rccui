use leptos::prelude::*;
use web_sys::Storage;

#[derive(Debug, Clone, Copy)]
pub struct ThemeMode {
    state: RwSignal<bool>,
}

const LOCALSTORAGE_KEY: &str = "darkmode";

/// Hook to access the dark mode context
///
/// Returns the ThemeMode instance from context for easy access
pub fn use_theme_mode() -> ThemeMode {
    expect_context::<ThemeMode>()
}

/* ========================================================== */
/*                     ✨ FUNCTIONS ✨                        */
/* ========================================================== */

impl ThemeMode {
    #[must_use]
    /// Initializes a new ThemeMode instance.
    pub fn init() -> Self {
        let theme_mode = Self { state: RwSignal::new(false) };

        provide_context(theme_mode);

        // Use Effect to handle browser-only initialization and DOM sync
        Effect::new(move |_| {
            let initial = Self::get_storage_state().unwrap_or(Self::prefers_dark_mode());
            theme_mode.state.set(initial);
            Self::apply_to_dom(initial);
        });

        theme_mode
    }

    pub fn toggle(&self) {
        self.state.update(|state| {
            *state = !*state;
            Self::set_storage_state(*state);
            Self::apply_to_dom(*state);
        });
    }

    pub fn set_dark(&self) {
        self.set(true);
    }

    pub fn set_light(&self) {
        self.set(false);
    }

    /// - `dark`: Set to `true` for dark mode, and `false` for light mode.
    pub fn set(&self, dark: bool) {
        self.state.set(dark);
        Self::set_storage_state(dark);
        Self::apply_to_dom(dark);
    }

    #[must_use]
    pub fn get(&self) -> bool {
        self.state.get()
    }

    #[must_use]
    pub fn is_dark(&self) -> bool {
        self.state.get()
    }

    #[must_use]
    pub fn is_light(&self) -> bool {
        !self.state.get()
    }

    /* ========================================================== */
    /*                     ✨ FUNCTIONS ✨                        */
    /* ========================================================== */

    /// Retrieves the local storage object, if available.
    fn get_storage() -> Option<Storage> {
        window().local_storage().ok().flatten()
    }

    /// Retrieves the dark mode state from local storage, if available.
    fn get_storage_state() -> Option<bool> {
        Self::get_storage()
            .and_then(|storage| storage.get(LOCALSTORAGE_KEY).ok())
            .flatten()
            .and_then(|entry| entry.parse::<bool>().ok())
    }

    /// Checks whether the user's system prefers dark mode based on media queries.
    fn prefers_dark_mode() -> bool {
        window()
            .match_media("(prefers-color-scheme: dark)")
            .ok()
            .flatten()
            .map(|media| media.matches())
            .unwrap_or_default()
    }

    /// Stores the dark mode state in local storage.
    fn set_storage_state(state: bool) {
        if let Some(storage) = Self::get_storage() {
            storage.set(LOCALSTORAGE_KEY, state.to_string().as_str()).ok();
        }
    }

    /// Applies or removes the `dark` class on the document root element (`<html>`).
    fn apply_to_dom(dark: bool) {
        if let Some(root) = window().document().and_then(|d| d.document_element()) {
            let class_list = root.class_list();
            if dark {
                let _ = class_list.add_1("dark");
            } else {
                let _ = class_list.remove_1("dark");
            }
        }
    }
}
