//! Structural primitives: separators, ratios, disclosure, scroll, carousel.

use leptos::prelude::*;
use ui::{
    Accordion, AccordionContent, AccordionItem, AccordionTrigger, AspectRatio, Carousel,
    CarouselContent, CarouselItem, CarouselNext, CarouselPrevious, Collapsible, CollapsibleContent,
    CollapsibleTrigger, ScrollArea, Separator, SeparatorOrientation, Tabs, TabsContent, TabsList,
    TabsTrigger,
};

use super::{Demo, PageShell};

/// Structural primitives.
#[component]
pub fn LayoutPage() -> impl IntoView {
    view! {
        <PageShell title="Layout" subtitle="Separators, ratios, and disclosure.">
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
            <Demo title="Tabs">
                <Tabs default_value="map" class="w-full gap-3">
                    <TabsList>
                        <TabsTrigger value="map">"Map"</TabsTrigger>
                        <TabsTrigger value="list">"List"</TabsTrigger>
                    </TabsList>
                    <TabsContent value="map">
                        <p class="text-muted-foreground text-sm">"The map view."</p>
                    </TabsContent>
                    <TabsContent value="list">
                        <p class="text-muted-foreground text-sm">"The list view."</p>
                    </TabsContent>
                </Tabs>
            </Demo>
            <Demo title="Accordion">
                <Accordion class="w-full">
                    <AccordionItem class="border-b">
                        <AccordionTrigger class="py-3 text-sm font-medium">
                            "How do I book a desk?"
                        </AccordionTrigger>
                        <AccordionContent>
                            <p class="text-muted-foreground pb-3 text-sm">
                                "Pick a floor, choose a free desk, and confirm."
                            </p>
                        </AccordionContent>
                    </AccordionItem>
                    <AccordionItem>
                        <AccordionTrigger class="py-3 text-sm font-medium">
                            "Can I cancel?"
                        </AccordionTrigger>
                        <AccordionContent>
                            <p class="text-muted-foreground pb-3 text-sm">
                                "Yes — cancel any time before the day starts."
                            </p>
                        </AccordionContent>
                    </AccordionItem>
                </Accordion>
            </Demo>
            <Demo title="Collapsible">
                <Collapsible class="w-full">
                    <CollapsibleTrigger class="cn-button cn-button-variant-outline cn-button-size-sm inline-flex items-center rounded-md">
                        "Toggle details"
                    </CollapsibleTrigger>
                    <CollapsibleContent class="pt-2">
                        <p class="text-muted-foreground text-sm">"Hidden details revealed."</p>
                    </CollapsibleContent>
                </Collapsible>
            </Demo>
            <Demo title="Scroll area">
                <ScrollArea class="h-32 w-full rounded-md border">
                    <div class="flex flex-col gap-2 p-3 text-sm">
                        {(1..=20)
                            .map(|n| view! { <div>{format!("Desk A-{n:02}")}</div> })
                            .collect_view()}
                    </div>
                </ScrollArea>
            </Demo>
            <Demo title="Carousel">
                <div class="w-full px-12">
                    <Carousel class="w-full">
                        <CarouselContent>
                            {(1..=5)
                                .map(|n| {
                                    view! {
                                        <CarouselItem>
                                            <div class="bg-muted flex aspect-square items-center justify-center rounded-md border text-2xl font-semibold">
                                                {n}
                                            </div>
                                        </CarouselItem>
                                    }
                                })
                                .collect_view()}
                        </CarouselContent>
                        <CarouselPrevious />
                        <CarouselNext />
                    </Carousel>
                </div>
            </Demo>
        </PageShell>
    }
}
