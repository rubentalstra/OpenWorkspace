//! Internal developer gallery for the `ui` kit. Rebuilt small during the shadcn
//! Base-UI port: it exercises the primitives ported so far so the nova styling can
//! be eyeballed and a11y-checked. Grows back as each wave lands; the sidebar-07
//! shell returns with the Sidebar family (Wave 4–5).

use leptos::prelude::*;
use leptos_router::components::Outlet;
use ui::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Card, CardContent, CardDescription,
    CardFooter, CardHeader, CardTitle, Input, Label, Separator, SeparatorOrientation, Skeleton,
    use_theme_mode,
};

/// Top-level frame: a sticky header with nav + theme toggle, and the routed page.
#[component]
pub fn ShowcaseLayout() -> impl IntoView {
    view! {
        <div data-slot="layout" class="flex min-h-svh flex-col">
            <header class="bg-background/80 sticky top-0 z-20 border-b backdrop-blur">
                <div class="container flex h-14 items-center justify-between gap-4">
                    <a href="/ui" class="cn-font-heading font-semibold">
                        "OpenWorkspace UI"
                    </a>
                    <nav class="flex items-center gap-2">
                        <Button href="/ui" variant=ButtonVariant::Ghost size=ButtonSize::Sm>
                            "Overview"
                        </Button>
                        <Button
                            href="/ui/components"
                            variant=ButtonVariant::Ghost
                            size=ButtonSize::Sm
                        >
                            "Components"
                        </Button>
                        <Separator orientation=SeparatorOrientation::Vertical class="h-6" />
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

/// Overview page.
#[component]
pub fn ShowcaseIndex() -> impl IntoView {
    view! {
        <div class="flex max-w-2xl flex-col gap-6">
            <div class="flex flex-col gap-2">
                <h1 class="cn-font-heading text-3xl font-semibold tracking-tight">
                    "OpenWorkspace UI"
                </h1>
                <p class="text-muted-foreground">
                    "A 1:1 Leptos port of shadcn/ui — the Base UI flavour, nova style."
                </p>
            </div>
            <Card>
                <CardHeader>
                    <CardTitle>"Components"</CardTitle>
                    <CardDescription>"The primitives ported so far."</CardDescription>
                </CardHeader>
                <CardContent>
                    <p class="text-muted-foreground text-sm">
                        "Buttons, badges, cards, inputs, labels, separators, skeletons."
                    </p>
                </CardContent>
                <CardFooter>
                    <Button href="/ui/components">"Browse components"</Button>
                </CardFooter>
            </Card>
        </div>
    }
}

/// A titled block within the components page.
#[component]
fn Section(#[prop(into)] title: String, children: Children) -> impl IntoView {
    view! {
        <section class="flex flex-col gap-4">
            <h2 class="cn-font-heading text-lg font-semibold">{title}</h2>
            <div class="flex flex-wrap items-center gap-3">{children()}</div>
        </section>
    }
}

/// Gallery of the ported primitives.
#[component]
pub fn ComponentsPage() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-10">
            <Section title="Button variants">
                <Button>"Default"</Button>
                <Button variant=ButtonVariant::Secondary>"Secondary"</Button>
                <Button variant=ButtonVariant::Outline>"Outline"</Button>
                <Button variant=ButtonVariant::Ghost>"Ghost"</Button>
                <Button variant=ButtonVariant::Destructive>"Destructive"</Button>
                <Button variant=ButtonVariant::Link>"Link"</Button>
            </Section>

            <Section title="Button sizes">
                <Button size=ButtonSize::Xs>"Xs"</Button>
                <Button size=ButtonSize::Sm>"Sm"</Button>
                <Button size=ButtonSize::Default>"Default"</Button>
                <Button size=ButtonSize::Lg>"Lg"</Button>
            </Section>

            <Section title="Badges">
                <Badge>"Default"</Badge>
                <Badge variant=BadgeVariant::Secondary>"Secondary"</Badge>
                <Badge variant=BadgeVariant::Outline>"Outline"</Badge>
                <Badge variant=BadgeVariant::Destructive>"Destructive"</Badge>
            </Section>

            <Section title="Field">
                <div class="flex w-full max-w-sm flex-col gap-2">
                    <Label attr:r#for="email">"Email"</Label>
                    <Input attr:id="email" attr:r#type="email" attr:placeholder="you@example.com" />
                </div>
            </Section>

            <Section title="Card">
                <Card class="w-full max-w-sm">
                    <CardHeader>
                        <CardTitle>"Desk booking"</CardTitle>
                        <CardDescription>"Reserve a workspace for the day."</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <p class="text-muted-foreground text-sm">"Pick a floor, then a desk."</p>
                    </CardContent>
                    <CardFooter class="gap-2">
                        <Button size=ButtonSize::Sm>"Book"</Button>
                        <Button size=ButtonSize::Sm variant=ButtonVariant::Outline>
                            "Cancel"
                        </Button>
                    </CardFooter>
                </Card>
            </Section>

            <Section title="Separator">
                <div class="flex h-5 items-center gap-3 text-sm">
                    "Map" <Separator orientation=SeparatorOrientation::Vertical /> "List"
                    <Separator orientation=SeparatorOrientation::Vertical /> "Calendar"
                </div>
            </Section>

            <Section title="Skeleton">
                <div class="flex w-full max-w-sm flex-col gap-2">
                    <Skeleton class="h-8 w-full rounded-md" />
                    <Skeleton class="h-4 w-3/4 rounded-md" />
                </div>
            </Section>
        </div>
    }
}
