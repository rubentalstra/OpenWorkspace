use crate::{clx, cn};
use leptos::prelude::*;
use leptos_icons::Icon;

clx! {Breadcrumb, nav, "[&_svg:not([class*='size-'])]:size-4"}
clx! {BreadcrumbList, ol, "text-muted-foreground flex flex-wrap items-center gap-1.5 text-sm break-words sm:gap-2.5"}
clx! {BreadcrumbItem, li, "inline-flex items-center gap-1.5"}
clx! {BreadcrumbLink, a, "hover:text-foreground transition-colors"}

/// Terminal breadcrumb representing the current page; non-interactive and
/// marked `aria-current="page"`.
#[component]
pub fn BreadcrumbPage(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            data-name="BreadcrumbPage"
            role="link"
            aria-disabled="true"
            aria-current="page"
            class=move || cn!("text-foreground font-normal", class.get())
        >
            {children()}
        </span>
    }
}

/// Visual divider between breadcrumb items. Defaults to a chevron; pass children
/// to override the glyph.
#[component]
pub fn BreadcrumbSeparator(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <li
            data-name="BreadcrumbSeparator"
            role="presentation"
            aria-hidden="true"
            class=move || cn!("[&>svg]:size-3.5", class.get())
        >
            {match children {
                Some(children) => children().into_any(),
                None => view! { <Icon icon=icondata::LuChevronRight /> }.into_any(),
            }}
        </li>
    }
}

/// Collapsed-items indicator rendered as an ellipsis glyph.
#[component]
pub fn BreadcrumbEllipsis(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <span
            data-name="BreadcrumbEllipsis"
            role="presentation"
            aria-hidden="true"
            class=move || cn!("flex size-9 items-center justify-center", class.get())
        >
            <Icon icon=icondata::LuEllipsis attr:class="size-4" />
            <span class="sr-only">More</span>
        </span>
    }
}
