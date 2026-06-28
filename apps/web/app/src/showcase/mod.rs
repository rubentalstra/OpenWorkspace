//! Multi-page gallery that exercises every `ui` component and hook in a real
//! SSR app. Internal developer tool: copy is hardcoded English, pages are
//! grouped by category, and every page is reached through the [`ShowcaseLayout`]
//! sidenav. Each page also serves as a living QA check — a component that cannot
//! be demonstrated here is a component that needs fixing.

mod buttons;
mod data;
mod dates;
mod feedback;
mod forms;
mod hooks;
mod index;
mod inputs;
mod layout;
mod navigation;
mod overlays;
mod shell;
mod theme;

// Glob re-exports so each page's component *and* its generated `…Props` struct
// surface together (a bare `pub use foo::FooPage` would leave `FooPageProps` an
// unreachable `pub` inside its private module).
pub use buttons::*;
pub use data::*;
pub use dates::*;
pub use feedback::*;
pub use forms::*;
pub use hooks::*;
pub use index::*;
pub use inputs::*;
pub use layout::*;
pub use navigation::*;
pub use overlays::*;
pub use shell::*;
pub use theme::*;

use leptos::prelude::*;

/// Ordered list of gallery pages: `(href, label)`. Drives the sidenav and the
/// overview grid so a new page is wired in one place.
pub(crate) const PAGES: &[(&str, &str)] = &[
    ("/ui", "Overview"),
    ("/ui/buttons", "Buttons & actions"),
    ("/ui/inputs", "Inputs"),
    ("/ui/forms", "Forms"),
    ("/ui/overlays", "Overlays"),
    ("/ui/navigation", "Navigation"),
    ("/ui/data", "Data"),
    ("/ui/dates", "Dates"),
    ("/ui/feedback", "Feedback"),
    ("/ui/layout", "Layout"),
    ("/ui/theme", "Theme"),
    ("/ui/hooks", "Hooks"),
];

/// Standard page frame: a heading, a one-line subtitle, and a vertical stack of
/// [`Section`]s passed as children.
#[component]
pub fn Page(
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-12">
            <header class="flex flex-col gap-2">
                <h1 class="text-3xl font-bold tracking-tight">{title}</h1>
                <p class="max-w-2xl text-muted-foreground">{subtitle}</p>
            </header>
            {children()}
        </div>
    }
}

/// A titled group of related demos within a [`Page`].
#[component]
pub fn Section(
    #[prop(into)] title: String,
    #[prop(into, optional)] description: String,
    children: Children,
) -> impl IntoView {
    let description = (!description.is_empty())
        .then(|| view! { <p class="text-sm text-muted-foreground">{description}</p> });

    view! {
        <section class="flex flex-col gap-5 scroll-mt-20">
            <div class="flex flex-col gap-1">
                <h2 class="text-xl font-semibold tracking-tight">{title}</h2>
                {description}
            </div>
            {children()}
        </section>
    }
}

/// A bordered preview surface holding one or more rendered examples. An optional
/// `label` captions the frame; `col` stacks examples vertically instead of the
/// default wrapped row.
#[component]
pub fn Demo(
    #[prop(into, optional)] label: String,
    #[prop(optional)] col: bool,
    children: Children,
) -> impl IntoView {
    let caption = (!label.is_empty()).then(|| {
        view! {
            <span class="text-xs font-medium tracking-wide uppercase text-muted-foreground">
                {label}
            </span>
        }
    });
    let layout = if col {
        "flex flex-col gap-4"
    } else {
        "flex flex-wrap gap-4 items-center"
    };

    view! {
        <div class="flex flex-col gap-3">
            {caption}
            <div class=format!(
                "{layout} rounded-lg border border-border bg-card p-6 text-card-foreground",
            )>{children()}</div>
        </div>
    }
}
