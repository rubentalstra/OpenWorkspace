use crate::{Separator, clx, cn, variants};
use leptos::prelude::*;

clx! {
    /// Vertical stack grouping a series of [`Item`] rows.
    ItemGroup, div, "group/item-group flex flex-col"
}

clx! {
    /// Primary content column of an [`Item`] holding its title and description.
    ItemContent, div, "flex flex-1 flex-col gap-1 [&+[data-slot=item-content]]:flex-none"
}

clx! {
    /// Title line of an [`Item`].
    ItemTitle, div, "flex w-fit items-center gap-2 text-sm leading-snug font-medium"
}

clx! {
    /// Supporting description text for an [`Item`].
    ItemDescription, p, "text-muted-foreground line-clamp-2 text-sm leading-normal font-normal text-balance [&>a:hover]:text-primary [&>a]:underline [&>a]:underline-offset-4"
}

clx! {
    /// Trailing action slot of an [`Item`], e.g. buttons or a menu trigger.
    ItemActions, div, "flex items-center gap-2"
}

clx! {
    /// Full-width header band spanning the top of an [`Item`].
    ItemHeader, div, "flex basis-full items-center justify-between gap-2"
}

clx! {
    /// Full-width footer band spanning the bottom of an [`Item`].
    ItemFooter, div, "flex basis-full items-center justify-between gap-2"
}

variants! {
    /// Interactive list/menu row. Renders as a link when given `href`; `variant`
    /// and `size` control surface treatment and density.
    Item {
        base: "group/item flex items-center border border-transparent text-sm rounded-md transition-colors [a]:hover:bg-accent/50 [a]:transition-colors duration-100 flex-wrap outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]",
        variants: {
            variant: {
                Default: "bg-transparent",
                Outline: "border-border",
                Muted: "bg-muted/50",
            },
            size: {
                Default: "p-4 gap-4",
                Sm: "py-3 px-4 gap-2.5",
                Xs: "py-2 px-3 gap-2",
            }
        },
        component: {
            element: div,
            support_href: true
        }
    }
}

variants! {
    /// Leading media slot for an [`Item`] — an icon, avatar or thumbnail.
    ItemMedia {
        base: "flex shrink-0 items-center justify-center gap-2 group-has-[[data-slot=item-description]]/item:self-start [&_svg]:pointer-events-none group-has-[[data-slot=item-description]]/item:translate-y-0.5",
        variants: {
            variant: {
                Default: "bg-transparent",
                Icon: "size-8 border rounded-sm bg-muted [&_svg:not([class*='size-'])]:size-4",
                Image: "size-10 rounded-sm overflow-hidden [&_img]:size-full [&_img]:object-cover",
            },
            size: {
                Default: "",
            }
        },
        component: {
            element: div
        }
    }
}

/// Horizontal divider between items, with the vertical margin reset.
#[component]
pub fn ItemSeparator(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! { <Separator attr:data-name="ItemSeparator" class=move || cn!("my-0", class.get()) /> }
}
