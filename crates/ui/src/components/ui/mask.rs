use crate::{clx, cn};
use leptos::prelude::*;

clx! {
    /// Bordered, centered stage that clips its contents — the canvas a [`Mask`]
    /// fades against (e.g. a marquee's edges).
    MaskWrapper,
    div,
    "relative flex h-full min-h-[300px] w-full items-center justify-center overflow-hidden rounded-lg border"
}

/// Which edge of the container the gradient fades from.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum MaskSide {
    #[default]
    None,
    Left,
    Right,
    Top,
    Bottom,
}

impl MaskSide {
    fn class(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Left => "inset-y-0 left-0 w-1/3 bg-gradient-to-r",
            Self::Right => "inset-y-0 right-0 w-1/3 bg-gradient-to-l",
            Self::Top => "inset-x-0 top-0 h-1/3 bg-gradient-to-b",
            Self::Bottom => "inset-x-0 bottom-0 h-1/3 bg-gradient-to-t",
        }
    }
}

/// Edge fade overlay. Renders a non-interactive gradient pinned to the chosen
/// `side`, fading from the surface colour to transparent so content scrolling
/// underneath dissolves at that edge.
#[component]
pub fn Mask(
    #[prop(into, optional)] side: Signal<MaskSide>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let merged = move || {
        cn!(
            "pointer-events-none absolute from-white dark:from-background",
            side.get().class(),
            class.get()
        )
    };

    view! { <div data-name="Mask" class=merged /> }
}
