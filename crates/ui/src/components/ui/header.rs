use crate::{clx, cn, use_data_scrolled, use_lock_body_scroll};
use leptos::prelude::*;
use leptos_icons::Icon;

const SCROLL_THRESHOLD_PX: u32 = 20;

/// Shared open/close state for the mobile navigation drawer, placed in context by
/// [`Header`] so the trigger and the fixed wrapper react to the same signal.
#[derive(Clone, Copy)]
struct HeaderState {
    mobile_open: RwSignal<bool>,
}

clx! {
    /// Centered max-width column that constrains nav content and floats the bar as
    /// the page scrolls past the threshold.
    NavMenuWrapper, nav,
    "px-3 mx-auto w-full max-w-6xl rounded-2xl transition-[padding,background-color,box-shadow,max-width,backdrop-filter] duration-500 ease-in-out",
    "in-data-scrolled:max-w-4xl in-data-scrolled:bg-background in-data-scrolled:backdrop-blur in-data-scrolled:border in-data-scrolled:shadow-lg in-data-scrolled:shadow-black/10 in-data-scrolled:ring-foreground/5 max-md:in-data-scrolled:px-5",
    "max-md:in-data-[state=active]:bg-background/75 max-md:in-data-[state=active]:backdrop-blur max-md:in-data-[state=active]:px-5 max-md:in-data-[state=active]:shadow-black/10 max-md:in-data-[state=active]:ring-foreground/5"
}
clx! {
    /// Nav link styled for the bar and dropdown panels; toggles emphasis from
    /// `data-active`/`data-state` attributes set by the caller.
    NavMenuLink, a,
    "group inline-flex flex-col gap-1 items-center justify-center w-full h-8 py-1 px-4 text-sm font-medium rounded-md outline-none transition-[color,box-shadow]",
    "text-muted-foreground hover:bg-foreground/5 hover:text-foreground focus:bg-foreground/5 focus:text-foreground",
    "disabled:opacity-50 disabled:pointer-events-none focus-visible:ring-ring/50 focus-visible:ring-[3px] focus-visible:outline-1",
    "data-[active=true]:bg-muted/50 data-[active=true]:text-foreground data-[active=true]:hover:bg-accent data-[active=true]:focus:bg-muted",
    "data-[state=open]:bg-foreground/5 data-[state=open]:text-foreground data-[state=open]:hover:bg-foreground/5 data-[state=open]:focus:bg-foreground/5",
    "[&>svg:not([class*='size-'])]:size-4 [&>svg:not([class*='text-'])]:text-muted-foreground"
}
clx! {
    /// Home/brand anchor anchored to the start of the bar.
    NavMenuHomeLink, a, "flex gap-2 h-fit transition-all duration-500 md:in-data-scrolled:px-2"
}
clx! {
    /// Absolutely centered slot used for a logo or title between the edges.
    NavMenuMiddle, div, "absolute inset-0 m-auto size-fit"
}
clx! {
    /// Horizontal list of top-level nav items.
    NavMenuList, menu, "group flex flex-1 gap-0 justify-center items-center list-none"
}
clx! {
    /// Single nav item; hovering or focusing it reveals its [`NavMenuContent`].
    NavMenuItem, li, "relative group/dropdown"
}
clx! {
    /// Section label inside a dropdown panel.
    NavMenuTitle, span, "ml-2 text-xs text-muted-foreground"
}
clx! {
    /// Two-column link tile pairing an icon with a title and description.
    NavMenuLinkGrid, a,
    "grid grid-cols-[auto_1fr] gap-3.5 p-2 text-sm rounded-md outline-none transition-all",
    "hover:bg-accent hover:text-foreground focus:bg-muted focus:text-foreground",
    "focus-visible:ring-ring/50 focus-visible:ring-[3px] focus-visible:outline-1"
}
clx! {
    /// Title line within a [`NavMenuLinkGrid`] tile.
    NavMenuLinkTitle, span, "text-sm font-medium text-foreground"
}
clx! {
    /// Muted, single-line description within a [`NavMenuLinkGrid`] tile.
    NavMenuLinkDescription, p, "text-xs text-muted-foreground line-clamp-1"
}
clx! {
    /// Framed glyph badge sized for nav and dropdown affordances.
    IconWrapper, div,
    "flex relative justify-center items-center rounded border border-transparent ring-1 shadow-sm bg-background ring-foreground/10 size-9 [&_svg:not([class*='size-'])]:size-4"
}
clx! {
    /// Padded, elevated surface that hosts a dropdown's contents.
    NavMenuContentInset, div, "p-0.5 rounded-xl border shadow-lg bg-popover backdrop-blur-md border-border/50"
}
clx! {
    /// Inner card surface nested inside a [`NavMenuContentInset`].
    InsetCard, div, "p-2 rounded-xl border shadow bg-background ring-foreground/5 border-border"
}

/// Page header landmark. Provides the shared mobile-drawer state and tints the
/// popover surface; place the fixed bar and nav inside it.
#[component]
pub fn Header(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    let mobile_open = RwSignal::new(false);
    provide_context(HeaderState { mobile_open });

    let locked = use_lock_body_scroll(false);
    Effect::new(move |_| locked.set(mobile_open.get()));

    view! {
        <header
            data-name="Header"
            class=move || {
                cn!(
                    "[--color-popover:color-mix(in_oklch,var(--color-muted)_25%,var(--color-background))]",
                    class.get(),
                )
            }
        >
            {children()}
        </header>
    }
}

/// Fixed, full-width bar pinned to the top of the viewport. Reads scroll progress
/// to float [`NavMenuWrapper`] and reflects the shared mobile-drawer state through
/// `data-state` for the responsive expanded layout.
#[component]
pub fn NavMenuFixed(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let scrolled = use_data_scrolled(SCROLL_THRESHOLD_PX);
    let state = expect_context::<HeaderState>();

    let data_scrolled = move || if scrolled.get() { "true" } else { "false" };
    let data_state = move || {
        if state.mobile_open.get() {
            "active"
        } else {
            "inactive"
        }
    };

    view! {
        <div
            data-name="NavMenuFixed"
            data-scrolled=data_scrolled
            data-state=data_state
            class=move || {
                cn!(
                    "fixed inset-x-0 top-0 z-50 pt-[calc(0.5rem+env(safe-area-inset-top))] md:pt-[calc(0.75rem+env(safe-area-inset-top))]",
                    "max-md:h-18 max-md:px-2 max-md:overflow-hidden",
                    "max-md:in-data-[state=active]:h-screen max-md:in-data-[state=active]:bg-background/75 max-md:in-data-[state=active]:backdrop-blur",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// Hamburger control that toggles the mobile drawer. Hidden on wide layouts;
/// drives the shared [`HeaderState`] and exposes `aria-expanded`/`aria-controls`
/// against the panel it reveals.
#[component]
pub fn NavMenuMobileTrigger(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let state = expect_context::<HeaderState>();
    let open = state.mobile_open;

    view! {
        <button
            type="button"
            data-name="NavMenuMobileTrigger"
            aria-controls="nav-menu-mobile"
            aria-expanded=move || if open.get() { "true" } else { "false" }
            aria-label="Toggle navigation"
            on:click=move |_| open.update(|value| *value = !*value)
            class=move || {
                cn!(
                    "inline-flex justify-center items-center size-9 rounded-md outline-none transition-colors md:hidden",
                    "text-muted-foreground hover:bg-foreground/5 hover:text-foreground focus-visible:ring-ring/50 focus-visible:ring-[3px]",
                    class.get(),
                )
            }
        >
            <Show
                when=move || open.get()
                fallback=|| view! { <Icon icon=icondata::LuMenu attr:class="size-5" /> }
            >
                <Icon icon=icondata::LuX attr:class="size-5" />
            </Show>
        </button>
    }
}

/// Top-level navigation region holding the [`NavMenuList`]. Hidden on small
/// screens, where the mobile drawer takes over.
#[component]
pub fn NavMenu(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <div
            data-name="NavMenu"
            role="navigation"
            aria-label="Main"
            class=move || {
                cn!(
                    "flex relative flex-1 justify-center items-center max-w-max group/navigation-menu max-md:hidden",
                    class.get(),
                )
            }
        >
            <div class="relative">{children()}</div>
        </div>
    }
}

/// Top-level entry that reveals a [`NavMenuContent`] panel on hover or focus.
/// Rendered as an anchor so keyboard focus opens the panel without JavaScript.
#[component]
pub fn NavMenuTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <a
            data-name="NavMenuTrigger"
            data-state="closed"
            class=move || {
                cn!(
                    "inline-flex justify-center items-center w-max h-8 py-1 px-4 text-sm font-medium rounded-md outline-none transition-[color,box-shadow]",
                    "text-muted-foreground hover:bg-foreground/5 hover:text-foreground focus:bg-foreground/5 focus:text-foreground",
                    "disabled:opacity-50 disabled:pointer-events-none focus-visible:ring-[3px] focus-visible:outline-1",
                    "data-[state=open]:bg-foreground/5 data-[state=open]:text-foreground data-[state=open]:hover:bg-foreground/5 data-[state=open]:focus:bg-foreground/5",
                    class.get(),
                )
            }
        >
            <span>{children()}</span>
            <Icon
                icon=icondata::LuChevronDown
                attr:class="relative ml-1.5 top-[1px] size-3 opacity-75 transition duration-300 group-hover/dropdown:rotate-180 group-hover/dropdown:translate-y-px group-focus-within/dropdown:rotate-180 group-focus-within/dropdown:translate-y-px"
            />
        </a>
    }
}

/// Dropdown panel anchored under its [`NavMenuItem`]. Stays hidden until the item
/// is hovered or focused, transitioning in via CSS only — no JavaScript.
#[component]
pub fn NavMenuContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-name="NavMenuContent"
            class=move || {
                cn!(
                    "absolute left-1/2 top-full z-[100] -translate-x-1/2 origin-center opacity-0 invisible scale-95 pointer-events-none transition-all duration-150 ease-out",
                    "group-hover/dropdown:opacity-100 group-hover/dropdown:visible group-hover/dropdown:scale-100 group-hover/dropdown:pointer-events-auto group-hover/dropdown:delay-200",
                    "group-focus-within/dropdown:opacity-100 group-focus-within/dropdown:visible group-focus-within/dropdown:scale-100 group-focus-within/dropdown:pointer-events-auto group-focus-within/dropdown:delay-200",
                    class.get(),
                )
            }
        >
            <div data-name="NavMenuGap" class="h-2 in-data-scrolled:h-5" />
            {children()}
        </div>
    }
}
