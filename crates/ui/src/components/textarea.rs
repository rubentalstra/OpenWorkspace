use crate::cn;
use leptos::prelude::*;

/// Textarea — shadcn Base UI `textarea`. Auto-sizes to its content. Set `value`,
/// `placeholder`, `on:input`, `bind:value`, etc. at the call site.
#[component]
pub fn Textarea(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Textarea>,
) -> impl IntoView {
    view! {
        <textarea
            node_ref=node_ref
            data-slot="textarea"
            class=move || {
                cn!(
                    "cn-textarea flex field-sizing-content min-h-16 w-full outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50",
                    class.get(),
                )
            }
        />
    }
}
