//! Overview page: a card per category, linking into the gallery.

use leptos::prelude::*;
use ui::{Button, ButtonSize, Card, CardDescription, CardFooter, CardHeader, CardTitle};

use super::{PAGES, PageShell};

/// Overview: a card per category.
#[component]
pub fn ShowcaseIndex() -> impl IntoView {
    view! {
        <PageShell
            title="OpenWorkspace UI"
            subtitle="A 1:1 Leptos port of shadcn/ui — the Base UI flavour, nova style."
        >
            {PAGES
                .iter()
                .skip(1)
                .map(|&(href, label)| {
                    view! {
                        <Card>
                            <CardHeader>
                                <CardTitle>{label}</CardTitle>
                                <CardDescription>"Component demos."</CardDescription>
                            </CardHeader>
                            <CardFooter>
                                <Button href=href.to_owned() size=ButtonSize::Sm>
                                    "Open"
                                </Button>
                            </CardFooter>
                        </Card>
                    }
                })
                .collect_view()}
        </PageShell>
    }
}
