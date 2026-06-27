use crate::cn;
use leptos::html;
use leptos::prelude::*;

const INPUT_BASE: &str = "text-foreground file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input flex h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-2 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive read-only:bg-muted";

/// Styled text input. Every native `<input>` attribute, event and binding is
/// forwarded to the underlying element — set `attr:type`, `attr:placeholder`,
/// `bind:value`, `on:input`, etc. at the call site.
#[component]
pub fn Input(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Input>,
) -> impl IntoView {
    view! { <input node_ref=node_ref data-name="Input" class=move || cn!(INPUT_BASE, class.get()) /> }
}
