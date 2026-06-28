use crate::cn;
use leptos::prelude::*;

/// Input — shadcn Base UI `input`. A bare themed `<input>`; set `type`, `value`,
/// `placeholder`, `on:input`, `bind:value`, etc. at the call site (Leptos spreads
/// them onto the root). `node_ref` is exposed for imperative focus/measurement.
#[component]
pub fn Input(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Input>,
) -> impl IntoView {
    view! {
        <input
            node_ref=node_ref
            data-slot="input"
            class=move || {
                cn!(
                    "cn-input w-full min-w-0 outline-none file:inline-flex file:border-0 file:bg-transparent file:text-foreground placeholder:text-muted-foreground disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
                    class.get(),
                )
            }
        />
    }
}
