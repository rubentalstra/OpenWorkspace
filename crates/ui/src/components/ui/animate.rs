use crate::{cn, variants};
use leptos::html;
use leptos::prelude::*;

variants! {
    Animate {
        variants: {
            variant: {
                None: "",
                FadeUp: "opacity-0 animate-fade_up",
                ScrollFadeOut: "animate-fade_out_down [animation-range:0px_300px] [animation-timeline:scroll()] supports-no-scroll-driven-animations:animate-none",
                ScrollBigger: "animate-make_it_bigger [animation-range:0%_60%] [animation-timeline:--quote] [view-timeline-name:--quote] supports-no-scroll-driven-animations:animate-none",
            }
        }
    }
}

variants! {
    AnimateHover {
        variants: {
            variant: {
                None: "",
                Blink: "hover:animate-Blink",
                BlurredFadeIn: "hover:animate-BlurredFadeIn",
                BounceFadeIn: "hover:animate-BounceFadeIn",
                BounceHorizontal: "hover:animate-BounceHorizontal",
                BounceVertical: "hover:animate-BounceVertical",
                ContractHorizontally: "hover:animate-ContractHorizontally",
                ContractVertically: "hover:animate-ContractVertically",
                ExpandHorizontally: "hover:animate-ExpandHorizontally",
                ExpandVertically: "hover:animate-ExpandVertically",
                FadeIn: "hover:animate-FadeIn",
                FadeInDown: "hover:animate-FadeInDown",
                FadeInLeft: "hover:animate-FadeInLeft",
                FadeInRight: "hover:animate-FadeInRight",
                FadeInUp: "hover:animate-FadeInUp",
                FadeOut: "hover:animate-FadeOut",
                FadeOutUp: "hover:animate-FadeOutUp",
                FadeOutLeft: "hover:animate-FadeOutLeft",
                FadeOutRight: "hover:animate-FadeOutRight",
                Flash: "hover:animate-Flash",
                FlipHorizontal: "hover:animate-FlipHorizontal",
                FlipVertical: "hover:animate-FlipVertical",
                FlipX: "hover:animate-FlipX",
                FlipY: "hover:animate-FlipY",
                FlipInX: "hover:animate-FlipInX",
                FlipInY: "hover:animate-FlipInY",
                FlipOutX: "hover:animate-FlipOutX",
                FlipOutY: "hover:animate-FlipOutY",
                Float: "hover:animate-Float",
                Hang: "hover:animate-Hang",
                Heartbeat: "hover:animate-Heartbeat",
                HorizontalVibration: "hover:animate-HorizontalVibration",
                Jiggle: "hover:animate-Jiggle",
                Jump: "hover:animate-Jump",
                Pop: "hover:animate-Pop",
                PulseFadeIn: "hover:animate-PulseFadeIn",
                Rise: "hover:animate-Rise",
                RollIn: "hover:animate-RollIn",
                RollOut: "hover:animate-RollOut",
                Rotate90: "hover:animate-Rotate90",
                Rotate180: "hover:animate-Rotate180",
                Rotate360: "hover:animate-Rotate360",
                RotateIn: "hover:animate-RotateIn",
                RotateOut: "hover:animate-RotateOut",
                RotationalWave: "hover:animate-RotationalWave",
                RubberBand: "hover:animate-RubberBand",
                Shake: "hover:animate-Shake",
                Sink: "hover:animate-Sink",
                Skew: "hover:animate-Skew",
                SlideDown: "hover:animate-SlideDown",
                SlideDownAndFade: "hover:animate-SlideDownAndFade",
                SlideInBottom: "hover:animate-SlideInBottom",
                SlideInLeft: "hover:animate-SlideInLeft",
                SlideInRight: "hover:animate-SlideInRight",
                SlideInTop: "hover:animate-SlideInTop",
                SlideLeft: "hover:animate-SlideLeft",
                SlideLeftAndFade: "hover:animate-SlideLeftAndFade",
                SlideOutBottom: "hover:animate-SlideOutBottom",
                SlideOutLeft: "hover:animate-SlideOutLeft",
                SlideOutTop: "hover:animate-SlideOutTop",
                SlideRight: "hover:animate-SlideRight",
                SlideRightAndFade: "hover:animate-SlideRightAndFade",
                SlideRotateIn: "hover:animate-SlideRotateIn",
                SlideRotateOut: "hover:animate-SlideRotateOut",
                SlideUp: "hover:animate-SlideUp",
                SlideUpAndFade: "hover:animate-SlideUpAndFade",
                SlideUpFade: "hover:animate-SlideUpFade",
                SpinClockwise: "hover:animate-SpinClockwise",
                SpinCounterClockwise: "hover:animate-SpinCounterClockwise",
                Squeeze: "hover:animate-Squeeze",
                Sway: "hover:animate-Sway",
                Swing: "hover:animate-Swing",
                SwingDropIn: "hover:animate-SwingDropIn",
                Tada: "hover:animate-Tada",
                TiltHorizontal: "hover:animate-TiltHorizontal",
                Vibrate: "hover:animate-Vibrate",
                Wobble: "hover:animate-Wobble",
                ZoomIn: "hover:animate-ZoomIn",
                ZoomOut: "hover:animate-ZoomOut",
            }
        }
    }
}

const ANIMATE_BASE: &str = "flex w-full items-center justify-center";

/// Which animation frame an [`AnimateGroupItem`] retains before it starts and
/// after it ends, mapped to the CSS `animation-fill-mode` keyword.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationFillMode {
    None,
    Backwards,
    Both,
    #[default]
    Forwards,
}

impl AnimationFillMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Backwards => "backwards",
            Self::Both => "both",
            Self::Forwards => "forwards",
        }
    }
}

/// Wraps content in an animated container. `variant` drives an enter/scroll
/// animation, `hover_variant` an on-hover animation; both default to no
/// animation. Native attributes, events and bindings forward to the root.
#[component]
pub fn Animate(
    #[prop(into, optional)] variant: Signal<AnimateVariant>,
    #[prop(into, optional)] hover_variant: Signal<AnimateHoverVariant>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        cn!(
            ANIMATE_BASE,
            variant.get().class(),
            hover_variant.get().class(),
            class.get()
        )
    };

    view! {
        <div node_ref=node_ref data-name="Animate" class=merged>
            {children()}
        </div>
    }
}

const ANIMATE_GROUP_BASE: &str = "w-full";

/// Layout wrapper for a sequence of [`AnimateGroupItem`]s. Native attributes,
/// events and bindings forward to the root.
#[component]
pub fn AnimateGroup(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            data-name="AnimateGroup"
            class=move || cn!(ANIMATE_GROUP_BASE, class.get())
        >
            {children()}
        </div>
    }
}

/// A single animated child within an [`AnimateGroup`]. `delay_ms` staggers the
/// animation start and `fill_mode` controls the retained frame. Native
/// attributes, events and bindings forward to the root.
#[component]
pub fn AnimateGroupItem(
    #[prop(into, optional)] variant: Signal<AnimateVariant>,
    #[prop(into, optional)] hover_variant: Signal<AnimateHoverVariant>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] delay_ms: Signal<u32>,
    #[prop(into, optional)] fill_mode: Signal<AnimationFillMode>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        cn!(
            ANIMATE_BASE,
            variant.get().class(),
            hover_variant.get().class(),
            class.get()
        )
    };
    let style = move || {
        format!(
            "animation-delay: {}ms; animation-fill-mode: {};",
            delay_ms.get(),
            fill_mode.get().as_str()
        )
    };

    view! {
        <div node_ref=node_ref data-name="AnimateGroupItem" class=merged style=style>
            {children()}
        </div>
    }
}
