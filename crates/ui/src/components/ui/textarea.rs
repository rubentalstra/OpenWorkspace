use crate::cn;
use leptos::html;
use leptos::prelude::*;

const TEXTAREA_BASE: &str = "border-input placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 flex field-sizing-content min-h-16 w-full rounded-md border bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm";

/// Styled multi-line text field. Every native `<textarea>` attribute and event
/// is forwarded to the underlying element — set `attr:placeholder`,
/// `attr:rows`, `prop:value`, `on:input`, etc. at the call site (`bind:` is not
/// available on a component; pair `prop:value` with `on:input` for two-way
/// control).
#[component]
pub fn Textarea(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Textarea>,
) -> impl IntoView {
    view! {
        <textarea
            node_ref=node_ref
            data-name="Textarea"
            class=move || cn!(TEXTAREA_BASE, class.get())
        />
    }
}
