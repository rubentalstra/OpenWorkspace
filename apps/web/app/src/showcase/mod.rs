//! Internal developer gallery for the `ui` kit, organised into category pages so
//! every ported component can be eyeballed and a11y-checked. Each demo is framed
//! in a `Card`. Grows as each wave lands; the sidebar-07 shell returns with the
//! Sidebar family (Wave 4–5).

use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::Outlet;
use ui::{
    Accordion, AccordionContent, AccordionItem, AccordionTrigger, Alert, AlertDescription,
    AlertTitle, AlertVariant, AspectRatio, Avatar, AvatarFallback, AvatarGroup, Badge,
    BadgeVariant, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage,
    BreadcrumbSeparator, Button, ButtonGroup, ButtonGroupText, ButtonSize, ButtonVariant, Card,
    CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Checkbox, Collapsible,
    CollapsibleContent, CollapsibleTrigger, Empty, EmptyDescription, EmptyHeader, EmptyMedia,
    EmptyTitle, Field, FieldDescription, FieldLabel, Input, InputGroup, InputGroupAddon,
    InputGroupInput, InputGroupText, Item, ItemContent, ItemDescription, ItemGroup, ItemMedia,
    ItemTitle, Kbd, KbdGroup, Label, NativeSelect, NativeSelectOption, Pagination,
    PaginationContent, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious,
    Progress, RadioGroup, RadioGroupItem, Separator, SeparatorOrientation, Skeleton, Slider,
    Spinner, Switch, Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Tabs,
    TabsContent, TabsList, TabsTrigger, Textarea, Toggle, ToggleGroup, ToggleGroupItem,
    use_theme_mode,
};
use ui::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger, ScrollArea,
};
use ui::{
    Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription,
    AttachmentMedia, AttachmentTitle, Bubble, BubbleAlign, BubbleContent, BubbleGroup, Carousel,
    CarouselContent, CarouselItem, CarouselNext, CarouselPrevious, ChartContainer, ChartSeries,
    Marker, MarkerContent, MarkerIcon, Message, MessageAvatar, MessageContent, MessageGroup,
    MessageHeader, MessageScroller, MessageScrollerButton, MessageScrollerContent,
    MessageScrollerItem, MessageScrollerViewport,
};
use ui::{
    DatePicker, Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader,
    DialogTitle, DialogTrigger, DropdownMenu, DropdownMenuContent, DropdownMenuItem,
    DropdownMenuItemVariant, DropdownMenuLabel, DropdownMenuTrigger, Popover, PopoverContent,
    PopoverDescription, PopoverTitle, PopoverTrigger, Select, SelectContent, SelectItem,
    SelectTrigger, SelectValue, Sheet, SheetContent, SheetDescription, SheetHeader, SheetSide,
    SheetTitle, SheetTrigger, Tooltip, TooltipContent, TooltipTrigger,
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

/// Buttons, badges, toggles, kbd.
#[component]
pub fn ButtonsPage() -> impl IntoView {
    let bold = RwSignal::new(false);
    let marks = RwSignal::new(vec!["bold".to_owned()]);
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
            <Demo title="Button group">
                <ButtonGroup>
                    <ButtonGroupText>"View"</ButtonGroupText>
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                        "Day"
                    </Button>
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                        "Week"
                    </Button>
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                        "Month"
                    </Button>
                </ButtonGroup>
            </Demo>
            <Demo title="Toggle group">
                <ToggleGroup value=marks on_change=Callback::new(move |v| marks.set(v))>
                    <ToggleGroupItem value="bold">"B"</ToggleGroupItem>
                    <ToggleGroupItem value="italic">"I"</ToggleGroupItem>
                    <ToggleGroupItem value="underline">"U"</ToggleGroupItem>
                </ToggleGroup>
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
    let plan = RwSignal::new("map".to_owned());
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
            <Demo title="Radio group">
                <RadioGroup
                    value=plan
                    on_change=Callback::new(move |v| plan.set(v))
                    class="flex flex-col gap-2"
                >
                    <label class="flex items-center gap-2 text-sm">
                        <RadioGroupItem value="map" />
                        "Map"
                    </label>
                    <label class="flex items-center gap-2 text-sm">
                        <RadioGroupItem value="list" />
                        "List"
                    </label>
                </RadioGroup>
            </Demo>
            <Demo title="Field">
                <Field>
                    <FieldLabel>"Display name"</FieldLabel>
                    <Input attr:placeholder="Ada Lovelace" />
                    <FieldDescription>"Shown to your teammates."</FieldDescription>
                </Field>
            </Demo>
            <Demo title="Input group">
                <InputGroup>
                    <InputGroupAddon>
                        <InputGroupText>"@"</InputGroupText>
                    </InputGroupAddon>
                    <InputGroupInput attr:placeholder="username" />
                </InputGroup>
            </Demo>
        </PageShell>
    }
}

/// Tables, lists, avatars, breadcrumbs, pagination, chart.
#[component]
#[expect(
    clippy::too_many_lines,
    reason = "a flat gallery page of independent demos"
)]
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
            <Demo title="Chart">
                <ChartContainer
                    id="bookings"
                    config=vec![
                        ChartSeries {
                            key: "bookings".into(),
                            label: "Bookings".into(),
                            color: "var(--chart-1)".into(),
                        },
                    ]
                    class="aspect-auto h-40 w-full"
                >
                    <svg viewBox="0 0 200 100" class="h-full w-full" preserveAspectRatio="none">
                        {[40_i32, 65, 50, 80, 60, 90]
                            .into_iter()
                            .enumerate()
                            .map(|(i, h)| {
                                let x = 12 + i32::try_from(i).unwrap_or(0) * 32;
                                view! {
                                    <rect
                                        x=x.to_string()
                                        y=(100 - h).to_string()
                                        width="20"
                                        height=h.to_string()
                                        rx="2"
                                        fill="var(--color-bookings)"
                                    ></rect>
                                }
                            })
                            .collect_view()}
                    </svg>
                </ChartContainer>
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

/// Anchored popups and modals.
#[component]
pub fn OverlaysPage() -> impl IntoView {
    let trigger = "cn-button cn-button-variant-outline cn-button-size-default px-3";
    view! {
        <PageShell title="Overlays" subtitle="Popups, menus, and modals.">
            <Demo title="Dialog">
                <Dialog>
                    <DialogTrigger class=trigger>"Open dialog"</DialogTrigger>
                    <DialogContent class="w-full max-w-md p-6">
                        <DialogHeader>
                            <DialogTitle>"Book a desk"</DialogTitle>
                            <DialogDescription>"Confirm your booking for today."</DialogDescription>
                        </DialogHeader>
                        <DialogFooter>
                            <DialogClose class=trigger>"Cancel"</DialogClose>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </Demo>
            <Demo title="Sheet">
                <Sheet>
                    <SheetTrigger class=trigger>"Open sheet"</SheetTrigger>
                    <SheetContent side=SheetSide::Right class="p-6">
                        <SheetHeader>
                            <SheetTitle>"Filters"</SheetTitle>
                            <SheetDescription>"Refine the desk list."</SheetDescription>
                        </SheetHeader>
                    </SheetContent>
                </Sheet>
            </Demo>
            <Demo title="Popover">
                <Popover>
                    <PopoverTrigger class=trigger>"Open popover"</PopoverTrigger>
                    <PopoverContent class="p-4">
                        <PopoverTitle>"Note"</PopoverTitle>
                        <PopoverDescription>"Anchored, dismissible content."</PopoverDescription>
                    </PopoverContent>
                </Popover>
            </Demo>
            <Demo title="Tooltip">
                <Tooltip>
                    <TooltipTrigger class=trigger>"Hover me"</TooltipTrigger>
                    <TooltipContent>"Helpful hint"</TooltipContent>
                </Tooltip>
            </Demo>
            <Demo title="Dropdown menu">
                <DropdownMenu>
                    <DropdownMenuTrigger class=trigger>"Open menu"</DropdownMenuTrigger>
                    <DropdownMenuContent>
                        <DropdownMenuLabel>"Account"</DropdownMenuLabel>
                        <DropdownMenuItem>"Profile"</DropdownMenuItem>
                        <DropdownMenuItem>"Settings"</DropdownMenuItem>
                        <DropdownMenuItem variant=DropdownMenuItemVariant::Destructive>
                            "Log out"
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </Demo>
            <Demo title="Select">
                <Select default_value="map">
                    <SelectTrigger class="w-44">
                        <SelectValue placeholder="Choose view" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="map" label="Map">
                            "Map"
                        </SelectItem>
                        <SelectItem value="list" label="List">
                            "List"
                        </SelectItem>
                        <SelectItem value="calendar" label="Calendar">
                            "Calendar"
                        </SelectItem>
                    </SelectContent>
                </Select>
            </Demo>
            <Demo title="Date picker">
                <DatePicker class="w-56" />
            </Demo>
            <Demo title="Alert dialog">
                <AlertDialog>
                    <AlertDialogTrigger class=trigger>"Cancel booking"</AlertDialogTrigger>
                    <AlertDialogContent class="max-w-md p-6">
                        <AlertDialogHeader>
                            <AlertDialogTitle>"Cancel this booking?"</AlertDialogTitle>
                            <AlertDialogDescription>
                                "This releases your desk for today. You can book again later."
                            </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                            <AlertDialogCancel>"Keep booking"</AlertDialogCancel>
                            <AlertDialogAction>"Cancel booking"</AlertDialogAction>
                        </AlertDialogFooter>
                    </AlertDialogContent>
                </AlertDialog>
            </Demo>
        </PageShell>
    }
}

/// Chat & maps building blocks: bubbles, messages, attachments, markers, scroller.
#[component]
pub fn ChatPage() -> impl IntoView {
    view! {
        <PageShell title="Chat" subtitle="Messaging and map building blocks.">
            <Demo title="Bubbles">
                <BubbleGroup class="w-full max-w-sm gap-2">
                    <Bubble align=BubbleAlign::Start>
                        <BubbleContent class="bg-muted rounded-2xl px-3 py-2 text-sm">
                            "Is desk A-12 free tomorrow?"
                        </BubbleContent>
                    </Bubble>
                    <Bubble align=BubbleAlign::End>
                        <BubbleContent class="bg-primary text-primary-foreground ml-auto rounded-2xl px-3 py-2 text-sm">
                            "Yep — booked it for you 👍"
                        </BubbleContent>
                    </Bubble>
                </BubbleGroup>
            </Demo>
            <Demo title="Message">
                <MessageGroup class="w-full max-w-sm gap-3">
                    <Message>
                        <MessageAvatar class="size-8 text-xs">"OV"</MessageAvatar>
                        <MessageContent class="gap-1">
                            <MessageHeader class="gap-2 text-sm">
                                <span class="font-medium">"Olivia"</span>
                                <span class="text-muted-foreground text-xs">"9:41"</span>
                            </MessageHeader>
                            <div class="bg-muted w-fit rounded-2xl px-3 py-2 text-sm">
                                "Heading to floor 2 now."
                            </div>
                        </MessageContent>
                    </Message>
                </MessageGroup>
            </Demo>
            <Demo title="Attachment">
                <Attachment class="w-72 gap-2 rounded-lg p-2">
                    <AttachmentMedia class="size-10 rounded-md">
                        <Icon icon=icondata::LuFileText attr:class="size-5" />
                    </AttachmentMedia>
                    <AttachmentContent class="self-center">
                        <AttachmentTitle>"floorplan.pdf"</AttachmentTitle>
                        <AttachmentDescription>"2.4 MB · PDF"</AttachmentDescription>
                    </AttachmentContent>
                    <AttachmentActions class="self-center">
                        <AttachmentAction>
                            <Icon icon=icondata::LuX attr:class="size-4" />
                        </AttachmentAction>
                    </AttachmentActions>
                </Attachment>
            </Demo>
            <Demo title="Marker">
                <div class="flex w-full flex-col gap-2 text-sm">
                    <Marker class="gap-2">
                        <MarkerIcon>
                            <Icon icon=icondata::LuMapPin attr:class="size-4" />
                        </MarkerIcon>
                        <MarkerContent>"Building A · Floor 2"</MarkerContent>
                    </Marker>
                    <Marker class="gap-2">
                        <MarkerIcon>
                            <Icon icon=icondata::LuMapPin attr:class="size-4" />
                        </MarkerIcon>
                        <MarkerContent>"Building B · Floor 1"</MarkerContent>
                    </Marker>
                </div>
            </Demo>
            <Demo title="Message scroller">
                <MessageScroller class="h-48 w-full rounded-md border">
                    <MessageScrollerViewport class="p-3">
                        <MessageScrollerContent class="gap-2">
                            {(1..=15)
                                .map(|n| {
                                    view! {
                                        <MessageScrollerItem>
                                            <div class="bg-muted rounded-md p-2 text-sm">
                                                {format!("Message {n}")}
                                            </div>
                                        </MessageScrollerItem>
                                    }
                                })
                                .collect_view()}
                        </MessageScrollerContent>
                    </MessageScrollerViewport>
                    <MessageScrollerButton />
                </MessageScroller>
            </Demo>
        </PageShell>
    }
}
