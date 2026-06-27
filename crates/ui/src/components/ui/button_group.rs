use crate::{Separator, SeparatorOrientation, clx, cn};
use leptos::prelude::*;

const BUTTON_GROUP_BASE: &str = "flex w-fit items-stretch [&>*]:focus-visible:z-10 [&>*]:focus-visible:relative [&>[data-name=Select]:not([class*='w-'])]:w-fit [&>input]:flex-1 has-[>[data-name=ButtonGroup]]:gap-2";

const BUTTON_GROUP_SEPARATOR_BASE: &str =
    "relative !m-0 self-stretch data-[orientation=vertical]:h-auto";

clx! {
    /// Inline label or readonly segment styled to sit flush inside a [`ButtonGroup`].
    ButtonGroupText, span,
    "bg-muted flex items-center gap-2 rounded-md border px-4 text-sm font-medium shadow-xs [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4"
}

/// Joins a row or column of buttons into a single segmented control, collapsing
/// the shared inner edges so neighbours read as one control. `orientation`
/// switches between a horizontal row (default) and a vertical stack.
#[component]
pub fn ButtonGroup(
    #[prop(into, optional)] orientation: Signal<ButtonGroupOrientation>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        let axis = match orientation.get() {
            ButtonGroupOrientation::Horizontal => {
                "[&>*:not(:first-child)]:rounded-l-none [&>*:not(:first-child)]:border-l-0 [&>*:not(:last-child)]:rounded-r-none"
            }
            ButtonGroupOrientation::Vertical => {
                "flex-col [&>*:not(:first-child)]:rounded-t-none [&>*:not(:first-child)]:border-t-0 [&>*:not(:last-child)]:rounded-b-none"
            }
        };
        cn!(BUTTON_GROUP_BASE, axis, class.get())
    };

    view! {
        <div data-name="ButtonGroup" role="group" class=merged>
            {children()}
        </div>
    }
}

/// Layout axis along which a [`ButtonGroup`] joins its children.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Divider between segments of a [`ButtonGroup`]; defaults to a vertical rule.
#[component]
pub fn ButtonGroupSeparator(
    #[prop(into, optional, default = SeparatorOrientation::Vertical.into())] orientation: Signal<
        SeparatorOrientation,
    >,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let merged = move || cn!(BUTTON_GROUP_SEPARATOR_BASE, class.get());

    view! { <Separator attr:data-name="ButtonGroupSeparator" orientation=orientation class=merged /> }
}
