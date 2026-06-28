use leptos::ev::PointerEvent;
use leptos::wasm_bindgen::JsCast;

/// Focus the menu item under the pointer so the nova layer's `focus:` highlight
/// rules fire on hover — mirroring Base UI's roving focus, where the hovered item
/// becomes the focused one. Attach to `on:pointermove` of each focusable item
/// (`<button>`s are focusable already; give `<div>` items `tabindex="-1"`).
pub fn focus_on_hover(ev: PointerEvent) {
    if let Some(el) = ev
        .current_target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = el.focus();
    }
}
