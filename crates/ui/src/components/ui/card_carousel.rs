use crate::use_card_carousel;
use crate::{clx, void};
use leptos::prelude::*;

clx! {
    /// Fixed-size carousel frame that clips its track and hosts the hover nav.
    CardCarousel, div, "group rounded-[20px] overflow-hidden relative w-[320px] h-[320px] bg-gray-200"
}
clx! {
    /// Bottom gradient region holding the nav and indicators over the slides.
    CardCarouselOverlay, div, "pb-4 absolute bottom-0 flex flex-col justify-between items-center z-10 h-[calc(50%+32px)] w-full"
}
clx! {
    /// Previous/next button row, revealed on hover.
    CardCarouselNav, div, "opacity-0 invisible group-hover:visible group-hover:opacity-100 transition-opacity duration-[240ms] p-3 flex justify-between items-center w-full"
}
clx! {
    /// A round nav button; `aria-disabled` hides it at the track's edges.
    CardCarouselNavButton, button, "border-0 rounded-full cursor-pointer flex items-center justify-center size-8 [&_svg:not([class*='size-'])]:size-3 bg-accent transition-all duration-[160ms] ease-in-out hover:shadow-sm hover:scale-110 aria-[disabled]:invisible"
}
clx! {
    /// Row of indicator dots.
    CardCarouselIndicators, div, "gap-1 flex"
}
void! {
    /// A single indicator dot; `aria-current` marks the active slide.
    CardCarouselIndicator, span, "rounded-full size-[6px] bg-white opacity-60 aria-[current]:opacity-100"
}
clx! {
    /// One full-size, scroll-snapping slide.
    CardCarouselSlide, div, "snap-center shrink-0 w-full h-full"
}
void! {
    /// Cover image filling a [`CardCarouselSlide`].
    CardCarouselImage, img, "object-cover w-full h-full"
}

/// Scrollable, scroll-snapping track for the slides. Wiring the page-wide
/// carousel controller (nav buttons and indicator dots) happens on the client.
#[component]
pub fn CardCarouselTrack(children: Children) -> impl IntoView {
    use_card_carousel();

    view! {
        <div
            data-name="CardCarouselTrack"
            class="flex overflow-x-scroll w-full h-full snap-x snap-mandatory scroll-smooth touch-pan-x [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
        >
            {children()}
        </div>
    }
}
