use crate::cn;
use leptos::html;
use leptos::prelude::*;

const ASPECT_RATIO_BASE: &str = "relative w-full overflow-hidden";

/// Box that constrains its content to a fixed width-to-height `ratio`
/// (defaults to 16:9). Native attributes, events and bindings forward to the
/// root element.
#[component]
pub fn AspectRatio(
    #[prop(into, default = Signal::derive(|| 16.0 / 9.0))] ratio: Signal<f64>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let style = move || format!("aspect-ratio: {}", ratio.get());

    view! {
        <div
            node_ref=node_ref
            data-name="AspectRatio"
            class=move || cn!(ASPECT_RATIO_BASE, class.get())
            style=style
        >
            {children()}
        </div>
    }
}
