use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{
    Button, ButtonVariant, Direction, DirectionProvider, MOBILE_BREAKPOINT, ThemeToggle,
    use_direction, use_is_mobile, use_media_query, use_theme_mode,
};

use super::{Demo, Page, Section};

/// Light/dark theming, reading direction and responsive helpers.
#[component]
pub fn ThemePage() -> impl IntoView {
    view! {
        <Page
            title="Theme"
            subtitle="Light/dark theming, reading direction and responsive helpers."
        >
            <ThemeSection />
            <DirectionSection />
            <MediaQuerySection />
            <MobileSection />
            <SsrNoteSection />
        </Page>
    }
}

/// Renders a `true`/`false` value as a coloured pill so live boolean readouts
/// stand out against the demo surface.
#[component]
fn BoolBadge(#[prop(into)] value: Signal<bool>) -> impl IntoView {
    let class = move || {
        let tone = if value.get() {
            "bg-success/15 text-success"
        } else {
            "bg-muted text-muted-foreground"
        };
        format!(
            "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium {tone}"
        )
    };

    view! {
        <span class=class>
            {move || {
                if value.get() {
                    view! { <Icon icon=icondata::LuCheck attr:class="size-3" /> }.into_any()
                } else {
                    view! { <Icon icon=icondata::LuX attr:class="size-3" /> }.into_any()
                }
            }} {move || if value.get() { "true" } else { "false" }}
        </span>
    }
}

#[component]
fn ThemeSection() -> impl IntoView {
    let theme = use_theme_mode();

    view! {
        <Section
            title="Theme mode"
            description="The shared ThemeMode is initialised at the app root. Read it with use_theme_mode and flip it from anywhere."
        >
            <Demo label="Toggle control">
                <ThemeToggle />
                <span class="text-sm text-muted-foreground">
                    "Click the icon to cross-fade between sun and moon."
                </span>
            </Demo>

            <Demo label="Live readout">
                <span class="text-sm font-medium">"is_dark"</span>
                <BoolBadge value=Signal::derive(move || theme.is_dark()) />
                <span class="text-sm font-medium">"is_light"</span>
                <BoolBadge value=Signal::derive(move || theme.is_light()) />
            </Demo>

            <Demo label="Imperative setters">
                <Button variant=ButtonVariant::Outline on:click=move |_| theme.set_light()>
                    <Icon icon=icondata::LuSun attr:class="size-4" />
                    "Light"
                </Button>
                <Button variant=ButtonVariant::Outline on:click=move |_| theme.set_dark()>
                    <Icon icon=icondata::LuMoon attr:class="size-4" />
                    "Dark"
                </Button>
                <Button variant=ButtonVariant::Secondary on:click=move |_| theme.toggle()>
                    "Toggle"
                </Button>
                <span class="text-sm text-muted-foreground">
                    {move || {
                        if theme.get() { "Currently dark mode" } else { "Currently light mode" }
                    }}
                </span>
            </Demo>
        </Section>
    }
}

#[component]
fn DirectionSection() -> impl IntoView {
    let dir = RwSignal::new(Direction::Ltr);
    let is_rtl = Signal::derive(move || dir.get() == Direction::Rtl);

    view! {
        <Section
            title="Reading direction"
            description="DirectionProvider sets the dir attribute on its subtree and shares the value through context; use_direction reads it back."
        >
            <Demo col=true label="Flip the contained sample">
                <div class="flex flex-wrap items-center gap-3">
                    <Button
                        variant=ButtonVariant::Outline
                        on:click=move |_| {
                            dir.update(|d| {
                                *d = match *d {
                                    Direction::Ltr => Direction::Rtl,
                                    Direction::Rtl => Direction::Ltr,
                                };
                            });
                        }
                    >
                        "Toggle direction"
                    </Button>
                    <span class="text-sm font-medium">"is_rtl"</span>
                    <BoolBadge value=is_rtl />
                </div>

                <DirectionProvider
                    dir=dir
                    class="flex flex-col gap-3 rounded-md border border-border bg-muted/30 p-4"
                        .to_string()
                >
                    <DirectionSample />
                </DirectionProvider>
            </Demo>
        </Section>
    }
}

/// Lives inside the [`DirectionProvider`] above so it reads the ambient
/// [`Direction`] through [`use_direction`] rather than a passed-in prop.
#[component]
fn DirectionSample() -> impl IntoView {
    let dir = use_direction();

    view! {
        <p class="text-sm text-muted-foreground">
            "use_direction reports: "
            <span class="font-medium text-foreground">
                {move || match dir.get() {
                    Direction::Ltr => "Ltr",
                    Direction::Rtl => "Rtl",
                }}
            </span>
        </p>
        <div class="flex items-center gap-3">
            <Icon icon=icondata::LuChevronLeft attr:class="size-4 text-muted-foreground" />
            <span class="text-sm">
                "Layout, padding and the chevrons mirror with the reading direction."
            </span>
            <Icon icon=icondata::LuChevronRight attr:class="size-4 text-muted-foreground" />
        </div>
        <div class="flex gap-2">
            <Button variant=ButtonVariant::Default>"Confirm"</Button>
            <Button variant=ButtonVariant::Ghost>"Cancel"</Button>
        </div>
    }
}

#[component]
fn MediaQuerySection() -> impl IntoView {
    let is_wide = use_media_query("(min-width: 1024px)");
    let prefers_dark = use_media_query("(prefers-color-scheme: dark)");
    let prefers_reduced_motion = use_media_query("(prefers-reduced-motion: reduce)");
    let is_landscape = use_media_query("(orientation: landscape)");

    view! {
        <Section
            title="Media queries"
            description="use_media_query tracks any CSS media feature and updates live as the viewport or OS preference changes."
        >
            <Demo col=true>
                <MediaQueryRow query="(min-width: 1024px)" value=is_wide />
                <MediaQueryRow query="(prefers-color-scheme: dark)" value=prefers_dark />
                <MediaQueryRow
                    query="(prefers-reduced-motion: reduce)"
                    value=prefers_reduced_motion
                />
                <MediaQueryRow query="(orientation: landscape)" value=is_landscape />
            </Demo>
        </Section>
    }
}

/// One labelled row pairing a media-query string with its live boolean result.
#[component]
fn MediaQueryRow(#[prop(into)] query: String, #[prop(into)] value: Signal<bool>) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between gap-4">
            <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{query}</code>
            <BoolBadge value=value />
        </div>
    }
}

#[component]
fn MobileSection() -> impl IntoView {
    let is_mobile = use_is_mobile();

    view! {
        <Section
            title="Mobile breakpoint"
            description="use_is_mobile is built on use_media_query and reports whether the viewport is narrower than the md breakpoint."
        >
            <Demo>
                <span class="text-sm font-medium">"is_mobile"</span>
                <BoolBadge value=is_mobile />
                <span class="text-sm text-muted-foreground">
                    {format!("(max-width: {}px)", MOBILE_BREAKPOINT - 1)}
                </span>
            </Demo>
            <Demo label="Branching on the breakpoint">
                {move || {
                    if is_mobile.get() {
                        view! { <span class="text-sm">"Compact: render a drawer here."</span> }
                            .into_any()
                    } else {
                        view! {
                            <span class="text-sm">"Roomy: render a side-by-side dialog here."</span>
                        }
                            .into_any()
                    }
                }}
            </Demo>
        </Section>
    }
}

#[component]
fn SsrNoteSection() -> impl IntoView {
    view! {
        <Section title="Server rendering">
            <Demo>
                <div class="flex items-start gap-3 text-sm text-muted-foreground">
                    <Icon
                        icon=icondata::LuInfo
                        attr:class="mt-0.5 size-4 shrink-0 text-foreground"
                    />
                    <p class="max-w-prose">
                        "These hooks have no viewport or stored preference on the server, so they
                        report "
                        <code class="rounded bg-muted px-1 py-0.5 font-mono text-xs">"false"</code>
                        " during SSR and the first client paint. A client-only effect then
                        resolves the real value, which keeps server and client markup identical and
                        avoids a hydration mismatch."
                    </p>
                </div>
            </Demo>
        </Section>
    }
}
