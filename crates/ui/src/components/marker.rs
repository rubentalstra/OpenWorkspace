use crate::{cn, slot};
use leptos::prelude::*;

/// Tone of a [`Marker`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum MarkerVariant {
    /// The default marker.
    #[default]
    Default,
    /// A separator-style marker.
    Separator,
    /// A bordered marker.
    Border,
}

impl MarkerVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-marker-variant-default",
            Self::Separator => "cn-marker-variant-separator",
            Self::Border => "cn-marker-variant-border",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Separator => "separator",
            Self::Border => "border",
        }
    }
}

/// Marker — shadcn Base UI `marker`. A timeline/map marker row with an icon and content.
#[component]
pub fn Marker(
    #[prop(default = MarkerVariant::Default)] variant: MarkerVariant,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="marker"
            data-variant=variant.as_str()
            class=move || {
                cn!(
                    "cn-marker group/marker relative flex w-full items-center",
                    variant.class(),
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// The leading icon of a [`Marker`].
#[component]
pub fn MarkerIcon(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            data-slot="marker-icon"
            aria-hidden="true"
            class=move || cn!("cn-marker-icon shrink-0", class.get())
        >
            {children()}
        </span>
    }
}

slot! { MarkerContent, span, "marker-content", "cn-marker-content min-w-0 wrap-break-word" }
