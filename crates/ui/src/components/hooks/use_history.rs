use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;

/// Undo/redo history stack for URL-based state.
///
/// Tracks a list of URL strings and navigates between them with
/// `history.replaceState`, so no new browser-history entries are created.
///
/// # Usage
/// Call [`UseHistory::init`] once at the top of a page component, then read it
/// anywhere below via [`use_history`].
///
/// ```ignore
/// // In the page component:
/// UseHistory::init();
///
/// // In a child component:
/// let history = use_history();
/// history.push("?color=red".to_string());
///
/// view! {
///     <button on:click=move |_| history.go_back()>"Undo"</button>
///     <button on:click=move |_| history.go_forward()>"Redo"</button>
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct UseHistory {
    history: RwSignal<Vec<String>>,
    index: RwSignal<usize>,
    is_navigating: RwSignal<bool>,
}

impl UseHistory {
    /// Initializes the history stack, provides it as context, and returns it.
    ///
    /// On the client it seeds the stack with the current query string and
    /// registers the undo (Cmd/Ctrl+Z) and redo (Cmd/Ctrl+Shift+Z, Cmd/Ctrl+Y)
    /// keyboard shortcuts; the listener is torn down with the owning component.
    /// The browser work runs inside effects, which never execute during server
    /// render, so SSR starts with an empty stack and touches no browser APIs.
    #[must_use]
    pub fn init() -> Self {
        let hook = Self {
            history: RwSignal::new(Vec::new()),
            index: RwSignal::new(0),
            is_navigating: RwSignal::new(false),
        };

        provide_context(hook);

        Effect::new(move |_| {
            let search = window().location().search().unwrap_or_default();
            hook.history.update(|h| h.push(search));
        });

        Effect::new(move |_| {
            // The handle deregisters the listener when dropped; hand it to
            // cleanup so it is removed with the owning component, not leaked.
            let handle = window_event_listener(leptos::ev::keydown, move |e| {
                hook.handle_keydown(&e);
            });
            on_cleanup(move || handle.remove());
        });

        hook
    }

    /// Pushes a new URL onto the stack, truncating any forward history.
    pub fn push(&self, url: String) {
        if self.is_navigating.get_untracked() {
            return;
        }

        let idx = self.index.get_untracked();
        Self::replace_state(&url);
        self.history.update(|h| {
            h.truncate(idx + 1);
            h.push(url);
        });
        self.index.update(|i| *i += 1);
    }

    /// Navigates one step back in the stack.
    pub fn go_back(&self) {
        let idx = self.index.get_untracked();
        if idx == 0 {
            return;
        }

        self.is_navigating.set(true);
        let new_idx = idx - 1;
        self.index.set(new_idx);

        let url = self
            .history
            .with_untracked(|h| h.get(new_idx).cloned())
            .unwrap_or_default();
        Self::replace_state(&url);

        self.is_navigating.set(false);
    }

    /// Navigates one step forward in the stack.
    pub fn go_forward(&self) {
        let idx = self.index.get_untracked();
        let len = self.history.with_untracked(Vec::len);
        if idx + 1 >= len {
            return;
        }

        self.is_navigating.set(true);
        let new_idx = idx + 1;
        self.index.set(new_idx);

        let url = self
            .history
            .with_untracked(|h| h.get(new_idx).cloned())
            .unwrap_or_default();
        Self::replace_state(&url);

        self.is_navigating.set(false);
    }

    /// Reactive flag that is `true` when there is a previous state to undo to.
    pub fn can_go_back(&self) -> Signal<bool> {
        let index = self.index;
        Signal::derive(move || index.get() > 0)
    }

    /// Reactive flag that is `true` when there is a future state to redo to.
    pub fn can_go_forward(&self) -> Signal<bool> {
        let history = self.history;
        let index = self.index;
        Signal::derive(move || index.get() + 1 < history.with(Vec::len))
    }

    /// Reactive 1-based position in the stack, suitable for display.
    pub fn position(&self) -> Signal<usize> {
        let index = self.index;
        Signal::derive(move || index.get() + 1)
    }

    /// Reactive total number of states in the stack.
    pub fn total(&self) -> Signal<usize> {
        let history = self.history;
        Signal::derive(move || history.with(Vec::len))
    }

    /// Reactive current URL in the history stack.
    pub fn current(&self) -> Signal<String> {
        let history = self.history;
        let index = self.index;
        Signal::derive(move || history.with(|h| h.get(index.get()).cloned().unwrap_or_default()))
    }

    /// Routes a keydown event to undo/redo, ignoring keystrokes that originate
    /// inside text-editing fields so native editing shortcuts keep working.
    fn handle_keydown(&self, e: &leptos::ev::KeyboardEvent) {
        if let Some(target) = e.target()
            && let Some(el) = target.dyn_ref::<web_sys::HtmlElement>()
        {
            let tag = el.tag_name().to_lowercase();
            if matches!(tag.as_str(), "input" | "textarea" | "select") {
                return;
            }
        }

        let key = e.key().to_lowercase();
        let meta = e.meta_key() || e.ctrl_key();
        let shift = e.shift_key();

        if meta && key == "z" && !shift {
            e.prevent_default();
            self.go_back();
        } else if meta && ((key == "z" && shift) || key == "y") {
            e.prevent_default();
            self.go_forward();
        }
    }

    /// Replaces the current history entry's URL without adding a new entry.
    ///
    /// Invoked only from event handlers, so it runs on the client; a failed
    /// `history` lookup or `replaceState` call is swallowed rather than panics.
    fn replace_state(url: &str) {
        let Ok(history) = window().history() else {
            return;
        };
        _ = history.replace_state_with_url(&JsValue::NULL, "", Some(url));
    }
}

/// Accesses the [`UseHistory`] context provided by [`UseHistory::init`].
///
/// Expects [`UseHistory::init`] to have run in an ancestor; calling it outside
/// such a subtree is a programming error and aborts.
#[must_use]
pub fn use_history() -> UseHistory {
    expect_context::<UseHistory>()
}
