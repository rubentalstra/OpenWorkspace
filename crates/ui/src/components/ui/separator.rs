use crate::cn;
use leptos::prelude::*;

/// Visual divider. Horizontal full-width by default; pass
/// `orientation=SeparatorOrientation::Vertical` for a full-height rule.
#[component]
pub fn Separator(
    #[prop(into, optional)] orientation: Signal<SeparatorOrientation>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let merged = move || {
        let axis = match orientation.get() {
            SeparatorOrientation::Horizontal => "h-px w-full",
            SeparatorOrientation::Vertical => "h-full w-px",
        };
        cn!("shrink-0 bg-border", axis, class.get())
    };
    let aria_orientation = move || match orientation.get() {
        SeparatorOrientation::Horizontal => "horizontal",
        SeparatorOrientation::Vertical => "vertical",
    };

    view! {
        <div
            data-name="Separator"
            role="separator"
            aria-orientation=aria_orientation
            class=merged
        />
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}
