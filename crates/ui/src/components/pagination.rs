use crate::{cn, slot};
use leptos::prelude::*;
use leptos_icons::Icon;

// shadcn renders pagination links as Buttons-as-anchors; we inline the button's
// semantic base so a `PaginationLink` is a standalone themed `<a>`.
const BTN_BASE: &str = "cn-button group/button inline-flex shrink-0 items-center justify-center whitespace-nowrap transition-all outline-none select-none disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0";

/// Pagination — shadcn Base UI `pagination` navigation landmark.
#[component]
pub fn Pagination(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <nav
            role="navigation"
            aria-label="pagination"
            data-slot="pagination"
            class=move || cn!("cn-pagination mx-auto flex w-full justify-center", class.get())
        >
            {children()}
        </nav>
    }
}

slot! { PaginationContent, ul, "pagination-content", "cn-pagination-content flex items-center" }
slot! { PaginationItem, li, "pagination-item", "" }

/// A page link, styled as an icon-sized ghost/outline button (outline when active).
/// Set the destination via `attr:href`. Use `wide` for the prev/next buttons.
#[component]
pub fn PaginationLink(
    #[prop(into, optional)] is_active: Signal<bool>,
    #[prop(default = false)] wide: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let size = if wide {
        "cn-button-size-default"
    } else {
        "cn-button-size-icon"
    };
    let variant = move || {
        if is_active.get() {
            "cn-button-variant-outline"
        } else {
            "cn-button-variant-ghost"
        }
    };
    view! {
        <a
            data-slot="pagination-link"
            data-active=move || is_active.get().to_string()
            aria-current=move || is_active.get().then_some("page")
            class=move || cn!(BTN_BASE, variant(), size, "cn-pagination-link", class.get())
        >
            {children()}
        </a>
    }
}

/// Previous-page link.
#[component]
pub fn PaginationPrevious(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional, default = "Previous".into())] text: String,
) -> impl IntoView {
    view! {
        <PaginationLink
            wide=true
            class=Signal::derive(move || cn!("cn-pagination-previous", class.get()))
        >
            <Icon
                icon=icondata::LuChevronLeft
                attr:data-icon="inline-start"
                attr:class="cn-rtl-flip"
            />
            <span class="cn-pagination-previous-text hidden sm:block">{text}</span>
        </PaginationLink>
    }
}

/// Next-page link.
#[component]
pub fn PaginationNext(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional, default = "Next".into())] text: String,
) -> impl IntoView {
    view! {
        <PaginationLink
            wide=true
            class=Signal::derive(move || cn!("cn-pagination-next", class.get()))
        >
            <span class="cn-pagination-next-text hidden sm:block">{text}</span>
            <Icon
                icon=icondata::LuChevronRight
                attr:data-icon="inline-end"
                attr:class="cn-rtl-flip"
            />
        </PaginationLink>
    }
}

/// Collapsed-pages indicator (…).
#[component]
pub fn PaginationEllipsis(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <span
            aria-hidden="true"
            data-slot="pagination-ellipsis"
            class=move || {
                cn!("cn-pagination-ellipsis flex items-center justify-center", class.get())
            }
        >
            <Icon icon=icondata::LuEllipsis />
            <span class="sr-only">"More pages"</span>
        </span>
    }
}
