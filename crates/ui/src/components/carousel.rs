use crate::cn;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use leptos::prelude::*;
use leptos_icons::Icon;

/// Scroll axis of a [`Carousel`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum CarouselOrientation {
    /// Slides left/right (the default).
    #[default]
    Horizontal,
    /// Slides up/down.
    Vertical,
}

/// Carousel state shared with descendants via context. The upstream wraps embla;
/// this is a pure-Leptos slide engine — an `index` over registered items, advanced by
/// the prev/next buttons or arrow keys, applied as a `transform` on the track.
#[derive(Clone, Copy)]
pub struct CarouselContext {
    index: RwSignal<usize>,
    count: RwSignal<usize>,
    orientation: CarouselOrientation,
}

impl CarouselContext {
    /// Go to the previous slide (clamped at the start).
    pub fn scroll_prev(self) {
        self.index.update(|i| *i = i.saturating_sub(1));
    }

    /// Go to the next slide (clamped at the end).
    pub fn scroll_next(self) {
        let max = self.count.get_untracked().saturating_sub(1);
        self.index.update(|i| {
            if *i < max {
                *i += 1;
            }
        });
    }

    /// Whether a previous slide exists.
    #[must_use]
    pub fn can_scroll_prev(self) -> bool {
        self.index.get() > 0
    }

    /// Whether a next slide exists.
    #[must_use]
    pub fn can_scroll_next(self) -> bool {
        self.index.get() + 1 < self.count.get()
    }
}

/// Access the enclosing [`CarouselContext`]; panics outside a [`Carousel`].
#[must_use]
pub fn use_carousel() -> CarouselContext {
    expect_context::<CarouselContext>()
}

/// Carousel — shadcn Base UI `carousel`. A slide region navigated by the prev/next
/// buttons or left/right arrow keys.
#[component]
pub fn Carousel(
    #[prop(default = CarouselOrientation::Horizontal)] orientation: CarouselOrientation,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = CarouselContext {
        index: RwSignal::new(0),
        count: RwSignal::new(0),
        orientation,
    };
    provide_context(ctx);
    view! {
        <div
            role="region"
            aria-roledescription="carousel"
            data-slot="carousel"
            tabindex="0"
            class=move || cn!("relative", class.get())
            on:keydown=move |event| match event.key().as_str() {
                "ArrowLeft" => {
                    event.prevent_default();
                    ctx.scroll_prev();
                }
                "ArrowRight" => {
                    event.prevent_default();
                    ctx.scroll_next();
                }
                _ => {}
            }
        >
            {children()}
        </div>
    }
}

/// The clipped viewport + sliding track holding the [`CarouselItem`]s.
#[component]
pub fn CarouselContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = use_carousel();
    let track_class = match ctx.orientation {
        CarouselOrientation::Horizontal => "flex -ml-4",
        CarouselOrientation::Vertical => "flex -mt-4 flex-col",
    };
    let transform = move || {
        let i = ctx.index.get();
        match ctx.orientation {
            CarouselOrientation::Horizontal => format!("translateX(calc({i} * -100%))"),
            CarouselOrientation::Vertical => format!("translateY(calc({i} * -100%))"),
        }
    };
    view! {
        <div class="overflow-hidden" data-slot="carousel-content">
            <div
                class=move || cn!(track_class, class.get())
                style:transition="transform 0.3s ease-out"
                style:transform=transform
            >
                {children()}
            </div>
        </div>
    }
}

/// A single full-width (or full-height) slide. Registers itself so the carousel knows
/// when the next/previous edges are reached.
#[component]
pub fn CarouselItem(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = use_carousel();
    ctx.count.update(|c| *c += 1);
    on_cleanup(move || ctx.count.update(|c| *c = c.saturating_sub(1)));
    let item_class = match ctx.orientation {
        CarouselOrientation::Horizontal => "pl-4",
        CarouselOrientation::Vertical => "pt-4",
    };
    view! {
        <div
            role="group"
            aria-roledescription="slide"
            data-slot="carousel-item"
            class=move || cn!("min-w-0 shrink-0 grow-0 basis-full", item_class, class.get())
        >
            {children()}
        </div>
    }
}

/// The previous-slide button (outline icon); disabled at the start.
#[component]
pub fn CarouselPrevious(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let ctx = use_carousel();
    let pos = match ctx.orientation {
        CarouselOrientation::Horizontal => "inset-y-0 -left-12 my-auto",
        CarouselOrientation::Vertical => "-top-12 left-1/2 -translate-x-1/2 rotate-90",
    };
    view! {
        <Button
            variant=ButtonVariant::Outline
            size=ButtonSize::IconSm
            class=Signal::derive(move || {
                cn!("cn-carousel-previous absolute touch-manipulation", pos, class.get())
            })
            attr:data-slot="carousel-previous"
            attr:disabled=move || !ctx.can_scroll_prev()
            on:click=move |_| ctx.scroll_prev()
        >
            <Icon icon=icondata::LuChevronLeft attr:class="cn-rtl-flip" />
            <span class="sr-only">"Previous slide"</span>
        </Button>
    }
}

/// The next-slide button (outline icon); disabled at the end.
#[component]
pub fn CarouselNext(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let ctx = use_carousel();
    let pos = match ctx.orientation {
        CarouselOrientation::Horizontal => "inset-y-0 -right-12 my-auto",
        CarouselOrientation::Vertical => "-bottom-12 left-1/2 -translate-x-1/2 rotate-90",
    };
    view! {
        <Button
            variant=ButtonVariant::Outline
            size=ButtonSize::IconSm
            class=Signal::derive(move || {
                cn!("cn-carousel-next absolute touch-manipulation", pos, class.get())
            })
            attr:data-slot="carousel-next"
            attr:disabled=move || !ctx.can_scroll_next()
            on:click=move |_| ctx.scroll_next()
        >
            <Icon icon=icondata::LuChevronRight attr:class="cn-rtl-flip" />
            <span class="sr-only">"Next slide"</span>
        </Button>
    }
}
