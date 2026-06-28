use leptos::prelude::*;

use super::{PAGES, Page};

/// Landing page: a short intro and a grid of cards linking to every category.
#[component]
pub fn ShowcaseIndex() -> impl IntoView {
    let cards = PAGES
        .iter()
        .filter(|(href, _)| *href != "/ui")
        .map(|(href, label)| {
            view! {
                <a
                    href=*href
                    class="flex flex-col gap-2 p-5 rounded-lg border transition-colors border-border bg-card hover:border-primary hover:bg-accent"
                >
                    <span class="font-medium">{*label}</span>
                    <span class="text-sm text-muted-foreground">{describe(href)}</span>
                </a>
            }
        })
        .collect_view();

    view! {
        <Page
            title="OpenWorkspace UI"
            subtitle="A living gallery of every design-system component and hook. Pick a category from the sidebar, or jump in below — each page also doubles as a working QA check."
        >
            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">{cards}</div>
        </Page>
    }
}

/// One-line blurb shown on each overview card.
fn describe(href: &str) -> &'static str {
    match href {
        "/ui/buttons" => "Buttons, button groups, press-and-hold, toggles, chips and keys.",
        "/ui/inputs" => "Text fields, selects, checkboxes, switches, radios and sliders.",
        "/ui/forms" => "Validated forms, auto-forms, OTP, phone and prompt inputs.",
        "/ui/overlays" => "Dialogs, sheets, drawers, popovers, menus and the command palette.",
        "/ui/navigation" => "Tabs, breadcrumbs, pagination, menubars and nav bars.",
        "/ui/data" => "Tables, the virtualized data grid, carousels and cards.",
        "/ui/dates" => "Single and range date pickers built on the calendar primitives.",
        "/ui/feedback" => "Alerts, toasts, progress, spinners, badges and empty states.",
        "/ui/layout" => "Accordions, scroll areas, aspect ratios, animation and media.",
        "/ui/theme" => "Light/dark theming, reading direction and responsive helpers.",
        "/ui/hooks" => "The reactive hooks behind the kit, shown in isolation.",
        _ => "",
    }
}
