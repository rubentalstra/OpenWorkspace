use crate::cn;
use leptos::html;
use leptos::prelude::*;

const KBD_BASE: &str = "bg-muted text-muted-foreground pointer-events-none inline-flex h-5 w-fit min-w-5 items-center justify-center gap-1 rounded-sm px-1 font-sans text-xs font-medium select-none [&_svg:not([class*='size-'])]:size-3 [[data-slot=tooltip-content]_&]:bg-background/20 [[data-slot=tooltip-content]_&]:text-background dark:[[data-slot=tooltip-content]_&]:bg-background/10";

/// Inline keyboard key, e.g. a single `Ctrl` or `K` cap. Forwards native
/// attributes, events and children to the underlying `<kbd>`.
#[component]
pub fn Kbd(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Kbd>,
    children: Children,
) -> impl IntoView {
    view! {
        <kbd node_ref=node_ref data-name="Kbd" class=move || cn!(KBD_BASE, class.get())>
            {children()}
        </kbd>
    }
}

/// Row wrapper that lays out several [`Kbd`] caps as one shortcut, e.g.
/// `Ctrl` + `K`. Forwards native attributes, events and children.
#[component]
pub fn KbdGroup(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Kbd>,
    children: Children,
) -> impl IntoView {
    view! {
        <kbd
            node_ref=node_ref
            data-name="KbdGroup"
            class=move || cn!("inline-flex items-center gap-1", class.get())
        >
            {children()}
        </kbd>
    }
}
