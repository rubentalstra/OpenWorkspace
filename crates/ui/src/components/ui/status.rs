use crate::{cn, variants};
use leptos::prelude::*;

variants! {
    StatusIndicator {
        variants: {
            variant: {
                Default: "bg-neutral-300",
                Active: "bg-green-300",
                Inactive: "bg-orange-300",
                Normal: "bg-sky-300",
            }
        }
    }
}

const STATUS_INDICATOR_BASE: &str = "rounded-full size-4 shrink-0";

/// Coloured status dot. `variant` selects the colour; native attributes forward
/// to the underlying element.
#[component]
pub fn StatusIndicator(
    #[prop(into, optional)] variant: Signal<StatusIndicatorVariant>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let merged = move || cn!(STATUS_INDICATOR_BASE, variant.get().class(), class.get());
    view! { <div data-name="StatusIndicator" class=merged /> }
}

/// Presence indicator: a static [`StatusIndicator`] dot with a pinging copy
/// behind it, anchored to the top-right of its `children`.
#[component]
pub fn Status(
    #[prop(into, optional)] variant: Signal<StatusIndicatorVariant>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let anchor = "absolute top-0 right-0 -mt-1 -mr-1";

    view! {
        <div data-name="Status" class=move || cn!("relative", class.get())>
            {children()}
            <StatusIndicator
                variant=variant
                class=Signal::derive(move || cn!(anchor, "animate-ping"))
            />
            <StatusIndicator variant=variant class=Signal::derive(move || anchor.to_string()) />
        </div>
    }
}
