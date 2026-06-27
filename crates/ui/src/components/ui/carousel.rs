use crate::{cn, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos_icons::Icon;

/// Layout axis for a [`Carousel`]. `Horizontal` slides translate along the x
/// axis and bind the left/right arrow keys; `Vertical` slides translate along
/// the y axis and bind the up/down arrow keys.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl CarouselOrientation {
    fn as_data(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// Slide state shared from [`Carousel`] to its parts. `index` is the active
/// slide; `count` is incremented as each [`CarouselItem`] mounts so the
/// controls and indicator can reason about bounds without measuring the DOM.
#[derive(Clone, Copy)]
struct CarouselContext {
    index: RwSignal<usize>,
    count: RwSignal<usize>,
    orientation: CarouselOrientation,
    looping: bool,
}

impl CarouselContext {
    fn can_prev(&self) -> bool {
        self.looping || self.index.get() > 0
    }

    fn can_next(&self) -> bool {
        let count = self.count.get();
        self.looping || (count > 0 && self.index.get() + 1 < count)
    }

    fn go_prev(&self) {
        let count = self.count.get();
        self.index.update(|i| {
            if *i > 0 {
                *i -= 1;
            } else if self.looping && count > 0 {
                *i = count - 1;
            }
        });
    }

    fn go_next(&self) {
        let count = self.count.get();
        self.index.update(|i| {
            if *i + 1 < count {
                *i += 1;
            } else if self.looping {
                *i = 0;
            }
        });
    }
}

/// Slide carousel. Owns the active-slide index and shares it with the nested
/// [`CarouselContent`], [`CarouselItem`]s, [`CarouselPrevious`],
/// [`CarouselNext`] and [`CarouselIndicator`] through context. The active slide
/// is driven by a reactive transform — no scrolling and no JavaScript. Arrow
/// keys move between slides along the configured axis.
#[component]
pub fn Carousel(
    #[prop(into, optional)] orientation: CarouselOrientation,
    /// When set, advancing past the last slide wraps to the first (and vice
    /// versa) and the controls stay enabled at the ends.
    #[prop(optional)]
    looping: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let carousel_id = use_random_id_for("carousel");
    let ctx = CarouselContext {
        index: RwSignal::new(0),
        count: RwSignal::new(0),
        orientation,
        looping,
    };

    let on_keydown = move |ev: KeyboardEvent| {
        let (prev, next) = match orientation {
            CarouselOrientation::Horizontal => ("ArrowLeft", "ArrowRight"),
            CarouselOrientation::Vertical => ("ArrowUp", "ArrowDown"),
        };
        let key = ev.key();
        if key == prev {
            ev.prevent_default();
            ctx.go_prev();
        } else if key == next {
            ev.prevent_default();
            ctx.go_next();
        }
    };

    view! {
        <Provider value=ctx>
            <div
                data-name="Carousel"
                data-carousel-id=carousel_id
                data-carousel-orientation=orientation.as_data()
                data-carousel-loop=looping.to_string()
                class=move || cn!("relative", class.get())
                role="region"
                aria-roledescription="carousel"
                tabindex="0"
                on:keydown=on_keydown
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Viewport that clips the slide track and translates it so the active
/// [`CarouselItem`] is shown. The transform animates between slides.
#[component]
pub fn CarouselContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CarouselContext>();

    let track_axis = match ctx.orientation {
        CarouselOrientation::Horizontal => "flex",
        CarouselOrientation::Vertical => "flex flex-col",
    };

    let transform = move || {
        let offset = ctx.index.get() * 100;
        match ctx.orientation {
            CarouselOrientation::Horizontal => format!("transform: translateX(-{offset}%)"),
            CarouselOrientation::Vertical => format!("transform: translateY(-{offset}%)"),
        }
    };

    view! {
        <div data-name="CarouselContent" aria-live="polite" class="overflow-hidden">
            <div
                class=move || {
                    cn!(track_axis, "transition-transform duration-300 ease-out", class.get())
                }
                style=transform
            >
                {children()}
            </div>
        </div>
    }
}

/// A single slide. Claims its slot in the track when it mounts and releases it
/// on cleanup so [`Carousel`] always knows the live slide count. Slides that are
/// not active are hidden from assistive technology and removed from the tab
/// order so a screen reader never reads an off-screen slide.
#[component]
pub fn CarouselItem(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CarouselContext>();

    let slot = StoredValue::new(ctx.count.get_untracked());
    ctx.count.update(|n| *n += 1);
    on_cleanup(move || {
        ctx.count.update(|n| *n = n.saturating_sub(1));
    });

    let is_active = Memo::new(move |_| ctx.index.get() == slot.get_value());

    view! {
        <div
            data-name="CarouselItem"
            role="group"
            aria-roledescription="slide"
            aria-hidden=move || (!is_active.get()).then_some("true")
            inert=move || !is_active.get()
            class=move || cn!("min-w-0 shrink-0 grow-0 basis-full", class.get())
        >
            {children()}
        </div>
    }
}

/// Button moving to the previous slide. Disabled at the first slide unless the
/// enclosing [`Carousel`] loops.
#[component]
pub fn CarouselPrevious(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let ctx = expect_context::<CarouselContext>();

    let position = match ctx.orientation {
        CarouselOrientation::Horizontal => "top-1/2 -left-12 -translate-y-1/2",
        CarouselOrientation::Vertical => "-top-12 left-1/2 -translate-x-1/2 rotate-90",
    };

    view! {
        <button
            type="button"
            data-name="CarouselPrevious"
            class=move || {
                cn!(
                    "absolute inline-flex items-center justify-center size-8 rounded-full border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground cursor-pointer touch-manipulation disabled:pointer-events-none disabled:opacity-50",
                    position,
                    class.get(),
                )
            }
            aria-label="Previous slide"
            disabled=move || !ctx.can_prev()
            on:click=move |_| ctx.go_prev()
        >
            <Icon icon=icondata::LuChevronLeft attr:class="size-4" />
            <span class="sr-only">"Previous slide"</span>
        </button>
    }
}

/// Button moving to the next slide. Disabled at the last slide unless the
/// enclosing [`Carousel`] loops.
#[component]
pub fn CarouselNext(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let ctx = expect_context::<CarouselContext>();

    let position = match ctx.orientation {
        CarouselOrientation::Horizontal => "top-1/2 -right-12 -translate-y-1/2",
        CarouselOrientation::Vertical => "-bottom-12 left-1/2 -translate-x-1/2 rotate-90",
    };

    view! {
        <button
            type="button"
            data-name="CarouselNext"
            class=move || {
                cn!(
                    "absolute inline-flex items-center justify-center size-8 rounded-full border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground cursor-pointer touch-manipulation disabled:pointer-events-none disabled:opacity-50",
                    position,
                    class.get(),
                )
            }
            aria-label="Next slide"
            disabled=move || !ctx.can_next()
            on:click=move |_| ctx.go_next()
        >
            <Icon icon=icondata::LuChevronRight attr:class="size-4" />
            <span class="sr-only">"Next slide"</span>
        </button>
    }
}

/// Live "current / total" slide readout, recomputed from the shared index and
/// slide count.
#[component]
pub fn CarouselIndicator(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let ctx = expect_context::<CarouselContext>();

    let label = move || {
        let count = ctx.count.get();
        let current = if count == 0 { 0 } else { ctx.index.get() + 1 };
        format!("{current} / {count}")
    };

    view! {
        <div
            data-name="CarouselIndicator"
            aria-live="polite"
            class=move || cn!("py-2 text-center text-sm text-muted-foreground", class.get())
        >
            {label}
        </div>
    }
}
