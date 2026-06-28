use crate::components::separator::{Separator, SeparatorOrientation};
use crate::{cn, slot, variants};
use leptos::prelude::*;

variants! {
    /// Item — shadcn Base UI `item`. A flexible list/row primitive.
    Item {
        slot: "item",
        base: "cn-item group/item flex w-full flex-wrap items-center transition-colors duration-100 outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 [a]:transition-colors",
        variants: {
            variant: {
                Default: "cn-item-variant-default",
                Outline: "cn-item-variant-outline",
                Muted: "cn-item-variant-muted",
            },
            size: {
                Default: "cn-item-size-default",
                Sm: "cn-item-size-sm",
                Xs: "cn-item-size-xs",
            }
        },
        component: { element: div }
    }
}

/// Media slot kind for [`ItemMedia`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ItemMediaVariant {
    #[default]
    Default,
    Icon,
    Image,
}

impl ItemMediaVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Icon => "icon",
            Self::Image => "image",
        }
    }
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-item-media-variant-default",
            Self::Icon => "cn-item-media-variant-icon",
            Self::Image => "cn-item-media-variant-image",
        }
    }
}

/// Leading media (icon, avatar, image) for an `Item`.
#[component]
pub fn ItemMedia(
    #[prop(into, optional)] variant: Signal<ItemMediaVariant>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="item-media"
            data-variant=move || variant.get().as_str()
            class=move || {
                cn!(
                    "cn-item-media flex shrink-0 items-center justify-center [&_svg]:pointer-events-none",
                    variant.get().class(),
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// A group of `Item`s with list semantics.
#[component]
pub fn ItemGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            role="list"
            data-slot="item-group"
            class=move || cn!("cn-item-group group/item-group flex w-full flex-col", class.get())
        >
            {children()}
        </div>
    }
}

/// A horizontal rule between items.
#[component]
pub fn ItemSeparator(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <Separator
            orientation=SeparatorOrientation::Horizontal
            class=Signal::derive(move || cn!("cn-item-separator", class.get()))
        />
    }
}

slot! {
    ItemContent, div, "item-content",
    "cn-item-content flex flex-1 flex-col [&+[data-slot=item-content]]:flex-none"
}
slot! { ItemTitle, div, "item-title", "cn-item-title line-clamp-1 flex w-fit items-center" }
slot! {
    ItemDescription, p, "item-description",
    "cn-item-description line-clamp-2 font-normal [&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-primary"
}
slot! { ItemActions, div, "item-actions", "cn-item-actions flex items-center" }
slot! {
    ItemHeader, div, "item-header",
    "cn-item-header flex basis-full items-center justify-between"
}
slot! {
    ItemFooter, div, "item-footer",
    "cn-item-footer flex basis-full items-center justify-between"
}
