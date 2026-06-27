use crate::cn;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

/// Strategy for deciding when a [`Link`] is "active" relative to the current
/// route. The variant is compared against `href` and the live pathname.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub enum LinkMatch {
    /// Active when the current path starts with `href` (section + descendants).
    #[default]
    Prefix,
    /// Active only when the current path equals `href`.
    Exact,
    /// Active when the current path contains `href` as a substring.
    Contains,
    /// Active when the current path starts with `href` but is not one of the
    /// excluded paths.
    PrefixExcept(Vec<String>),
    /// Active when the current path equals any of these paths.
    Any(Vec<String>),
}

impl LinkMatch {
    fn matches(&self, href: &str, current: &str) -> bool {
        match self {
            Self::Prefix => current.starts_with(href),
            Self::Exact => current == href,
            Self::Contains => current.contains(href),
            Self::PrefixExcept(excludes) => {
                current.starts_with(href) && !excludes.iter().any(|p| p == current)
            }
            Self::Any(paths) => paths.iter().any(|p| p == current),
        }
    }
}

const LINK_BASE: &str = "text-primary underline-offset-4 hover:underline outline-none focus-visible:ring-ring/50 focus-visible:ring-[3px] rounded-sm";
const LINK_ACTIVE: &str = "font-semibold";

/// Styled, route-aware navigation link. Performs client-side navigation via the
/// router and applies `LINK_ACTIVE` styling when the current route matches `href`
/// per `match_type`; the router marks the active anchor with `aria-current="page"`
/// for accessibility. Native attributes, events and bindings forward to the
/// underlying anchor — set `attr:target`, `on:click`, etc. at the call site.
#[component]
pub fn Link(
    #[prop(into)] href: String,
    #[prop(into, optional)] match_type: LinkMatch,
    /// Whether navigation scrolls to the top of the page (router default).
    #[prop(default = true)]
    scroll: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let location = use_location();
    let merged = {
        let href = href.clone();
        move || {
            let active = match_type.matches(&href, &location.pathname.get());
            cn!(LINK_BASE, active.then_some(LINK_ACTIVE), class.get())
        }
    };

    view! {
        <A href=href scroll=scroll attr:data-name="Link" attr:class=merged>
            {children()}
        </A>
    }
}
