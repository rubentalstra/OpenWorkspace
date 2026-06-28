use crate::{cn, slot};
use leptos::prelude::*;

slot! { BubbleGroup, div, "bubble-group", "cn-bubble-group flex min-w-0 flex-col" }

/// Tone of a [`Bubble`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BubbleVariant {
    /// The default bubble.
    #[default]
    Default,
    /// A secondary tone.
    Secondary,
    /// A muted tone.
    Muted,
    /// A tinted tone.
    Tinted,
    /// An outlined bubble.
    Outline,
    /// A ghost (transparent) bubble.
    Ghost,
    /// A destructive tone.
    Destructive,
}

impl BubbleVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-bubble-variant-default",
            Self::Secondary => "cn-bubble-variant-secondary",
            Self::Muted => "cn-bubble-variant-muted",
            Self::Tinted => "cn-bubble-variant-tinted",
            Self::Outline => "cn-bubble-variant-outline",
            Self::Ghost => "cn-bubble-variant-ghost",
            Self::Destructive => "cn-bubble-variant-destructive",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Secondary => "secondary",
            Self::Muted => "muted",
            Self::Tinted => "tinted",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
            Self::Destructive => "destructive",
        }
    }
}

/// Horizontal alignment of a chat element (sender side).
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BubbleAlign {
    /// Aligned to the start (incoming).
    #[default]
    Start,
    /// Aligned to the end (outgoing).
    End,
}

impl BubbleAlign {
    fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// Bubble — shadcn Base UI `bubble`. A chat message bubble.
#[component]
pub fn Bubble(
    #[prop(default = BubbleVariant::Default)] variant: BubbleVariant,
    #[prop(default = BubbleAlign::Start)] align: BubbleAlign,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="bubble"
            data-variant=variant.as_str()
            data-align=align.as_str()
            class=move || {
                cn!(
                    "cn-bubble group/bubble relative flex w-fit min-w-0 flex-col",
                    variant.class(),
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

slot! {
    BubbleContent, div, "bubble-content",
    "cn-bubble-content w-fit max-w-full min-w-0 overflow-hidden wrap-break-word [button]:text-left [button,a]:transition-colors"
}

/// Which edge of the bubble the reactions sit on.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BubbleReactionsSide {
    /// Above the bubble.
    Top,
    /// Below the bubble (the default).
    #[default]
    Bottom,
}

impl BubbleReactionsSide {
    fn class(self) -> &'static str {
        match self {
            Self::Top => "cn-bubble-reactions-side-top",
            Self::Bottom => "cn-bubble-reactions-side-bottom",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

/// A floating cluster of reactions anchored to a [`Bubble`].
#[component]
pub fn BubbleReactions(
    #[prop(default = BubbleReactionsSide::Bottom)] side: BubbleReactionsSide,
    #[prop(default = BubbleAlign::End)] align: BubbleAlign,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let align_class = match align {
        BubbleAlign::Start => "cn-bubble-reactions-align-start",
        BubbleAlign::End => "cn-bubble-reactions-align-end",
    };
    view! {
        <div
            data-slot="bubble-reactions"
            data-align=align.as_str()
            data-side=side.as_str()
            class=move || {
                cn!(
                    "cn-bubble-reactions absolute z-10 flex w-fit items-center justify-center",
                    side.class(),
                    align_class,
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}
