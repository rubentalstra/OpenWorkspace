//! Internal developer gallery for the `ui` kit and the floor renderer. The shell
//! (header nav + theme toggle) lives here with the shared page chrome
//! (`PageShell`, `Demo`); each routed page is its own module — the shape the real
//! app uses later (a shell + navbar wrapping separate pages with their content).

pub mod buttons;
pub mod chat;
pub mod data;
pub mod feedback;
pub mod floor;
pub mod inputs;
pub mod layout;
pub mod overlays;
pub mod overview;

pub use buttons::ButtonsPage;
pub use chat::ChatPage;
pub use data::DataPage;
pub use feedback::FeedbackPage;
pub use floor::FloorPage;
pub use inputs::InputsPage;
pub use layout::LayoutPage;
pub use overlays::OverlaysPage;
pub use overview::ShowcaseIndex;

use leptos::prelude::*;
use leptos_router::components::Outlet;
use ui::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, CardHeader, CardTitle, Separator,
    SeparatorOrientation, use_theme_mode,
};

/// Ordered nav: `(href, label)`. Drives the header tabs and the overview grid.
pub(crate) const PAGES: &[(&str, &str)] = &[
    ("/ui", "Overview"),
    ("/ui/buttons", "Buttons"),
    ("/ui/inputs", "Inputs"),
    ("/ui/data", "Data"),
    ("/ui/feedback", "Feedback"),
    ("/ui/overlays", "Overlays"),
    ("/ui/layout", "Layout"),
    ("/ui/chat", "Chat"),
    ("/ui/floor", "Floor"),
];

/// Sticky header (nav + theme toggle) wrapping the routed page.
#[component]
pub fn ShowcaseLayout() -> impl IntoView {
    view! {
        <div data-slot="layout" class="flex min-h-svh flex-col">
            <header class="bg-background/80 sticky top-0 z-20 border-b backdrop-blur">
                <div class="container flex h-14 items-center justify-between gap-4">
                    <a href="/ui" class="cn-font-heading font-semibold">
                        "OpenWorkspace UI"
                    </a>
                    <nav class="flex items-center gap-1">
                        {PAGES
                            .iter()
                            .map(|&(href, label)| {
                                view! {
                                    <Button
                                        href=href.to_owned()
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Sm
                                    >
                                        {label}
                                    </Button>
                                }
                            })
                            .collect_view()}
                        <Separator orientation=SeparatorOrientation::Vertical class="mx-1 h-6" />
                        <ThemeToggle />
                    </nav>
                </div>
            </header>
            <main class="container flex-1 py-8">
                <Outlet />
            </main>
        </div>
    }
}

#[component]
fn ThemeToggle() -> impl IntoView {
    let theme = use_theme_mode();
    view! {
        <Button
            variant=ButtonVariant::Outline
            size=ButtonSize::Icon
            on:click=move |_| theme.toggle()
            attr:aria-label="Toggle dark mode"
        >
            {move || if theme.is_dark() { "☀" } else { "☾" }}
        </Button>
    }
}

/// Page heading + a responsive grid for the demo cards. Shared by every gallery
/// page (the floor page lays out its own content).
#[component]
pub(crate) fn PageShell(
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6">
            <div class="flex flex-col gap-1">
                <h1 class="cn-font-heading text-2xl font-semibold tracking-tight">{title}</h1>
                <p class="text-muted-foreground text-sm">{subtitle}</p>
            </div>
            <div class="grid items-start gap-4 lg:grid-cols-2">{children()}</div>
        </div>
    }
}

/// One demo, framed in a card.
#[component]
pub(crate) fn Demo(#[prop(into)] title: String, children: Children) -> impl IntoView {
    view! {
        <Card>
            <CardHeader>
                <CardTitle class="text-base">{title}</CardTitle>
            </CardHeader>
            <CardContent class="flex flex-wrap items-center gap-3">{children()}</CardContent>
        </Card>
    }
}
