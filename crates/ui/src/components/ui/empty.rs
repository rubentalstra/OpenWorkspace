use crate::{clx, variants};
use leptos::prelude::*;

clx! {
    /// Empty-state placeholder surface centring its header, media and actions.
    Empty, div, "flex flex-col items-center justify-center gap-4 rounded-lg border border-dashed p-8 text-center"
}

clx! {
    /// Stacked title and description block within an [`Empty`] surface.
    EmptyHeader, div, "flex flex-col items-center gap-2"
}

clx! {
    /// Heading for an empty state.
    EmptyTitle, h3, "text-lg font-semibold leading-none"
}

clx! {
    /// Supporting copy for an empty state.
    EmptyDescription, p, "text-muted-foreground text-sm"
}

clx! {
    /// Action row (e.g. buttons) rendered beneath an empty state's header.
    EmptyContent, div, "flex items-center justify-center gap-2"
}

variants! {
    /// Leading visual for an empty state; frames an icon in a muted tile.
    EmptyMedia {
        base: "flex shrink-0 items-center justify-center mb-2 [&_svg]:pointer-events-none [&_svg]:shrink-0",
        variants: {
            variant: {
                Default: "bg-transparent",
                Icon: "bg-muted text-foreground flex size-10 shrink-0 items-center justify-center rounded-lg [&_svg:not([class*='size-'])]:size-6",
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
