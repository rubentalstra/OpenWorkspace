use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use web_sys::Storage;

const STORAGE_KEY: &str = "darkmode";
const PREFERS_DARK_QUERY: &str = "(prefers-color-scheme: dark)";
const DARK_CLASS: &str = "dark";

/// Reactive dark-mode state, shared through Leptos context.
///
/// Construct once near the app root with [`ThemeMode::init`], then read or
/// mutate it anywhere via [`use_theme_mode`]. The signal is `false` (light) on
/// the server and during the first client paint, so server and client markup
/// stay identical; a client-only effect then resolves the real preference from
/// local storage — falling back to the OS color-scheme — and a second effect
/// mirrors the active value onto the `dark` class of the document root so the
/// Tailwind dark variant takes effect. Pure Leptos — no inline theme script.
///
/// This is first-party plumbing (shadcn does dark mode via `next-themes`, which
/// is React-only), kept deliberately outside the shadcn component port.
#[derive(Debug, Clone, Copy)]
pub struct ThemeMode {
    state: RwSignal<bool>,
}

/// Returns the [`ThemeMode`] provided by [`ThemeMode::init`]. Requires that
/// `init` ran earlier in the render tree.
#[must_use]
pub fn use_theme_mode() -> ThemeMode {
    expect_context::<ThemeMode>()
}

impl ThemeMode {
    /// Creates the shared theme state, provides it to context, and schedules the
    /// client-only effects that resolve the initial preference and keep the
    /// document root's `dark` class in sync. Returns the context handle.
    #[must_use]
    pub fn init() -> Self {
        let theme_mode = Self {
            state: RwSignal::new(false),
        };

        provide_context(theme_mode);
        theme_mode.resolve_initial_preference();
        theme_mode.sync_document_class();

        theme_mode
    }

    /// Flips between dark and light, persisting the new value.
    pub fn toggle(self) {
        self.set(!self.state.get_untracked());
    }

    /// Sets dark mode (`true` = dark, `false` = light) and persists it.
    pub fn set(self, dark: bool) {
        self.state.set(dark);
        Self::persist(dark);
    }

    /// Returns whether dark mode is currently active, tracking reactively.
    #[must_use]
    pub fn is_dark(self) -> bool {
        self.state.get()
    }

    fn resolve_initial_preference(self) {
        let state = self.state;

        // Storage and the media query are browser-only; an effect never runs
        // during SSR, so reading them here keeps the server render identical.
        Effect::new(move |_| {
            if let Some(stored) = Self::stored_preference() {
                state.set(stored);
                return;
            }

            let Some(mql) = window().match_media(PREFERS_DARK_QUERY).ok().flatten() else {
                return;
            };
            state.set(mql.matches());

            // While no explicit choice is stored the theme follows the OS, so a
            // live `change` listener keeps it current when the scheme flips.
            let media = mql.clone();
            let on_change = Closure::<dyn Fn()>::new(move || {
                if Self::stored_preference().is_none() {
                    state.set(media.matches());
                }
            });
            _ = mql.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
            // The browser holds only a weak reference; the closure must outlive
            // this scope and lives for the page's lifetime to track the OS scheme.
            on_change.forget();
        });
    }

    fn sync_document_class(self) {
        let state = self.state;

        Effect::new(move |_| {
            let dark = state.get();
            let Some(root) = document().document_element() else {
                return;
            };
            let classes = root.class_list();
            _ = if dark {
                classes.add_1(DARK_CLASS)
            } else {
                classes.remove_1(DARK_CLASS)
            };
        });
    }

    fn storage() -> Option<Storage> {
        window().local_storage().ok().flatten()
    }

    fn stored_preference() -> Option<bool> {
        Self::storage()
            .and_then(|storage| storage.get(STORAGE_KEY).ok().flatten())
            .and_then(|entry| entry.parse::<bool>().ok())
    }

    fn persist(dark: bool) {
        if let Some(storage) = Self::storage() {
            _ = storage.set(STORAGE_KEY, dark.to_string().as_str());
        }
    }
}
