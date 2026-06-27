use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;

/// Reactively tracks whether a CSS media query matches the current viewport.
///
/// Returns a [`Signal<bool>`] that is `false` during server render and on the
/// first client paint, then resolves to the live `MediaQueryList` result and
/// updates whenever the match state changes (e.g. on resize or orientation
/// change). The listener is registered inside an effect, so it never runs during
/// SSR; the browser closure is owned by the page for its lifetime.
pub fn use_media_query(query: &str) -> Signal<bool> {
    let matches = RwSignal::new(false);
    let query = query.to_owned();

    Effect::new(move |_| {
        let Some(mql) = window().match_media(&query).ok().flatten() else {
            return;
        };
        matches.set(mql.matches());

        let source = mql.clone();
        let on_change = Closure::<dyn Fn()>::new(move || matches.set(source.matches()));
        _ = mql.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
        // The browser keeps only a weak reference, so the closure must outlive
        // this scope; it lives for the page's lifetime alongside the query.
        on_change.forget();
    });

    matches.into()
}
