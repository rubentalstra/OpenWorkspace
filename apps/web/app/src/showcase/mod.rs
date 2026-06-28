//! Internal developer gallery for the `ui` kit, organised into category pages so
//! every ported component can be eyeballed and a11y-checked. Each demo is framed
//! in a `Card`. Grows as each wave lands; the sidebar-07 shell returns with the
//! Sidebar family (Wave 4–5).

use leptos::prelude::*;
use leptos_router::components::Outlet;
use ui::{
    Alert, AlertDescription, AlertTitle, AlertVariant, AspectRatio, Avatar, AvatarFallback,
    AvatarGroup, Badge, BadgeVariant, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList,
    BreadcrumbPage, BreadcrumbSeparator, Button, ButtonSize, ButtonVariant, Card, CardContent,
    CardDescription, CardFooter, CardHeader, CardTitle, Checkbox, Empty, EmptyDescription,
    EmptyHeader, EmptyMedia, EmptyTitle, Input, Item, ItemContent, ItemDescription, ItemGroup,
    ItemMedia, ItemTitle, Kbd, KbdGroup, Label, NativeSelect, NativeSelectOption, Pagination,
    PaginationContent, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious,
    Progress, Separator, SeparatorOrientation, Skeleton, Slider, Spinner, Switch, Table, TableBody,
    TableCell, TableHead, TableHeader, TableRow, Textarea, Toggle, use_theme_mode,
};

/// Ordered nav: `(href, label)`. Drives the header tabs and the overview grid.
pub(crate) const PAGES: &[(&str, &str)] = &[
    ("/ui", "Overview"),
    ("/ui/buttons", "Buttons"),
    ("/ui/inputs", "Inputs"),
    ("/ui/data", "Data"),
    ("/ui/feedback", "Feedback"),
    ("/ui/layout", "Layout"),
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
                                        href=href.to_string()
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

/// Page heading + a responsive grid for the demo cards.
#[component]
fn PageShell(
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
fn Demo(#[prop(into)] title: String, children: Children) -> impl IntoView {
    view! {
        <Card>
            <CardHeader>
                <CardTitle class="text-base">{title}</CardTitle>
            </CardHeader>
            <CardContent class="flex flex-wrap items-center gap-3">{children()}</CardContent>
        </Card>
    }
}

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
                                <Button href=href.to_string() size=ButtonSize::Sm>
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

/// Buttons, badges, toggles, kbd.
#[component]
pub fn ButtonsPage() -> impl IntoView {
    let bold = RwSignal::new(false);
    view! {
        <PageShell title="Buttons" subtitle="Actions, badges, toggles.">
            <Demo title="Variants">
                <Button>"Default"</Button>
                <Button variant=ButtonVariant::Secondary>"Secondary"</Button>
                <Button variant=ButtonVariant::Outline>"Outline"</Button>
                <Button variant=ButtonVariant::Ghost>"Ghost"</Button>
                <Button variant=ButtonVariant::Destructive>"Destructive"</Button>
                <Button variant=ButtonVariant::Link>"Link"</Button>
            </Demo>
            <Demo title="Sizes">
                <Button size=ButtonSize::Xs>"Xs"</Button>
                <Button size=ButtonSize::Sm>"Sm"</Button>
                <Button size=ButtonSize::Default>"Default"</Button>
                <Button size=ButtonSize::Lg>"Lg"</Button>
            </Demo>
            <Demo title="Badges">
                <Badge>"Default"</Badge>
                <Badge variant=BadgeVariant::Secondary>"Secondary"</Badge>
                <Badge variant=BadgeVariant::Outline>"Outline"</Badge>
                <Badge variant=BadgeVariant::Destructive>"Destructive"</Badge>
            </Demo>
            <Demo title="Toggle + Kbd">
                <Toggle pressed=bold on_change=Callback::new(move |v| bold.set(v))>
                    "Bold"
                </Toggle>
                <KbdGroup>
                    <Kbd>"⌘"</Kbd>
                    <Kbd>"K"</Kbd>
                </KbdGroup>
            </Demo>
        </PageShell>
    }
}

/// Form inputs.
#[component]
pub fn InputsPage() -> impl IntoView {
    let checked = RwSignal::new(true);
    let switch_on = RwSignal::new(true);
    let vol = RwSignal::new(40.0);
    view! {
        <PageShell title="Inputs" subtitle="Text, choice, and range controls.">
            <Demo title="Text field">
                <div class="flex w-full flex-col gap-2">
                    <Label attr:r#for="email">"Email"</Label>
                    <Input attr:id="email" attr:r#type="email" attr:placeholder="you@example.com" />
                    <Textarea attr:placeholder="Notes…" />
                </div>
            </Demo>
            <Demo title="Checkbox / Switch">
                <Checkbox checked=checked on_change=Callback::new(move |v| checked.set(v)) />
                <Switch checked=switch_on on_change=Callback::new(move |v| switch_on.set(v)) />
            </Demo>
            <Demo title="Slider">
                <Slider value=vol on_change=Callback::new(move |v| vol.set(v)) class="w-full" />
                <span class="text-muted-foreground text-sm tabular-nums">
                    {move || format!("{:.0}", vol.get())}
                </span>
            </Demo>
            <Demo title="Native select">
                <NativeSelect>
                    <NativeSelectOption>"Map"</NativeSelectOption>
                    <NativeSelectOption>"List"</NativeSelectOption>
                    <NativeSelectOption>"Calendar"</NativeSelectOption>
                </NativeSelect>
            </Demo>
        </PageShell>
    }
}

/// Tables, lists, avatars, breadcrumbs, pagination.
#[component]
pub fn DataPage() -> impl IntoView {
    view! {
        <PageShell title="Data" subtitle="Tables, lists, identities, navigation.">
            <Demo title="Table">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>"Desk"</TableHead>
                            <TableHead>"Floor"</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        <TableRow>
                            <TableCell>"A-12"</TableCell>
                            <TableCell>"2"</TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell>"B-03"</TableCell>
                            <TableCell>"3"</TableCell>
                        </TableRow>
                    </TableBody>
                </Table>
            </Demo>
            <Demo title="Avatars + Item">
                <AvatarGroup>
                    <Avatar>
                        <AvatarFallback>"OW"</AvatarFallback>
                    </Avatar>
                    <Avatar>
                        <AvatarFallback>"RT"</AvatarFallback>
                    </Avatar>
                </AvatarGroup>
                <ItemGroup class="w-full">
                    <Item>
                        <ItemMedia>
                            <Avatar>
                                <AvatarFallback>"OW"</AvatarFallback>
                            </Avatar>
                        </ItemMedia>
                        <ItemContent>
                            <ItemTitle>"Window desk"</ItemTitle>
                            <ItemDescription>"Floor 2 · near the kitchen"</ItemDescription>
                        </ItemContent>
                    </Item>
                </ItemGroup>
            </Demo>
            <Demo title="Breadcrumb">
                <Breadcrumb>
                    <BreadcrumbList>
                        <BreadcrumbItem>
                            <BreadcrumbLink attr:href="#">"Campus"</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbPage>"Floor 2"</BreadcrumbPage>
                        </BreadcrumbItem>
                    </BreadcrumbList>
                </Breadcrumb>
            </Demo>
            <Demo title="Pagination">
                <Pagination>
                    <PaginationContent>
                        <PaginationItem>
                            <PaginationPrevious attr:href="#" />
                        </PaginationItem>
                        <PaginationItem>
                            <PaginationLink attr:href="#">"1"</PaginationLink>
                        </PaginationItem>
                        <PaginationItem>
                            <PaginationLink is_active=true attr:href="#">
                                "2"
                            </PaginationLink>
                        </PaginationItem>
                        <PaginationItem>
                            <PaginationNext attr:href="#" />
                        </PaginationItem>
                    </PaginationContent>
                </Pagination>
            </Demo>
        </PageShell>
    }
}

/// Alerts, progress, spinners, skeletons, empty states.
#[component]
pub fn FeedbackPage() -> impl IntoView {
    view! {
        <PageShell title="Feedback" subtitle="Status, progress, and empty states.">
            <Demo title="Alert">
                <div class="flex w-full flex-col gap-3">
                    <Alert>
                        <AlertTitle>"Heads up"</AlertTitle>
                        <AlertDescription>"Your booking is confirmed."</AlertDescription>
                    </Alert>
                    <Alert variant=AlertVariant::Destructive>
                        <AlertTitle>"Conflict"</AlertTitle>
                        <AlertDescription>"That desk is already booked."</AlertDescription>
                    </Alert>
                </div>
            </Demo>
            <Demo title="Progress / Spinner">
                <Progress value=66.0 class="w-full" />
                <Spinner />
            </Demo>
            <Demo title="Skeleton">
                <div class="flex w-full flex-col gap-2">
                    <Skeleton class="h-8 w-full rounded-md" />
                    <Skeleton class="h-4 w-3/4 rounded-md" />
                </div>
            </Demo>
            <Demo title="Empty">
                <Empty class="py-6">
                    <EmptyHeader>
                        <EmptyMedia>"📭"</EmptyMedia>
                        <EmptyTitle>"No bookings"</EmptyTitle>
                        <EmptyDescription>"You have nothing booked today."</EmptyDescription>
                    </EmptyHeader>
                </Empty>
            </Demo>
        </PageShell>
    }
}

/// Structural primitives.
#[component]
pub fn LayoutPage() -> impl IntoView {
    view! {
        <PageShell title="Layout" subtitle="Separators and ratios.">
            <Demo title="Separator">
                <div class="flex h-5 items-center gap-3 text-sm">
                    "Map" <Separator orientation=SeparatorOrientation::Vertical /> "List"
                    <Separator orientation=SeparatorOrientation::Vertical /> "Calendar"
                </div>
            </Demo>
            <Demo title="Aspect ratio">
                <AspectRatio
                    ratio=1.7777
                    class="bg-muted text-muted-foreground flex w-full items-center justify-center rounded-md text-sm"
                >
                    "16 / 9"
                </AspectRatio>
            </Demo>
        </PageShell>
    }
}
