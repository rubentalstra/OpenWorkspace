use crate::cn;
use leptos::html;
use leptos::prelude::*;

/// Scroll axis for a [`Marquee`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum MarqueeOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// CSS keyframes powering the marquee. Emitted once per [`Marquee`]; identical
/// blocks collapse harmlessly. Direction is derived from `data-` attributes so
/// the whole component stays CSS-only — no JavaScript drives the animation.
const MARQUEE_KEYFRAMES: &str = "\
@keyframes ow_marquee_x { from { transform: translateX(0); } to { transform: translateX(calc(-100% - var(--ow-marquee-gap))); } }\
@keyframes ow_marquee_y { from { transform: translateY(0); } to { transform: translateY(calc(-100% - var(--ow-marquee-gap))); } }\
[data-name=MarqueeRow] { animation: ow_marquee_x var(--ow-marquee-duration) linear infinite; }\
[data-name=Marquee][data-orientation=vertical] [data-name=MarqueeRow] { animation-name: ow_marquee_y; }\
[data-name=Marquee][data-reverse=true] [data-name=MarqueeRow] { animation-direction: reverse; }\
[data-name=Marquee][data-pause-on-hover=true]:hover [data-name=MarqueeRow] { animation-play-state: paused; }\
@media (prefers-reduced-motion: reduce) { [data-name=MarqueeRow] { animation: none; } }";

const MARQUEE_BASE: &str = "group flex overflow-hidden p-2 \
[--ow-marquee-gap:1rem] [gap:var(--ow-marquee-gap)] [--ow-marquee-duration:20s]";

const MARQUEE_ROW_BASE: &str = "flex shrink-0 justify-around [gap:var(--ow-marquee-gap)]";

/// CSS-animated scrolling marquee. The single set of `children` is repeated
/// `repeat` times across seamless rows so the loop appears continuous.
///
/// `orientation` picks the scroll axis, `reverse` flips the direction, and
/// `pause_on_hover` halts motion while the pointer is over the track. Tune speed
/// and spacing with `--ow-marquee-duration` / `--ow-marquee-gap` via `class` or
/// inline `style`. Respects `prefers-reduced-motion`. Native attributes, events
/// and bindings forward to the root.
#[component]
pub fn Marquee(
    #[prop(into, optional)] orientation: Signal<MarqueeOrientation>,
    #[prop(into, optional)] reverse: Signal<bool>,
    #[prop(into, default = Signal::derive(|| true))] pause_on_hover: Signal<bool>,
    #[prop(into, default = Signal::derive(|| 4u32))] repeat: Signal<u32>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: ChildrenFn,
) -> impl IntoView {
    let merged = move || {
        let axis = match orientation.get() {
            MarqueeOrientation::Horizontal => "flex-row",
            MarqueeOrientation::Vertical => "flex-col",
        };
        cn!(MARQUEE_BASE, axis, class.get())
    };
    let data_orientation = move || match orientation.get() {
        MarqueeOrientation::Horizontal => "horizontal",
        MarqueeOrientation::Vertical => "vertical",
    };
    let rows = move || {
        (0..repeat.get())
            .map(|_| {
                view! {
                    <div data-name="MarqueeRow" class=MARQUEE_ROW_BASE>
                        {children()}
                    </div>
                }
            })
            .collect_view()
    };

    view! {
        <div
            node_ref=node_ref
            data-name="Marquee"
            data-orientation=data_orientation
            data-reverse=move || reverse.get().to_string()
            data-pause-on-hover=move || pause_on_hover.get().to_string()
            class=merged
        >
            <style>{MARQUEE_KEYFRAMES}</style>
            {rows}
        </div>
    }
}

const MARQUEE_WRAPPER_BASE: &str = "relative flex h-full w-full flex-col items-center \
justify-center overflow-hidden bg-background p-20 min-h-[300px] md:shadow-xl";

/// Showcase frame around a [`Marquee`], adding gradient edge fades that blend the
/// scrolling content into the background. Native attributes, events and bindings
/// forward to the root.
#[component]
pub fn MarqueeWrapper(
    #[prop(into, optional)] orientation: Signal<MarqueeOrientation>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let merged = move || cn!(MARQUEE_WRAPPER_BASE, class.get());
    let edges = move || {
        match orientation.get() {
        MarqueeOrientation::Horizontal => view! {
            <div class="pointer-events-none absolute inset-y-0 left-0 w-1/4 bg-gradient-to-r from-background" />
            <div class="pointer-events-none absolute inset-y-0 right-0 w-1/4 bg-gradient-to-l from-background" />
        }
        .into_any(),
        MarqueeOrientation::Vertical => view! {
            <div class="pointer-events-none absolute inset-x-0 top-0 h-1/4 bg-gradient-to-b from-background" />
            <div class="pointer-events-none absolute inset-x-0 bottom-0 h-1/4 bg-gradient-to-t from-background" />
        }
        .into_any(),
    }
    };

    view! {
        <div node_ref=node_ref data-name="MarqueeWrapper" class=merged>
            {children()}
            {edges}
        </div>
    }
}
