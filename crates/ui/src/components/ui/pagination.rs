use crate::{clx, cn};
use leptos::html;
use leptos::prelude::*;
use leptos_icons::Icon;

const LINK_BASE: &str = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 hover:cursor-pointer aria-disabled:pointer-events-none aria-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";
const LINK_ACTIVE: &str = "border bg-background shadow-xs dark:bg-input/30 dark:border-input";
const LINK_INACTIVE: &str = "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50";

clx! {
    /// Ordered list wrapping the pagination items.
    PaginationContent, ul, "flex flex-row items-center gap-1"
}
clx! {
    /// Single slot in the pagination row; holds one link or ellipsis.
    PaginationItem, li, ""
}

/// Pagination navigation landmark. Renders a `<nav>` labelled for assistive
/// technology; compose [`PaginationContent`] / [`PaginationItem`] /
/// [`PaginationLink`] inside it and drive page state from the call site.
#[component]
pub fn Pagination(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <nav
            data-name="Pagination"
            role="navigation"
            aria-label="pagination"
            class=move || cn!("mx-auto flex w-full justify-center", class.get())
        >
            {children()}
        </nav>
    }
}

/// Size of a [`PaginationLink`]: `Default` for labelled links, `Icon` for the
/// square previous/next/page-number buttons.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum PaginationLinkSize {
    Default,
    #[default]
    Icon,
}

impl PaginationLinkSize {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "h-9 px-4 py-2",
            Self::Icon => "size-9",
        }
    }
}

/// Anchor for a single page. Set `is_active` for the current page (applies the
/// active surface styling and `aria-current="page"`). Native attributes, events
/// and bindings — `href`, `on:click`, etc. — forward to the underlying `<a>`.
#[component]
pub fn PaginationLink(
    #[prop(into, optional)] is_active: Signal<bool>,
    #[prop(into, optional)] size: Signal<PaginationLinkSize>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::A>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        let state_class = if is_active.get() {
            LINK_ACTIVE
        } else {
            LINK_INACTIVE
        };
        cn!(LINK_BASE, size.get().class(), state_class, class.get())
    };
    let aria_current = move || is_active.get().then_some("page");

    view! {
        <a
            node_ref=node_ref
            data-name="PaginationLink"
            data-active=move || is_active.get().to_string()
            aria-current=aria_current
            class=merged
        >
            {children()}
        </a>
    }
}

/// Link to the previous page, rendered with a leading chevron and label. Forward
/// `href` / `on:click` from the call site; pass `attr:aria-disabled="true"` when
/// already on the first page.
#[component]
pub fn PaginationPrevious(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::A>,
) -> impl IntoView {
    view! {
        <PaginationLink
            node_ref=node_ref
            size=PaginationLinkSize::Default
            class=Signal::derive(move || cn!("gap-1 px-2.5 sm:pl-2.5", class.get()))
            attr:aria-label="Go to previous page"
        >
            <Icon icon=icondata::LuChevronLeft attr:class="size-4" />
            <span class="hidden sm:block">Previous</span>
        </PaginationLink>
    }
}

/// Link to the next page, rendered with a trailing chevron and label. Forward
/// `href` / `on:click` from the call site; pass `attr:aria-disabled="true"` when
/// already on the last page.
#[component]
pub fn PaginationNext(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::A>,
) -> impl IntoView {
    view! {
        <PaginationLink
            node_ref=node_ref
            size=PaginationLinkSize::Default
            class=Signal::derive(move || cn!("gap-1 px-2.5 sm:pr-2.5", class.get()))
            attr:aria-label="Go to next page"
        >
            <span class="hidden sm:block">Next</span>
            <Icon icon=icondata::LuChevronRight attr:class="size-4" />
        </PaginationLink>
    }
}

/// Non-interactive indicator standing in for one or more collapsed page links.
#[component]
pub fn PaginationEllipsis(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <span
            data-name="PaginationEllipsis"
            role="presentation"
            aria-hidden="true"
            class=move || cn!("flex size-9 items-center justify-center", class.get())
        >
            <Icon icon=icondata::LuEllipsis attr:class="size-4" />
            <span class="sr-only">More pages</span>
        </span>
    }
}
