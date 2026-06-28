use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{
    Alert, AlertDescription, AlertTitle, Avatar, AvatarBadge, AvatarFallback, AvatarGroup,
    AvatarGroupCount, AvatarImage, AvatarSize, Badge, BadgeSize, BadgeVariant, Button,
    ButtonVariant, Callout, CalloutVariant, Empty, EmptyContent, EmptyDescription, EmptyHeader,
    EmptyMedia, EmptyMediaVariant, EmptyTitle, Progress, Shimmer, Skeleton, SonnerTrigger, Spinner,
    SpinnerCircle, Status, StatusIndicator, StatusIndicatorVariant, Toast, ToastType, use_toaster,
};

use super::{Demo, Page, Section};

/// Alerts, toasts, progress, spinners, badges and empty states.
#[component]
pub fn FeedbackPage() -> impl IntoView {
    view! {
        <Page
            title="Feedback"
            subtitle="Alerts, toasts, progress, spinners, badges and empty states."
        >
            <AlertsSection />
            <CalloutsSection />
            <ToastsSection />
            <ProgressSection />
            <SpinnersSection />
            <PlaceholdersSection />
            <BadgesSection />
            <StatusSection />
            <AvatarsSection />
            <EmptySection />
        </Page>
    }
}

#[component]
fn AlertsSection() -> impl IntoView {
    view! {
        <Section
            title="Alerts"
            description="Inline banner with an optional leading icon, title and description."
        >
            <Demo col=true>
                <Alert>
                    <Icon icon=icondata::LuInfo attr:class="size-4" />
                    <AlertTitle>"Heads up"</AlertTitle>
                    <AlertDescription>
                        "You can add components to your workspace using the kit."
                    </AlertDescription>
                </Alert>
                <Alert class="border-destructive/50 text-destructive [&>svg]:text-destructive"
                    .to_string()>
                    <Icon icon=icondata::LuCircleAlert attr:class="size-4" />
                    <AlertTitle>"Something went wrong"</AlertTitle>
                    <AlertDescription>
                        "Your session could not be verified. Sign in again to continue."
                    </AlertDescription>
                </Alert>
                <Alert>
                    <AlertTitle>"Title only"</AlertTitle>
                    <AlertDescription>"An alert renders fine without an icon."</AlertDescription>
                </Alert>
            </Demo>
        </Section>
    }
}

#[component]
fn CalloutsSection() -> impl IntoView {
    view! {
        <Section
            title="Callouts"
            description="Tonal notice boxes for inline documentation and tips."
        >
            <Demo col=true>
                <Callout title="Note">
                    "Desks are released automatically at the end of each booking window."
                </Callout>
                <Callout variant=CalloutVariant::Info title="Good to know">
                    "Bookings sync across every site you have access to."
                </Callout>
                <Callout variant=CalloutVariant::Warning title="Careful">
                    "Cancelling within an hour of the start counts against your no-show limit."
                </Callout>
                <Callout>"A callout works without a title, too."</Callout>
            </Demo>
        </Section>
    }
}

#[component]
fn ToastsSection() -> impl IntoView {
    let toaster = use_toaster();

    let raise = move |kind: ToastType, title: &'static str, body: &'static str| {
        move |_| {
            _ = toaster.show(Toast::new(kind, title).with_description(body));
        }
    };

    view! {
        <Section
            title="Toasts"
            description="Transient notifications raised through the ambient toaster. One of each semantic type."
        >
            <Demo>
                <Button
                    variant=ButtonVariant::Outline
                    on:click=raise(ToastType::Default, "Note", "A neutral message.")
                >
                    "Default"
                </Button>
                <Button
                    variant=ButtonVariant::Success
                    on:click=raise(ToastType::Success, "Saved", "Your changes were saved.")
                >
                    "Success"
                </Button>
                <Button
                    variant=ButtonVariant::Destructive
                    on:click=raise(ToastType::Error, "Failed", "The booking could not be created.")
                >
                    "Error"
                </Button>
                <Button
                    variant=ButtonVariant::Warning
                    on:click=raise(
                        ToastType::Warning,
                        "Heads up",
                        "This desk is only bookable until 5pm.",
                    )
                >
                    "Warning"
                </Button>
                <Button
                    variant=ButtonVariant::Secondary
                    on:click=raise(ToastType::Info, "FYI", "A new floor plan is available.")
                >
                    "Info"
                </Button>
                <Button
                    variant=ButtonVariant::Secondary
                    on:click=move |_| {
                        let id = toaster
                            .show(
                                Toast::new(ToastType::Loading, "Working")
                                    .with_description("Syncing your bookings…")
                                    .with_duration(None),
                            );
                        set_timeout(
                            move || {
                                toaster.dismiss(&id);
                                _ = toaster
                                    .show(
                                        Toast::new(ToastType::Success, "Synced")
                                            .with_description("All bookings are up to date."),
                                    );
                            },
                            std::time::Duration::from_secs(2),
                        );
                    }
                >
                    "Loading then resolve"
                </Button>
                <Button variant=ButtonVariant::Ghost on:click=move |_| toaster.clear()>
                    "Clear all"
                </Button>
            </Demo>
            <Demo label="SonnerTrigger convenience button">
                <SonnerTrigger
                    title="Copied"
                    description="The desk link is on your clipboard."
                    variant=ToastType::Success
                >
                    "Trigger a toast"
                </SonnerTrigger>
            </Demo>
        </Section>
    }
}

#[component]
fn ProgressSection() -> impl IntoView {
    let value = RwSignal::new(40.0_f64);

    view! {
        <Section
            title="Progress"
            description="A determinate bar driven by a reactive value out of 100."
        >
            <Demo col=true>
                <div class="flex flex-col gap-2">
                    <Progress value=value />
                    <span class="text-sm text-muted-foreground">
                        {move || format!("{:.0}% complete", value.get())}
                    </span>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    <Button
                        variant=ButtonVariant::Outline
                        on:click=move |_| {
                            value.update(|v| *v = (*v - 10.0).max(0.0));
                        }
                    >
                        "-10"
                    </Button>
                    <Button
                        variant=ButtonVariant::Outline
                        on:click=move |_| {
                            value.update(|v| *v = (*v + 10.0).min(100.0));
                        }
                    >
                        "+10"
                    </Button>
                </div>
            </Demo>
            <Demo label="Fixed milestones" col=true>
                <Progress value=0.0 />
                <Progress value=25.0 />
                <Progress value=60.0 />
                <Progress value=100.0 />
            </Demo>
        </Section>
    }
}

#[component]
fn SpinnersSection() -> impl IntoView {
    view! {
        <Section
            title="Spinners"
            description="Indeterminate loaders in two glyphs and a range of sizes."
        >
            <Demo label="Spinner (loader)">
                <Spinner class="size-4".to_string() />
                <Spinner class="size-6".to_string() />
                <Spinner class="size-8 text-primary".to_string() />
            </Demo>
            <Demo label="SpinnerCircle">
                <SpinnerCircle class="size-4".to_string() />
                <SpinnerCircle class="size-6".to_string() />
                <SpinnerCircle class="size-8 text-primary".to_string() />
            </Demo>
            <Demo label="In a button">
                <Button attr:disabled=true>
                    <Spinner class="size-4".to_string() />
                    "Loading…"
                </Button>
            </Demo>
        </Section>
    }
}

#[component]
fn PlaceholdersSection() -> impl IntoView {
    let loading = RwSignal::new(true);

    view! {
        <Section
            title="Skeletons & shimmer"
            description="Loading placeholders: pulsing skeletons and a shimmer-sweep wrapper."
        >
            <Demo label="Skeleton shapes" col=true>
                <div class="flex items-center gap-4">
                    <Skeleton class="size-12 rounded-full".to_string() />
                    <div class="flex flex-col gap-2">
                        <Skeleton class="h-4 w-40".to_string() />
                        <Skeleton class="h-4 w-24".to_string() />
                    </div>
                </div>
                <Skeleton class="h-32 w-full max-w-sm rounded-lg".to_string() />
            </Demo>
            <Demo label="Shimmer toggle" col=true>
                <Shimmer loading=loading class="w-full max-w-sm rounded-lg".to_string()>
                    <div class="flex flex-col gap-2 rounded-lg border border-border p-4">
                        <p class="font-medium">"Desk 4B — Window row"</p>
                        <p class="text-sm text-muted-foreground">
                            "Standing desk, dual monitors, near the kitchen."
                        </p>
                    </div>
                </Shimmer>
                <Button
                    variant=ButtonVariant::Outline
                    on:click=move |_| {
                        loading.update(|l| *l = !*l);
                    }
                >
                    {move || if loading.get() { "Reveal content" } else { "Show shimmer" }}
                </Button>
            </Demo>
        </Section>
    }
}

#[component]
fn BadgesSection() -> impl IntoView {
    view! {
        <Section title="Badges" description="Compact status pills across every variant and size.">
            <Demo label="Variants">
                <Badge>"Default"</Badge>
                <Badge variant=BadgeVariant::Secondary>"Secondary"</Badge>
                <Badge variant=BadgeVariant::Accent>"Accent"</Badge>
                <Badge variant=BadgeVariant::Muted>"Muted"</Badge>
                <Badge variant=BadgeVariant::Destructive>"Destructive"</Badge>
                <Badge variant=BadgeVariant::Outline>"Outline"</Badge>
                <Badge variant=BadgeVariant::Success>"Success"</Badge>
                <Badge variant=BadgeVariant::Warning>"Warning"</Badge>
                <Badge variant=BadgeVariant::Info>"Info"</Badge>
            </Demo>
            <Demo label="Sizes">
                <Badge size=BadgeSize::Sm>"Small"</Badge>
                <Badge size=BadgeSize::Default>"Default"</Badge>
                <Badge size=BadgeSize::Lg>"Large"</Badge>
            </Demo>
            <Demo label="With an icon">
                <Badge variant=BadgeVariant::Success class="gap-1".to_string()>
                    <Icon icon=icondata::LuCircleCheck attr:class="size-3" />
                    "Confirmed"
                </Badge>
                <Badge variant=BadgeVariant::Destructive class="gap-1".to_string()>
                    <Icon icon=icondata::LuCircleX attr:class="size-3" />
                    "Cancelled"
                </Badge>
            </Demo>
        </Section>
    }
}

#[component]
fn StatusSection() -> impl IntoView {
    view! {
        <Section
            title="Status indicators"
            description="A bare coloured dot, and a pinging presence dot anchored to its content."
        >
            <Demo label="Indicator dots">
                <div class="flex items-center gap-2">
                    <StatusIndicator />
                    <span class="text-sm">"Offline"</span>
                </div>
                <div class="flex items-center gap-2">
                    <StatusIndicator variant=StatusIndicatorVariant::Active />
                    <span class="text-sm">"Available"</span>
                </div>
                <div class="flex items-center gap-2">
                    <StatusIndicator variant=StatusIndicatorVariant::Inactive />
                    <span class="text-sm">"Away"</span>
                </div>
                <div class="flex items-center gap-2">
                    <StatusIndicator variant=StatusIndicatorVariant::Normal />
                    <span class="text-sm">"In a meeting"</span>
                </div>
            </Demo>
            <Demo label="Pinging presence">
                <Status variant=StatusIndicatorVariant::Active>
                    <div class="flex size-12 items-center justify-center rounded-lg border border-border bg-muted text-sm font-medium">
                        "4B"
                    </div>
                </Status>
                <Status variant=StatusIndicatorVariant::Inactive>
                    <div class="flex size-12 items-center justify-center rounded-lg border border-border bg-muted text-sm font-medium">
                        "9C"
                    </div>
                </Status>
            </Demo>
        </Section>
    }
}

#[component]
fn AvatarsSection() -> impl IntoView {
    view! {
        <Section
            title="Avatars"
            description="Image avatars with text fallbacks, presence badges, sizes and stacked groups."
        >
            <Demo label="Sizes & fallback">
                <Avatar size=AvatarSize::Sm>
                    <AvatarFallback>"AB"</AvatarFallback>
                </Avatar>
                <Avatar>
                    <AvatarFallback>"CD"</AvatarFallback>
                </Avatar>
                <Avatar size=AvatarSize::Lg>
                    <AvatarFallback>"EF"</AvatarFallback>
                </Avatar>
            </Demo>
            <Demo label="With image (falls back if it fails to load)">
                <Avatar size=AvatarSize::Lg>
                    <AvatarFallback>"GH"</AvatarFallback>
                    <AvatarImage attr:src="https://i.pravatar.cc/80?img=12" attr:alt="Avatar" />
                </Avatar>
            </Demo>
            <Demo label="With a presence badge">
                <Avatar size=AvatarSize::Lg>
                    <AvatarFallback>"IJ"</AvatarFallback>
                    <AvatarBadge class="bg-success".to_string() />
                </Avatar>
                <Avatar size=AvatarSize::Lg>
                    <AvatarFallback>"KL"</AvatarFallback>
                    <AvatarBadge>
                        <Icon icon=icondata::LuCheck />
                    </AvatarBadge>
                </Avatar>
            </Demo>
            <Demo label="Group with overflow count">
                <AvatarGroup>
                    <Avatar>
                        <AvatarFallback>"MN"</AvatarFallback>
                    </Avatar>
                    <Avatar>
                        <AvatarFallback>"OP"</AvatarFallback>
                    </Avatar>
                    <Avatar>
                        <AvatarFallback>"QR"</AvatarFallback>
                    </Avatar>
                    <AvatarGroupCount>"+5"</AvatarGroupCount>
                </AvatarGroup>
            </Demo>
        </Section>
    }
}

#[component]
fn EmptySection() -> impl IntoView {
    view! {
        <Section
            title="Empty states"
            description="A composed placeholder for when a list or surface has nothing to show."
        >
            <Demo>
                <Empty class="w-full max-w-md".to_string()>
                    <EmptyHeader>
                        <EmptyMedia variant=EmptyMediaVariant::Icon>
                            <Icon icon=icondata::LuSearch attr:class="size-6" />
                        </EmptyMedia>
                        <EmptyTitle>"No desks found"</EmptyTitle>
                        <EmptyDescription>
                            "No desks match your filters. Try widening your search or pick another floor."
                        </EmptyDescription>
                    </EmptyHeader>
                    <EmptyContent>
                        <Button variant=ButtonVariant::Outline>"Clear filters"</Button>
                        <Button>
                            <Icon icon=icondata::LuPlus attr:class="size-4" />
                            "Book elsewhere"
                        </Button>
                    </EmptyContent>
                </Empty>
            </Demo>
        </Section>
    }
}
