use crate::{clx, cn, void};
use leptos::prelude::*;

void! {
    /// Draggable scrollbar thumb rendered inside a [`ScrollBar`].
    ScrollAreaThumb, div, "bg-border relative flex-1 rounded-full"
}
void! {
    /// Filler shown where a horizontal and vertical [`ScrollBar`] meet.
    ScrollAreaCorner, div, "bg-border"
}

clx! {
    /// Scroll container with a styled viewport. Use [`ScrollBar`] children for
    /// the decorative scrollbars and [`ScrollAreaCorner`] for the junction.
    ScrollArea, div, "relative overflow-hidden"
}
clx! {
    /// Scrollable viewport region inside a [`ScrollArea`].
    ScrollAreaViewport,
    div,
    "focus-visible:ring-ring/50 size-full rounded-[inherit] transition-[color,box-shadow] outline-none focus-visible:ring-[3px] focus-visible:outline-1 overflow-auto"
}

/// Decorative scrollbar track. `orientation` selects a vertical or horizontal rail.
#[component]
pub fn ScrollBar(
    #[prop(into, optional)] orientation: Signal<ScrollBarOrientation>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let merged = move || {
        let axis = match orientation.get() {
            ScrollBarOrientation::Vertical => "h-full w-2.5 border-l border-l-transparent",
            ScrollBarOrientation::Horizontal => "h-2.5 flex-col border-t border-t-transparent",
        };
        cn!(
            "flex touch-none p-px transition-colors select-none",
            axis,
            class.get()
        )
    };
    let aria_orientation = move || match orientation.get() {
        ScrollBarOrientation::Vertical => "vertical",
        ScrollBarOrientation::Horizontal => "horizontal",
    };

    view! {
        <div data-name="ScrollBar" aria-orientation=aria_orientation class=merged>
            <ScrollAreaThumb />
        </div>
    }
}

/// Axis of a [`ScrollBar`] rail.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ScrollBarOrientation {
    /// Rail runs top to bottom along the right edge.
    #[default]
    Vertical,
    /// Rail runs left to right along the bottom edge.
    Horizontal,
}

clx! {
    /// Horizontal scroll container with CSS scroll-snap on the x axis. Wrap each
    /// child in a [`SnapItem`] to define snap points.
    SnapScrollArea, div, "overflow-x-auto snap-x"
}

/// Snap target inside a [`SnapScrollArea`]; `alignment` sets where it rests.
#[component]
pub fn SnapItem(
    #[prop(into, optional)] alignment: Signal<SnapAlignment>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        let snap = match alignment.get() {
            SnapAlignment::Center => "snap-center",
            SnapAlignment::Start => "snap-start",
            SnapAlignment::End => "snap-end",
        };
        cn!("shrink-0", snap, class.get())
    };

    view! {
        <div data-name="SnapItem" class=merged>
            {children()}
        </div>
    }
}

/// Resting position of a [`SnapItem`] within its [`SnapScrollArea`].
#[derive(Clone, Copy, PartialEq, Default)]
pub enum SnapAlignment {
    /// Item centers in the viewport when snapped.
    #[default]
    Center,
    /// Item aligns to the start edge when snapped.
    Start,
    /// Item aligns to the end edge when snapped.
    End,
}
