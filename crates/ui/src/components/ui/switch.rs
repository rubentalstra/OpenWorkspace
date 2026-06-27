use crate::{clx, cn};
use leptos::html;
use leptos::prelude::*;

const SWITCH_BASE: &str = "peer inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-full border border-transparent shadow-xs transition-all outline-none disabled:cursor-not-allowed disabled:opacity-50 focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-checked:bg-primary aria-[checked=false]:bg-input dark:aria-[checked=false]:bg-input/80";

const SWITCH_THUMB: &str = "pointer-events-none block size-4 rounded-full bg-background ring-0 shadow-lg transition-transform peer-aria-checked:translate-x-[calc(100%-2px)] peer-aria-[checked=false]:translate-x-0 dark:peer-aria-checked:bg-primary-foreground dark:peer-aria-[checked=false]:bg-foreground";

clx! {SwitchLabel, span, "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-50"}

/// Toggle switch styled as a `<button role="switch">`. Drive its state via the
/// forwarded `aria-checked` attribute and handle `on:click` at the call site;
/// the track and thumb restyle from `aria-checked` selectors.
#[component]
pub fn Switch(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Button>,
) -> impl IntoView {
    view! {
        <button
            node_ref=node_ref
            data-name="Switch"
            type="button"
            role="switch"
            class=move || cn!(SWITCH_BASE, class.get())
        >
            <span data-name="SwitchThumb" class=SWITCH_THUMB />
        </button>
    }
}
