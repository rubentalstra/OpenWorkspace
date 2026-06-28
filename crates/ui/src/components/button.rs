use crate::variants;
use leptos::prelude::*;

variants! {
    /// Button — shadcn Base UI `button`. Renders `<button>` (or `<a>` when `href`
    /// is set). Styled by the `cn-button*` nova layer; `variant`/`size` select the
    /// semantic look. Extra attributes/events (`attr:type`, `on:click`, …) are
    /// spread onto the root at the call site.
    Button {
        slot: "button",
        base: "cn-button group/button inline-flex shrink-0 items-center justify-center whitespace-nowrap transition-all outline-none select-none disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
        variants: {
            variant: {
                Default: "cn-button-variant-default",
                Outline: "cn-button-variant-outline",
                Secondary: "cn-button-variant-secondary",
                Ghost: "cn-button-variant-ghost",
                Destructive: "cn-button-variant-destructive",
                Link: "cn-button-variant-link",
            },
            size: {
                Default: "cn-button-size-default",
                Xs: "cn-button-size-xs",
                Sm: "cn-button-size-sm",
                Lg: "cn-button-size-lg",
                Icon: "cn-button-size-icon",
                IconXs: "cn-button-size-icon-xs",
                IconSm: "cn-button-size-icon-sm",
                IconLg: "cn-button-size-icon-lg",
            }
        },
        component: { element: button }
    }
}

/// shadcn's `buttonVariants`: the full class string for a button of `variant`/`size`.
/// For first-party components that render a button-styled non-`<button>` element
/// (e.g. `PaginationLink`'s `<a>`) so they reuse the exact button look.
#[must_use]
pub fn button_variants(variant: ButtonVariant, size: ButtonSize) -> String {
    crate::cn!(BUTTON_BASE, variant.class(), size.class())
}
