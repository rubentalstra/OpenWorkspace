use crate::components::separator::{Separator, SeparatorOrientation};
use crate::{cn, slot};
use leptos::prelude::*;

/// Button-group orientation.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl ButtonGroupOrientation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
    fn class(self) -> &'static str {
        match self {
            Self::Horizontal => {
                "cn-button-group-orientation-horizontal *:data-slot:rounded-r-none [&>[data-slot]~[data-slot]]:rounded-l-none [&>[data-slot]~[data-slot]]:border-l-0"
            }
            Self::Vertical => {
                "cn-button-group-orientation-vertical flex-col *:data-slot:rounded-b-none [&>[data-slot]~[data-slot]]:rounded-t-none [&>[data-slot]~[data-slot]]:border-t-0"
            }
        }
    }
}

/// ButtonGroup — shadcn Base UI `button-group`. Joins adjacent buttons/controls.
#[component]
pub fn ButtonGroup(
    #[prop(into, optional)] orientation: Signal<ButtonGroupOrientation>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            role="group"
            data-slot="button-group"
            data-orientation=move || orientation.get().as_str()
            class=move || {
                cn!(
                    "cn-button-group flex w-fit items-stretch *:focus-visible:relative *:focus-visible:z-10 [&>[data-slot=select-trigger]:not([class*='w-'])]:w-fit [&>input]:flex-1",
                    orientation.get().class(),
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

slot! {
    ButtonGroupText, div, "button-group-text",
    "cn-button-group-text flex items-center [&_svg]:pointer-events-none"
}

/// A separator between grouped buttons (vertical by default).
#[component]
pub fn ButtonGroupSeparator(
    #[prop(into, optional, default = SeparatorOrientation::Vertical.into())] orientation: Signal<
        SeparatorOrientation,
    >,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <Separator
            orientation=orientation
            class=Signal::derive(move || {
                cn!(
                    "cn-button-group-separator relative self-stretch data-horizontal:mx-px data-horizontal:w-auto data-vertical:my-px data-vertical:h-auto",
                    class.get(),
                )
            })
        />
    }
}
