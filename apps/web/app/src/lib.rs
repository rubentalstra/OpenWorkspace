use i18n::I18nProvider;
use leptos::prelude::*;
use leptos_fluent::move_tr;
use leptos_meta::{Meta, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{ParentRoute, Route, Router, Routes},
};

mod csrf_client;
pub mod showcase;
pub use csrf_client::CsrfClient;

/// The per-request CSRF token, provided as Leptos context by the server so the
/// `App` can render it into `<head>` as `<meta name="csrf-token">`. Defined here
/// (not in `auth`) so the app crate stays free of the ssr-only auth facade and
/// hydrates cleanly when the context is absent.
#[derive(Clone, Debug)]
pub struct CsrfToken(pub String);

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Dark-mode state: light by default, resolved on the client from local
    // storage (falling back to the OS color-scheme) and mirrored onto the
    // document root's `dark` class. Pure Leptos — no inline theme script.
    _ = ui::ThemeMode::init();

    // On SSR the server provides the per-request CSRF token; emit it into <head>
    // so both the header (JS) and hidden-field (no-JS) paths can read it. Absent
    // during hydration, where it is unneeded.
    let csrf = use_context::<CsrfToken>().map(|t| t.0);

    view! {
        <Stylesheet id="leptos" href="/pkg/openworkspace.css" />

        {csrf.map(|token| view! { <Meta name="csrf-token" content=token /> })}

        // sets the document title
        <Title text="Welcome to Leptos" />

        // content for this welcome page
        <I18nProvider>
            <Router>
                <main>
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=StaticSegment("") view=HomePage />
                        <ParentRoute path=StaticSegment("ui") view=showcase::ShowcaseLayout>
                            <Route path=StaticSegment("") view=showcase::ShowcaseIndex />
                            <Route path=StaticSegment("buttons") view=showcase::ButtonsPage />
                            <Route path=StaticSegment("inputs") view=showcase::InputsPage />
                            <Route path=StaticSegment("forms") view=showcase::FormsPage />
                            <Route path=StaticSegment("overlays") view=showcase::OverlaysPage />
                            <Route path=StaticSegment("navigation") view=showcase::NavigationPage />
                            <Route path=StaticSegment("data") view=showcase::DataPage />
                            <Route path=StaticSegment("dates") view=showcase::DatesPage />
                            <Route path=StaticSegment("feedback") view=showcase::FeedbackPage />
                            <Route path=StaticSegment("layout") view=showcase::LayoutPage />
                            <Route path=StaticSegment("theme") view=showcase::ThemePage />
                            <Route path=StaticSegment("hooks") view=showcase::HooksPage />
                        </ParentRoute>
                    </Routes>
                </main>
            </Router>
        </I18nProvider>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <h1>{move_tr!("home-title")}</h1>
        <button on:click=on_click>{move_tr!("home-count", { "count" => count.get() })}</button>
    }
}
