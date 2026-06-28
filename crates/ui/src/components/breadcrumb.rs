use crate::{cn, slot};
use leptos::prelude::*;
use leptos_icons::Icon;

/// Breadcrumb — shadcn Base UI `breadcrumb` navigation landmark.
#[component]
pub fn Breadcrumb(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <nav
            aria-label="breadcrumb"
            data-slot="breadcrumb"
            class=move || cn!("cn-breadcrumb", class.get())
        >
            {children()}
        </nav>
    }
}

slot! {
    BreadcrumbList, ol, "breadcrumb-list",
    "cn-breadcrumb-list flex flex-wrap items-center wrap-break-word"
}
slot! { BreadcrumbItem, li, "breadcrumb-item", "cn-breadcrumb-item inline-flex items-center" }
slot! {
    /// A breadcrumb hyperlink. Set the destination via `attr:href` at the call site.
    BreadcrumbLink, a, "breadcrumb-link", "cn-breadcrumb-link"
}

/// The current page (non-link, terminal crumb).
#[component]
pub fn BreadcrumbPage(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            data-slot="breadcrumb-page"
            role="link"
            aria-disabled="true"
            aria-current="page"
            class=move || cn!("cn-breadcrumb-page", class.get())
        >
            {children()}
        </span>
    }
}

/// The crumb separator; defaults to a right chevron, override with children.
#[component]
pub fn BreadcrumbSeparator(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <li
            data-slot="breadcrumb-separator"
            role="presentation"
            aria-hidden="true"
            class=move || cn!("cn-breadcrumb-separator", class.get())
        >
            {children
                .map_or_else(
                    || {
                        view! { <Icon icon=icondata::LuChevronRight attr:class="cn-rtl-flip" /> }
                            .into_any()
                    },
                    |children| children(),
                )}
        </li>
    }
}

/// A collapsed-crumbs indicator (…).
#[component]
pub fn BreadcrumbEllipsis(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <span
            data-slot="breadcrumb-ellipsis"
            role="presentation"
            aria-hidden="true"
            class=move || {
                cn!("cn-breadcrumb-ellipsis flex items-center justify-center", class.get())
            }
        >
            <Icon icon=icondata::LuEllipsis />
            <span class="sr-only">"More"</span>
        </span>
    }
}
