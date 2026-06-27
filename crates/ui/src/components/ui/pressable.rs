use crate::cn;
use leptos::html;
use leptos::prelude::*;

const PRESSABLE_BASE: &str =
    "transition-transform touch-manipulation [-webkit-tap-highlight-color:transparent] select-none";

/// Press-feedback wrapper that scales its children down while held, giving
/// tactile feedback on touch devices where the `:active` pseudo-class is
/// unreliable on non-interactive elements. Native attributes, events and
/// bindings forward to the root `<div>`.
#[component]
pub fn Pressable(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let pressed = RwSignal::new(false);
    let merged = move || {
        cn!(
            PRESSABLE_BASE,
            pressed.get().then_some("scale-[0.98]"),
            class.get()
        )
    };

    view! {
        <div
            node_ref=node_ref
            data-name="Pressable"
            class=merged
            on:pointerdown=move |_| pressed.set(true)
            on:pointerup=move |_| pressed.set(false)
            on:pointerleave=move |_| pressed.set(false)
            on:pointercancel=move |_| pressed.set(false)
        >
            {children()}
        </div>
    }
}
