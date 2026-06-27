use crate::{clx, cn};
use leptos::html;
use leptos::prelude::*;
use leptos_icons::Icon;

clx! {
    /// Flex-wrap surface that lays out a set of [`ChipItem`]s, wrapping onto new
    /// rows as needed.
    ChipsContainer,
    div,
    "flex flex-wrap content-center items-center gap-2 rounded-xl border bg-card/5 p-3"
}

const CHIP_BASE: &str = "group inline-flex select-none items-center gap-1.5 rounded-full border bg-muted px-3 py-1 text-sm font-medium text-muted-foreground shadow-xs transition-colors outline-none cursor-pointer hover:bg-muted/80 focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 data-[state=selected]:border-warning data-[state=selected]:bg-warning data-[state=selected]:text-warning-foreground";

/// Selectable pill chip rendered as a toggle button. Drive its state from the
/// call site via the `selected` signal — the root reflects it through
/// `aria-pressed` and `data-[state=selected]`, and a check mark appears while
/// selected. All native attributes, events and bindings (e.g. `on:click`)
/// forward to the underlying `<button>`.
#[component]
pub fn ChipItem(
    #[prop(into)] label: Signal<String>,
    #[prop(into, optional)] selected: Signal<bool>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Button>,
) -> impl IntoView {
    view! {
        <button
            node_ref=node_ref
            data-name="ChipItem"
            type="button"
            aria-pressed=move || if selected.get() { "true" } else { "false" }
            data-state=move || if selected.get() { "selected" } else { "unselected" }
            class=move || cn!(CHIP_BASE, class.get())
        >
            <span>{move || label.get()}</span>
            <Icon
                icon=icondata::LuCheck
                attr:class="size-3.5 hidden group-data-[state=selected]:block"
            />
        </button>
    }
}
