use crate::variants;
use leptos::prelude::*;

variants! {
    /// Badge — shadcn Base UI `badge`. Renders `<span>` (or `<a>` when `href` is
    /// set). Styled by the `cn-badge*` nova layer.
    Badge {
        slot: "badge",
        base: "cn-badge group/badge inline-flex w-fit shrink-0 items-center justify-center overflow-hidden whitespace-nowrap focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none",
        variants: {
            variant: {
                Default: "cn-badge-variant-default",
                Secondary: "cn-badge-variant-secondary",
                Destructive: "cn-badge-variant-destructive",
                Outline: "cn-badge-variant-outline",
                Ghost: "cn-badge-variant-ghost",
                Link: "cn-badge-variant-link",
            }
        },
        component: { element: span }
    }
}
