use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{
    Accordion, AccordionContent, AccordionDescription, AccordionHeader, AccordionItem,
    AccordionLink, AccordionTitle, AccordionTrigger, AccordionTriggerIcon, Animate, AnimateGroup,
    AnimateGroupItem, AnimateHoverVariant, AnimateVariant, AnimationFillMode, AspectRatio,
    BentoCell, BentoGrid, BentoGrid6, BentoRow, Collapsible, CollapsibleContent,
    CollapsibleTrigger, Draggable, DraggableItem, DraggableZone, Expandable, ExpandableContent,
    ExpandableTransition, ExpandableTrigger, Faq, FaqContent, FaqDescription, FaqLabel, FaqSection,
    FaqTitle, Image, Marquee, MarqueeOrientation, MarqueeWrapper, Mask, MaskSide, MaskWrapper,
    ScrollArea, ScrollAreaCorner, ScrollAreaViewport, ScrollBar, ScrollBarOrientation, Separator,
    SeparatorOrientation, SnapItem, SnapScrollArea, use_can_scroll_vertical, use_random_id,
    use_random_transition_name,
};

use super::{Demo, Page, Section};

/// Accordions, scroll areas, aspect ratios, animation and media.
#[component]
pub fn LayoutPage() -> impl IntoView {
    view! {
        <Page
            title="Layout"
            subtitle="Accordions, scroll areas, aspect ratios, animation and media."
        >
            <AccordionSection />
            <CollapsibleSection />
            <ExpandableSection />
            <FaqSectionDemo />
            <SeparatorSection />
            <AspectRatioSection />
            <ScrollAreaSection />
            <SnapScrollSection />
            <BentoSection />
            <MaskSection />
            <MarqueeSection />
            <AnimateSection />
            <ImageSection />
            <DragAndDropSection />
        </Page>
    }
}

#[component]
fn AccordionSection() -> impl IntoView {
    view! {
        <Section
            title="Accordion"
            description="Disclosure rows; several can stay open at once. Chevron and plus indicators."
        >
            <Demo col=true>
                <Accordion>
                    <AccordionItem default_open=true>
                        <AccordionTrigger>
                            <AccordionHeader>
                                <Icon icon=icondata::LuInfo />
                                <AccordionTitle>"What is OpenWorkspace?"</AccordionTitle>
                            </AccordionHeader>
                        </AccordionTrigger>
                        <AccordionContent>
                            <AccordionDescription>
                                "A self-hosted, multi-site workspace-booking platform built in Rust."
                            </AccordionDescription>
                        </AccordionContent>
                    </AccordionItem>
                    <AccordionItem>
                        <AccordionTrigger>
                            <AccordionHeader>
                                <Icon icon=icondata::LuCalendar />
                                <AccordionTitle>"How do bookings work?"</AccordionTitle>
                            </AccordionHeader>
                        </AccordionTrigger>
                        <AccordionContent>
                            <AccordionDescription>
                                "Pick a site, a date and a desk. The first release is desk-only."
                            </AccordionDescription>
                            <AccordionLink attr:href="/ui/dates">
                                <Icon icon=icondata::LuChevronRight />
                                "See the date pickers"
                            </AccordionLink>
                        </AccordionContent>
                    </AccordionItem>
                    <AccordionItem>
                        <AccordionTrigger icon=AccordionTriggerIcon::Plus>
                            <AccordionHeader>
                                <Icon icon=icondata::LuSettings />
                                <AccordionTitle>"Plus-style indicator"</AccordionTitle>
                            </AccordionHeader>
                        </AccordionTrigger>
                        <AccordionContent>
                            <AccordionDescription>
                                "This row swaps the chevron for a plus that rotates into an x."
                            </AccordionDescription>
                        </AccordionContent>
                    </AccordionItem>
                </Accordion>
            </Demo>
        </Section>
    }
}

#[component]
fn CollapsibleSection() -> impl IntoView {
    let open = RwSignal::new(false);

    view! {
        <Section
            title="Collapsible"
            description="A single disclosure region. Uncontrolled (default_open) and controlled."
        >
            <Demo col=true label="Uncontrolled">
                <Collapsible default_open=true class="w-full max-w-md">
                    <CollapsibleTrigger class="flex w-full items-center justify-between rounded-md border border-input bg-card px-4 py-2 text-sm font-medium [&_svg]:size-4">
                        "Site amenities" <Icon icon=icondata::LuChevronDown />
                    </CollapsibleTrigger>
                    <CollapsibleContent class="px-4 py-2 text-sm text-muted-foreground">
                        <p>"Standing desks, dual monitors, lockers and bike storage."</p>
                    </CollapsibleContent>
                </Collapsible>
            </Demo>
            <Demo col=true label="Controlled">
                <button
                    type="button"
                    class="w-fit text-sm font-medium underline"
                    on:click=move |_| {
                        open.update(|o| *o = !*o);
                    }
                >
                    {move || if open.get() { "Hide external state" } else { "Show external state" }}
                </button>
                <Collapsible open=open class="w-full max-w-md">
                    <CollapsibleContent class="rounded-md border border-input bg-muted/40 px-4 py-2 text-sm text-muted-foreground">
                        <p>"This panel's open state is driven by the button above."</p>
                    </CollapsibleContent>
                </Collapsible>
            </Demo>
        </Section>
    }
}

#[component]
fn ExpandableSection() -> impl IntoView {
    view! {
        <Section
            title="Expandable"
            description="A compact trigger that morphs into a larger panel, then collapses with its close button."
        >
            <Demo>
                <Expandable class="h-48 w-64 border border-input">
                    <ExpandableTransition class="flex h-full w-full items-center justify-center">
                        <ExpandableTrigger class="flex size-full items-center justify-center text-sm font-medium">
                            "Open details"
                        </ExpandableTrigger>
                    </ExpandableTransition>
                    <ExpandableContent class="flex flex-col gap-2 p-4">
                        <h4 class="text-sm font-semibold">"Desk 4B"</h4>
                        <p class="text-sm text-muted-foreground">
                            "Window seat, standing desk, near the kitchen."
                        </p>
                    </ExpandableContent>
                </Expandable>
            </Demo>
        </Section>
    }
}

#[component]
fn FaqSectionDemo() -> impl IntoView {
    view! {
        <Section
            title="FAQ"
            description="A stacked set of expandable FAQ entries built on the FaqSection disclosure."
        >
            <Demo>
                <Faq>
                    <FaqTitle>"Frequently asked"</FaqTitle>
                    <FaqDescription>"Everything booking-related, in one place."</FaqDescription>
                    <FaqSection default_open=true>
                        <FaqLabel>
                            "Can I book recurring desks?"
                            <Icon icon=icondata::LuChevronDown attr:class="size-4" />
                        </FaqLabel>
                        <FaqContent>
                            <p class="pb-4 text-sm text-muted-foreground">
                                "Yes — set a weekly pattern and it books each matching day automatically."
                            </p>
                        </FaqContent>
                    </FaqSection>
                    <FaqSection>
                        <FaqLabel>
                            "What happens if I do not check in?"
                            <Icon icon=icondata::LuChevronDown attr:class="size-4" />
                        </FaqLabel>
                        <FaqContent>
                            <p class="pb-4 text-sm text-muted-foreground">
                                "Unclaimed desks are released after the configured grace window."
                            </p>
                        </FaqContent>
                    </FaqSection>
                </Faq>
            </Demo>
        </Section>
    }
}

#[component]
fn SeparatorSection() -> impl IntoView {
    view! {
        <Section
            title="Separator"
            description="A thin rule for splitting content along either axis."
        >
            <Demo col=true label="Horizontal">
                <div class="w-full max-w-sm">
                    <p class="text-sm font-medium">"Account"</p>
                    <p class="text-sm text-muted-foreground">"Manage your profile and sessions."</p>
                    <Separator class="my-3" />
                    <p class="text-sm font-medium">"Sites"</p>
                    <p class="text-sm text-muted-foreground">
                        "Switch between bookable locations."
                    </p>
                </div>
            </Demo>
            <Demo label="Vertical">
                <div class="flex h-6 items-center gap-3 text-sm">
                    <span>"Profile"</span>
                    <Separator orientation=SeparatorOrientation::Vertical />
                    <span>"Bookings"</span>
                    <Separator orientation=SeparatorOrientation::Vertical />
                    <span>"Settings"</span>
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn AspectRatioSection() -> impl IntoView {
    view! {
        <Section title="Aspect ratio" description="Boxes locked to a fixed width-to-height ratio.">
            <Demo>
                <div class="w-48">
                    <AspectRatio ratio=Signal::derive(|| 16.0 / 9.0) class="rounded-lg bg-muted">
                        <div class="flex size-full items-center justify-center text-sm text-muted-foreground">
                            "16:9"
                        </div>
                    </AspectRatio>
                </div>
                <div class="w-48">
                    <AspectRatio ratio=Signal::derive(|| 1.0) class="rounded-lg bg-muted">
                        <div class="flex size-full items-center justify-center text-sm text-muted-foreground">
                            "1:1"
                        </div>
                    </AspectRatio>
                </div>
                <div class="w-48">
                    <AspectRatio ratio=Signal::derive(|| 4.0 / 3.0) class="rounded-lg bg-muted">
                        <div class="flex size-full items-center justify-center text-sm text-muted-foreground">
                            "4:3"
                        </div>
                    </AspectRatio>
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn ScrollAreaSection() -> impl IntoView {
    let (on_scroll, can_scroll_up, can_scroll_down) = use_can_scroll_vertical();
    let rows = (1..=24).collect::<Vec<_>>();

    view! {
        <Section
            title="Scroll area"
            description="A bounded viewport with a custom scrollbar. The edge hints are driven by use_can_scroll_vertical."
        >
            <Demo col=true>
                <div class="flex items-center gap-4 text-xs text-muted-foreground">
                    <span>
                        {move || if can_scroll_up.get() { "\u{2191} more above" } else { "top" }}
                    </span>
                    <span>
                        {move || {
                            if can_scroll_down.get() { "\u{2193} more below" } else { "bottom" }
                        }}
                    </span>
                </div>
                <ScrollArea class="h-56 w-72 rounded-md border border-input">
                    <ScrollAreaViewport on:scroll=on_scroll class="p-4">
                        <h4 class="mb-2 text-sm font-medium">"Recent bookings"</h4>
                        <ul class="flex flex-col gap-2 text-sm text-muted-foreground">
                            {rows
                                .into_iter()
                                .map(|n| view! { <li>{format!("Desk booking #{n}")}</li> })
                                .collect_view()}
                        </ul>
                    </ScrollAreaViewport>
                    <ScrollBar orientation=ScrollBarOrientation::Vertical />
                </ScrollArea>
            </Demo>
            <Demo col=true label="Both axes">
                <ScrollArea class="size-56 rounded-md border border-input">
                    <ScrollAreaViewport class="p-4">
                        <div class="h-72 w-[28rem] bg-gradient-to-br from-muted to-accent" />
                    </ScrollAreaViewport>
                    <ScrollBar orientation=ScrollBarOrientation::Vertical />
                    <ScrollBar orientation=ScrollBarOrientation::Horizontal />
                    <ScrollAreaCorner />
                </ScrollArea>
            </Demo>
        </Section>
    }
}

#[component]
fn SnapScrollSection() -> impl IntoView {
    let cards = ["Atrium", "Studio", "Loft", "Terrace", "Annex"];

    view! {
        <Section
            title="Snap scroll"
            description="A horizontal rail with CSS scroll-snap; each card rests at a snap point."
        >
            <Demo>
                <SnapScrollArea class="w-full max-w-md gap-4 pb-2">
                    {cards
                        .into_iter()
                        .map(|name| {
                            view! {
                                <SnapItem class="mr-4 w-40">
                                    <div class="flex h-28 w-40 items-center justify-center rounded-lg bg-muted text-sm font-medium">
                                        {name}
                                    </div>
                                </SnapItem>
                            }
                        })
                        .collect_view()}
                </SnapScrollArea>
            </Demo>
        </Section>
    }
}

#[component]
fn BentoSection() -> impl IntoView {
    view! {
        <Section
            title="Bento grid"
            description="Responsive feature-tile grids in four-tile and six-tile layouts."
        >
            <Demo col=true label="Four tiles">
                <BentoGrid class="w-full">
                    <BentoRow class="md:col-span-2">
                        <BentoCell>"Overview"</BentoCell>
                    </BentoRow>
                    <BentoRow>
                        <BentoCell>"Sites"</BentoCell>
                    </BentoRow>
                    <BentoRow>
                        <BentoCell>"Desks"</BentoCell>
                    </BentoRow>
                </BentoGrid>
            </Demo>
            <Demo col=true label="Six tiles">
                <BentoGrid6 class="w-full">
                    {["Bookings", "Teams", "Reports", "Floors", "Lockers", "Parking"]
                        .into_iter()
                        .map(|label| {
                            view! {
                                <BentoRow>
                                    <BentoCell>{label}</BentoCell>
                                </BentoRow>
                            }
                        })
                        .collect_view()}
                </BentoGrid6>
            </Demo>
        </Section>
    }
}

#[component]
fn MaskSection() -> impl IntoView {
    view! {
        <Section
            title="Mask"
            description="Edge-fade gradients pinned to a side of a clipped stage."
        >
            <Demo>
                <MaskWrapper class="min-h-48">
                    <p class="px-12 text-center text-sm text-muted-foreground">
                        "Content fades against the left and right edges of this stage."
                    </p>
                    <Mask side=MaskSide::Left />
                    <Mask side=MaskSide::Right />
                </MaskWrapper>
                <MaskWrapper class="min-h-48">
                    <p class="px-12 text-center text-sm text-muted-foreground">
                        "Top and bottom edge fades."
                    </p>
                    <Mask side=MaskSide::Top />
                    <Mask side=MaskSide::Bottom />
                </MaskWrapper>
            </Demo>
        </Section>
    }
}

#[component]
fn MarqueeSection() -> impl IntoView {
    let logos = ["Acme", "Globex", "Initech", "Umbrella", "Soylent"];
    let pill = move || {
        logos
            .into_iter()
            .map(|name| {
                view! {
                    <span class="rounded-full border border-input bg-card px-4 py-2 text-sm font-medium">
                        {name}
                    </span>
                }
            })
            .collect_view()
    };

    view! {
        <Section
            title="Marquee"
            description="CSS-only scrolling rows; pauses on hover and respects reduced-motion."
        >
            <Demo col=true label="Horizontal">
                <MarqueeWrapper class="min-h-32 p-8">
                    <Marquee repeat=4u32>{pill}</Marquee>
                </MarqueeWrapper>
            </Demo>
            <Demo col=true label="Reversed">
                <MarqueeWrapper class="min-h-32 p-8">
                    <Marquee reverse=true pause_on_hover=false repeat=4u32>
                        {pill}
                    </Marquee>
                </MarqueeWrapper>
            </Demo>
            <Demo col=true label="Vertical">
                <MarqueeWrapper orientation=MarqueeOrientation::Vertical class="h-64 p-8">
                    <Marquee orientation=MarqueeOrientation::Vertical repeat=4u32 class="h-full">
                        {pill}
                    </Marquee>
                </MarqueeWrapper>
            </Demo>
        </Section>
    }
}

#[component]
fn AnimateSection() -> impl IntoView {
    // use_random_* seeds unique CSS view-transition / id names so cloned demos
    // never collide; demonstrated here on the staggered group.
    let transition = use_random_transition_name();
    let group_id = use_random_id();

    let hovers = [
        ("Pop", AnimateHoverVariant::Pop),
        ("Tada", AnimateHoverVariant::Tada),
        ("Heartbeat", AnimateHoverVariant::Heartbeat),
        ("Wobble", AnimateHoverVariant::Wobble),
        ("RubberBand", AnimateHoverVariant::RubberBand),
        ("Jiggle", AnimateHoverVariant::Jiggle),
    ];

    view! {
        <Section
            title="Animate"
            description="Enter animations and a rich set of hover animations. Hover the tiles below."
        >
            <Demo col=true label="Enter animation">
                <Animate variant=AnimateVariant::FadeUp class="w-fit">
                    <span class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground">
                        "Fades up on mount"
                    </span>
                </Animate>
            </Demo>
            <Demo label="Hover animations">
                {hovers
                    .into_iter()
                    .map(|(label, variant)| {
                        view! {
                            <Animate hover_variant=variant class="w-fit">
                                <span class="rounded-md border border-input bg-card px-4 py-2 text-sm font-medium">
                                    {label}
                                </span>
                            </Animate>
                        }
                    })
                    .collect_view()}
            </Demo>
            <Demo col=true label="Staggered group">
                <AnimateGroup attr:id=group_id attr:style=transition class="flex flex-col gap-2">
                    {[0u32, 120, 240, 360]
                        .into_iter()
                        .map(|delay| {
                            view! {
                                <AnimateGroupItem
                                    variant=AnimateVariant::FadeUp
                                    delay_ms=delay
                                    fill_mode=AnimationFillMode::Both
                                    class="justify-start"
                                >
                                    <span class="rounded-md bg-muted px-4 py-2 text-sm">
                                        {format!("Item {}", delay / 120 + 1)}
                                    </span>
                                </AnimateGroupItem>
                            }
                        })
                        .collect_view()}
                </AnimateGroup>
            </Demo>
        </Section>
    }
}

const PLACEHOLDER_SVG: &str = "data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20\
width='320'%20height='180'%20viewBox='0%200%20320%20180'%3E%3Crect%20width='320'%20height='180'%20\
fill='%23cbd5e1'/%3E%3Ctext%20x='160'%20y='96'%20font-family='sans-serif'%20font-size='20'%20\
fill='%2364748b'%20text-anchor='middle'%3EWorkspace%3C/text%3E%3C/svg%3E";

#[component]
fn ImageSection() -> impl IntoView {
    let fallback = RwSignal::new(false);

    view! {
        <Section
            title="Image"
            description="Styled <img> with optional locked aspect ratio and load-error tracking. Sources are inline data: URIs because the CSP blocks external hosts."
        >
            <Demo>
                <div class="w-64">
                    <Image
                        aspect=Signal::derive(|| 16.0 / 9.0)
                        attr:src=PLACEHOLDER_SVG
                        attr:alt="Workspace placeholder"
                        attr:loading="lazy"
                        class="rounded-lg"
                    />
                </div>
                <div class="w-40">
                    <Image
                        attr:src=PLACEHOLDER_SVG
                        attr:alt="Workspace thumbnail"
                        class="rounded-full"
                    />
                </div>
            </Demo>
            <Demo col=true label="Error fallback">
                <div class="relative w-64">
                    <Image
                        aspect=Signal::derive(|| 16.0 / 9.0)
                        attr:src="/this-does-not-exist.png"
                        attr:alt="Broken source"
                        on_error=Callback::new(move |()| fallback.set(true))
                        class="rounded-lg"
                    />
                    <Show when=move || fallback.get()>
                        <div class="flex aspect-video w-full items-center justify-center rounded-lg bg-muted text-sm text-muted-foreground">
                            <Icon icon=icondata::LuEyeOff attr:class="size-4" />
                            <span class="ml-2">"Image unavailable"</span>
                        </div>
                    </Show>
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn DragAndDropSection() -> impl IntoView {
    let items = RwSignal::new(vec![
        "Plan the floor map".to_string(),
        "Import the desk list".to_string(),
        "Invite the team".to_string(),
        "Publish bookable sites".to_string(),
    ]);

    let on_reorder = Callback::new(move |(from, to): (usize, usize)| {
        items.update(|list| {
            if from < list.len() && to < list.len() {
                let moved = list.remove(from);
                list.insert(to, moved);
            }
        });
    });

    view! {
        <Section
            title="Drag and drop"
            description="Reorderable list. Reordering is data-driven: the callback mutates the backing signal and the DOM follows."
        >
            <Demo col=true>
                <Draggable on_reorder=on_reorder>
                    <DraggableZone>
                        {move || {
                            items
                                .get()
                                .into_iter()
                                .enumerate()
                                .map(|(index, label)| {
                                    view! {
                                        <DraggableItem index=index>
                                            <Icon
                                                icon=icondata::LuMenu
                                                attr:class="mr-3 text-muted-foreground"
                                            />
                                            <span class="text-sm">{label}</span>
                                        </DraggableItem>
                                    }
                                })
                                .collect_view()
                        }}
                    </DraggableZone>
                </Draggable>
                <p class="text-xs text-muted-foreground">"Drag a row onto another to reorder."</p>
            </Demo>
        </Section>
    }
}
