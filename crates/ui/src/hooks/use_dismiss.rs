use leptos::ev;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

/// Wire "dismiss on outside-pointerdown or Escape" for an anchored overlay.
///
/// `root` must wrap **both** the trigger and the popup, so clicking either counts
/// as inside and does not dismiss. Listeners are registered on `window` and torn
/// down on cleanup; on the server they are no-ops (no `window`).
pub fn use_dismiss(open: RwSignal<bool>, root: NodeRef<leptos::html::Div>) {
    let on_pointer = window_event_listener(ev::pointerdown, move |event| {
        if !open.get_untracked() {
            return;
        }
        let Some(root_el) = root.get_untracked() else {
            return;
        };
        let inside = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Node>().ok())
            .is_some_and(|node| root_el.contains(Some(&node)));
        if !inside {
            open.set(false);
        }
    });
    let on_key = window_event_listener(ev::keydown, move |event| {
        if open.get_untracked() && event.key() == "Escape" {
            open.set(false);
        }
    });
    on_cleanup(move || {
        on_pointer.remove();
        on_key.remove();
    });
}
