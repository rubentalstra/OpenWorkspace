use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

/// Reactive vertical-scroll state for a scrollable element, exposed as scroll
/// affordances (typically used to fade or reveal top/bottom edges).
///
/// Returns `(on_scroll, can_scroll_up, can_scroll_down)`:
/// - `on_scroll`: attach to the element's `on:scroll`; it recomputes the
///   signals from the event target's scroll metrics.
/// - `can_scroll_up`: `true` once the element is scrolled away from the top.
/// - `can_scroll_down`: `true` while content remains below the viewport.
///
/// Both signals default to `false`, which is the correct server-render state
/// (no layout, nothing scrolled); the client populates them on the first
/// scroll event without any work at hook-invocation time.
pub fn use_can_scroll_vertical() -> (
    impl Fn(leptos::ev::Event) + Clone,
    RwSignal<bool>,
    RwSignal<bool>,
) {
    let can_scroll_up = RwSignal::new(false);
    let can_scroll_down = RwSignal::new(false);

    let on_scroll = move |ev: leptos::ev::Event| {
        let Some(target) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            return;
        };

        let scroll_top = target.scroll_top();
        let scroll_height = target.scroll_height();
        let client_height = target.client_height();

        can_scroll_up.set(scroll_top > 0);
        can_scroll_down.set(scroll_top < scroll_height - client_height - 1);
    };

    (on_scroll, can_scroll_up, can_scroll_down)
}
