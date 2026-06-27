use crate::clx;
use leptos::prelude::*;

clx! {
    /// Bottom navigation bar. Anchors a row of [`BottomNavButton`]s above the
    /// home-indicator safe area on touch devices.
    BottomNav, nav,
    "z-50 mx-auto w-full max-w-lg border-t border-border bg-background pb-[env(safe-area-inset-bottom,0px)]"
}

clx! {
    /// Equal-width column track that lays the navigation buttons out side by side.
    BottomNavGrid, div,
    "grid grid-flow-col auto-cols-fr h-[var(--bottom__nav__height)] font-medium"
}

clx! {
    /// Caption rendered under a [`BottomNavButton`] icon; tracks the active state
    /// of its hovered or `aria-current` parent.
    BottomNavLabel, span,
    "text-sm text-muted-foreground group-hover:text-primary group-aria-[current=page]:text-primary"
}

clx! {
    /// Single navigation entry. Stacks an icon over a [`BottomNavLabel`]; set
    /// `aria-current="page"` at the call site to mark the active destination.
    BottomNavButton, button,
    "inline-flex flex-col justify-center items-center px-5 group [&_svg]:mb-2 [&_svg]:text-muted-foreground hover:[&_svg]:text-primary aria-[current=page]:[&_svg]:text-primary active:scale-[0.98]",
    "touch-manipulation [-webkit-tap-highlight-color:transparent] select-none [-webkit-touch-callout:none]",
    "supports-[-webkit-touch-callout:none]:justify-end supports-[-webkit-touch-callout:none]:pb-0 supports-[-webkit-touch-callout:none]:translate-y-1"
}
