use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{
    AlertDialog, AlertDialogAction, AlertDialogBackdrop, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, Button,
    ButtonSize, ButtonVariant, Command, CommandDialog, CommandDialogProvider, CommandDialogTrigger,
    CommandEmpty, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem,
    CommandList, ContextMenu, ContextMenuContent, ContextMenuGroup, ContextMenuItem,
    ContextMenuLabel, ContextMenuSub, ContextMenuSubContent, ContextMenuSubItem,
    ContextMenuSubTrigger, ContextMenuTrigger, Dialog, DialogAction, DialogBody, DialogClose,
    DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger,
    Drawer, DrawerBody, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHandle,
    DrawerHeader, DrawerPosition, DrawerTitle, DrawerTrigger, DrawerVariant, DropdownMenu,
    DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem,
    DropdownMenuItemVariant, DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItem,
    DropdownMenuSeparator, DropdownMenuTrigger, HoverCard, HoverCardContent, HoverCardSide,
    HoverCardTrigger, Kbd, KbdGroup, Popover, PopoverAlign, PopoverContent, PopoverDescription,
    PopoverTitle, PopoverTrigger, Sheet, SheetBody, SheetClose, SheetContent, SheetDescription,
    SheetDirection, SheetFooter, SheetHeader, SheetTitle, SheetTrigger, Tooltip, TooltipContent,
    TooltipPosition, TooltipTrigger, use_is_mobile,
};

use super::{Demo, Page, Section};

/// Dialogs, alert dialogs, sheets, drawers, popovers, hover cards, tooltips,
/// menus and the command palette.
#[component]
pub fn OverlaysPage() -> impl IntoView {
    view! {
        <Page
            title="Overlays"
            subtitle="Dialogs, sheets, drawers, popovers, menus and the command palette."
        >
            <DialogSection />
            <AlertDialogSection />
            <SheetSection />
            <DrawerSection />
            <PopoverSection />
            <HoverCardSection />
            <TooltipSection />
            <DropdownMenuSection />
            <ContextMenuSection />
            <CommandSection />
            <ResponsiveSection />
        </Page>
    }
}

#[component]
fn DialogSection() -> impl IntoView {
    view! {
        <Section
            title="Dialog"
            description="Modal panel with a backdrop. Focuses itself on open and returns focus to the trigger on close."
        >
            <Demo>
                <Dialog>
                    <DialogTrigger>"Edit profile"</DialogTrigger>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>"Edit profile"</DialogTitle>
                            <DialogDescription>
                                "Make changes to your account here. Click save when you are done."
                            </DialogDescription>
                        </DialogHeader>
                        <DialogBody>
                            <p class="text-sm text-muted-foreground">
                                "Your display name is shown next to every booking you make."
                            </p>
                        </DialogBody>
                        <DialogFooter>
                            <DialogClose>"Cancel"</DialogClose>
                            <DialogAction>"Save changes"</DialogAction>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
                <Dialog>
                    <DialogTrigger variant=ButtonVariant::Destructive>"Delete site"</DialogTrigger>
                    <DialogContent close_on_backdrop_click=false>
                        <DialogHeader>
                            <DialogTitle>"Delete this site?"</DialogTitle>
                            <DialogDescription>
                                "Backdrop clicks are disabled — choose an action explicitly."
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter>
                            <DialogClose>"Keep site"</DialogClose>
                            <DialogAction variant=ButtonVariant::Destructive>"Delete"</DialogAction>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </Demo>
        </Section>
    }
}

#[component]
fn AlertDialogSection() -> impl IntoView {
    let open = RwSignal::new(false);

    view! {
        <Section
            title="Alert dialog"
            description="A confirmation interruption. The family is static markup, so the page wires its open state."
        >
            <Demo>
                <AlertDialog>
                    <Button
                        variant=ButtonVariant::Destructive
                        on:click=move |_| {
                            open.set(true);
                        }
                    >
                        "Cancel booking"
                    </Button>
                    <Show when=move || open.get()>
                        <AlertDialogBackdrop on:click=move |_| {
                            open.set(false);
                        } />
                        <AlertDialogContent>
                            <AlertDialogHeader>
                                <AlertDialogTitle>"Cancel this booking?"</AlertDialogTitle>
                                <AlertDialogDescription>
                                    "This frees the desk for the day and cannot be undone."
                                </AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                                <AlertDialogCancel on:click=move |_| {
                                    open.set(false);
                                }>"Keep booking"</AlertDialogCancel>
                                <AlertDialogAction
                                    variant=ButtonVariant::Destructive
                                    on:click=move |_| {
                                        open.set(false);
                                    }
                                >
                                    "Cancel booking"
                                </AlertDialogAction>
                            </AlertDialogFooter>
                        </AlertDialogContent>
                    </Show>
                </AlertDialog>
            </Demo>
        </Section>
    }
}

#[component]
fn SheetSection() -> impl IntoView {
    view! {
        <Section
            title="Sheet"
            description="Edge-anchored modal panel that slides in. Choose the side it enters from."
        >
            <Demo>
                <Sheet>
                    <SheetTrigger>"Open right"</SheetTrigger>
                    <SheetContent>
                        <SheetHeader>
                            <SheetTitle>"Booking details"</SheetTitle>
                            <SheetDescription>
                                "Inspect and edit a desk reservation."
                            </SheetDescription>
                        </SheetHeader>
                        <SheetBody>
                            <p class="px-4 text-sm text-muted-foreground">
                                "Desk 14B · Level 3 · 09:00 – 17:00"
                            </p>
                        </SheetBody>
                        <SheetFooter>
                            <SheetClose>"Close"</SheetClose>
                            <SheetClose variant=ButtonVariant::Default>"Save"</SheetClose>
                        </SheetFooter>
                    </SheetContent>
                </Sheet>
                <Sheet>
                    <SheetTrigger>"Open left"</SheetTrigger>
                    <SheetContent direction=SheetDirection::Left>
                        <SheetHeader>
                            <SheetTitle>"Filters"</SheetTitle>
                            <SheetDescription>
                                "Narrow the list of available desks."
                            </SheetDescription>
                        </SheetHeader>
                    </SheetContent>
                </Sheet>
                <Sheet>
                    <SheetTrigger>"Open top"</SheetTrigger>
                    <SheetContent direction=SheetDirection::Top>
                        <SheetHeader>
                            <SheetTitle>"Notifications"</SheetTitle>
                            <SheetDescription>
                                "Recent activity across your sites."
                            </SheetDescription>
                        </SheetHeader>
                    </SheetContent>
                </Sheet>
                <Sheet>
                    <SheetTrigger>"Open bottom"</SheetTrigger>
                    <SheetContent direction=SheetDirection::Bottom>
                        <SheetHeader>
                            <SheetTitle>"Quick actions"</SheetTitle>
                            <SheetDescription>"Common shortcuts for this page."</SheetDescription>
                        </SheetHeader>
                    </SheetContent>
                </Sheet>
            </Demo>
        </Section>
    }
}

#[component]
fn DrawerSection() -> impl IntoView {
    view! {
        <Section
            title="Drawer"
            description="An edge-anchored panel with a drag handle, in inset and floating surface variants."
        >
            <Demo>
                <Drawer>
                    <DrawerTrigger>"Bottom drawer"</DrawerTrigger>
                    <DrawerContent>
                        <DrawerHandle />
                        <DrawerBody>
                            <DrawerHeader>
                                <DrawerTitle>"Confirm reservation"</DrawerTitle>
                                <DrawerDescription>
                                    "Review the details before you commit the desk."
                                </DrawerDescription>
                            </DrawerHeader>
                            <DrawerFooter>
                                <DrawerClose>"Back"</DrawerClose>
                                <DrawerClose variant=ButtonVariant::Default>"Confirm"</DrawerClose>
                            </DrawerFooter>
                        </DrawerBody>
                    </DrawerContent>
                </Drawer>
                <Drawer>
                    <DrawerTrigger>"Right · floating"</DrawerTrigger>
                    <DrawerContent position=DrawerPosition::Right variant=DrawerVariant::Floating>
                        <DrawerBody>
                            <DrawerHeader>
                                <DrawerTitle>"Activity"</DrawerTitle>
                                <DrawerDescription>
                                    "A floating panel detached from the edge."
                                </DrawerDescription>
                            </DrawerHeader>
                            <DrawerFooter>
                                <DrawerClose>"Close"</DrawerClose>
                            </DrawerFooter>
                        </DrawerBody>
                    </DrawerContent>
                </Drawer>
                <Drawer>
                    <DrawerTrigger>"Left drawer"</DrawerTrigger>
                    <DrawerContent position=DrawerPosition::Left>
                        <DrawerBody>
                            <DrawerHeader>
                                <DrawerTitle>"Navigation"</DrawerTitle>
                                <DrawerDescription>
                                    "Jump between sites and floors."
                                </DrawerDescription>
                            </DrawerHeader>
                        </DrawerBody>
                    </DrawerContent>
                </Drawer>
            </Demo>
        </Section>
    }
}

#[component]
fn PopoverSection() -> impl IntoView {
    view! {
        <Section
            title="Popover"
            description="An anchored panel toggled by its trigger; dismisses on Escape or an outside click."
        >
            <Demo>
                <Popover>
                    <PopoverTrigger>"Open · center"</PopoverTrigger>
                    <PopoverContent>
                        <PopoverTitle>"Dimensions"</PopoverTitle>
                        <PopoverDescription>
                            "Set the size of the booking grid for this floor."
                        </PopoverDescription>
                    </PopoverContent>
                </Popover>
                <Popover align=PopoverAlign::Start>
                    <PopoverTrigger>"Aligned start"</PopoverTrigger>
                    <PopoverContent>
                        <PopoverTitle>"Start"</PopoverTitle>
                        <PopoverDescription>
                            "Anchored to the trigger's left edge."
                        </PopoverDescription>
                    </PopoverContent>
                </Popover>
                <Popover align=PopoverAlign::End>
                    <PopoverTrigger>"Aligned end"</PopoverTrigger>
                    <PopoverContent>
                        <PopoverTitle>"End"</PopoverTitle>
                        <PopoverDescription>
                            "Anchored to the trigger's right edge."
                        </PopoverDescription>
                    </PopoverContent>
                </Popover>
                <Popover align=PopoverAlign::EndOuter>
                    <PopoverTrigger>"To the right"</PopoverTrigger>
                    <PopoverContent>
                        <PopoverTitle>"End outer"</PopoverTitle>
                        <PopoverDescription>"Flies out beside the trigger."</PopoverDescription>
                    </PopoverContent>
                </Popover>
            </Demo>
        </Section>
    }
}

#[component]
fn HoverCardSection() -> impl IntoView {
    view! {
        <Section
            title="Hover card"
            description="A rich card revealed on hover or focus after a short delay."
        >
            <Demo>
                <HoverCard>
                    <HoverCardTrigger>
                        <Button variant=ButtonVariant::Link>"@avery"</Button>
                    </HoverCardTrigger>
                    <HoverCardContent>
                        <div class="flex flex-col gap-1">
                            <span class="font-semibold">"Avery Okonkwo"</span>
                            <span class="text-sm text-muted-foreground">
                                "Facilities lead · Amsterdam HQ"
                            </span>
                        </div>
                    </HoverCardContent>
                </HoverCard>
                <HoverCard side=HoverCardSide::Right>
                    <HoverCardTrigger>
                        <Button variant=ButtonVariant::Outline>
                            <Icon icon=icondata::LuInfo attr:class="size-4" />
                            "Hover right"
                        </Button>
                    </HoverCardTrigger>
                    <HoverCardContent>
                        <PopoverTitle>"On the right"</PopoverTitle>
                        <span class="text-sm text-muted-foreground">
                            "The card anchors to the trigger's right edge."
                        </span>
                    </HoverCardContent>
                </HoverCard>
            </Demo>
        </Section>
    }
}

#[component]
fn TooltipSection() -> impl IntoView {
    view! {
        <Section
            title="Tooltip"
            description="A short hint shown on hover or focus, oriented to any side."
        >
            <Demo>
                <Tooltip>
                    <TooltipTrigger>
                        <Button variant=ButtonVariant::Outline>"Top"</Button>
                    </TooltipTrigger>
                    <TooltipContent>"Default placement"</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger>
                        <Button variant=ButtonVariant::Outline>"Right"</Button>
                    </TooltipTrigger>
                    <TooltipContent position=TooltipPosition::Right>"To the right"</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger>
                        <Button variant=ButtonVariant::Outline>"Bottom"</Button>
                    </TooltipTrigger>
                    <TooltipContent position=TooltipPosition::Bottom>"Below"</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger>
                        <Button size=ButtonSize::Icon attr:aria-label="Settings">
                            <Icon icon=icondata::LuSettings attr:class="size-4" />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent position=TooltipPosition::Left>"Settings"</TooltipContent>
                </Tooltip>
            </Demo>
        </Section>
    }
}

#[component]
fn DropdownMenuSection() -> impl IntoView {
    let density = RwSignal::new("comfortable".to_string());
    let show_weekends = RwSignal::new(true);

    view! {
        <Section
            title="Dropdown menu"
            description="A button-triggered menu with a label, a separator, a radio group, a checkbox item and a destructive action."
        >
            <Demo>
                <DropdownMenu>
                    <DropdownMenuTrigger>
                        "Options" <Icon icon=icondata::LuChevronDown attr:class="size-4" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                        <DropdownMenuLabel>"My account"</DropdownMenuLabel>
                        <DropdownMenuGroup>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuSettings attr:class="size-4" />
                                "Settings"
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuCopy attr:class="size-4" />
                                "Duplicate"
                            </DropdownMenuItem>
                        </DropdownMenuGroup>
                        <DropdownMenuSeparator />
                        <DropdownMenuLabel>"Density"</DropdownMenuLabel>
                        <DropdownMenuRadioGroup value=density>
                            <DropdownMenuRadioItem value="comfortable"
                                .to_string()>"Comfortable"</DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="compact"
                                .to_string()>"Compact"</DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>
                        <DropdownMenuSeparator />
                        <DropdownMenuCheckboxItem checked=show_weekends>
                            "Show weekends"
                        </DropdownMenuCheckboxItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem variant=DropdownMenuItemVariant::Destructive>
                            <Icon icon=icondata::LuTrash2 attr:class="size-4" />
                            "Delete"
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
                <span class="text-sm text-muted-foreground">
                    {move || format!("Density: {}", density.get())}
                </span>
            </Demo>
        </Section>
    }
}

#[component]
fn ContextMenuSection() -> impl IntoView {
    view! {
        <Section
            title="Context menu"
            description="Right-click the surface below to open a menu anchored at the cursor, including a nested submenu."
        >
            <Demo>
                <ContextMenu>
                    <ContextMenuTrigger>
                        <div class="flex justify-center items-center w-72 h-32 text-sm rounded-md border border-dashed select-none border-border text-muted-foreground">
                            "Right-click here"
                        </div>
                    </ContextMenuTrigger>
                    <ContextMenuContent>
                        <ContextMenuLabel>"Desk 14B"</ContextMenuLabel>
                        <ContextMenuGroup>
                            <ContextMenuItem>
                                <Icon icon=icondata::LuCalendar attr:class="size-4" />
                                "Book for today"
                            </ContextMenuItem>
                            <ContextMenuItem>
                                <Icon icon=icondata::LuCopy attr:class="size-4" />
                                "Copy link"
                            </ContextMenuItem>
                            <ContextMenuSub>
                                <ContextMenuSubTrigger>"Move to floor"</ContextMenuSubTrigger>
                                <ContextMenuSubContent>
                                    <ContextMenuSubItem>"Level 1"</ContextMenuSubItem>
                                    <ContextMenuSubItem>"Level 2"</ContextMenuSubItem>
                                    <ContextMenuSubItem>"Level 3"</ContextMenuSubItem>
                                </ContextMenuSubContent>
                            </ContextMenuSub>
                            <ContextMenuItem>
                                <Icon icon=icondata::LuTrash2 attr:class="size-4" />
                                "Release desk"
                            </ContextMenuItem>
                        </ContextMenuGroup>
                    </ContextMenuContent>
                </ContextMenu>
            </Demo>
        </Section>
    }
}

#[component]
fn CommandSection() -> impl IntoView {
    view! {
        <Section
            title="Command palette"
            description="Press Cmd/Ctrl + K (or / outside a text field) to open the palette, or use the button. Type to filter; arrow keys and Enter select."
        >
            <Demo>
                <CommandDialogProvider>
                    <CommandDialogTrigger>
                        <Icon icon=icondata::LuSearch attr:class="size-4" />
                        "Search commands"
                        <KbdGroup>
                            <Kbd>"\u{2318}"</Kbd>
                            <Kbd>"K"</Kbd>
                        </KbdGroup>
                    </CommandDialogTrigger>
                    <CommandDialog>
                        <Command>
                            <div class="flex gap-2 items-center px-3 border-b border-border">
                                <Icon
                                    icon=icondata::LuSearch
                                    attr:class="size-4 text-muted-foreground"
                                />
                                <CommandInput attr:placeholder="Type a command or search..." />
                            </div>
                            <CommandList>
                                <CommandEmpty>"No results found."</CommandEmpty>
                                <CommandGroup>
                                    <CommandGroupLabel>"Bookings"</CommandGroupLabel>
                                    <CommandItem value="New booking">
                                        <Icon icon=icondata::LuPlus attr:class="size-4" />
                                        "New booking"
                                    </CommandItem>
                                    <CommandItem value="Find a desk">
                                        <Icon icon=icondata::LuSearch attr:class="size-4" />
                                        "Find a desk"
                                    </CommandItem>
                                    <CommandItem value="Cancel booking">
                                        <Icon icon=icondata::LuTrash2 attr:class="size-4" />
                                        "Cancel booking"
                                    </CommandItem>
                                </CommandGroup>
                                <CommandGroup>
                                    <CommandGroupLabel>"Settings"</CommandGroupLabel>
                                    <CommandItem value="Open settings">
                                        <Icon icon=icondata::LuSettings attr:class="size-4" />
                                        "Open settings"
                                    </CommandItem>
                                    <CommandItem value="Toggle theme">
                                        <Icon icon=icondata::LuMoon attr:class="size-4" />
                                        "Toggle theme"
                                    </CommandItem>
                                    <CommandItem value="Notifications" disabled=true>
                                        <Icon icon=icondata::LuBell attr:class="size-4" />
                                        "Notifications (disabled)"
                                    </CommandItem>
                                </CommandGroup>
                            </CommandList>
                            <CommandFooter>
                                <span class="flex gap-1.5 items-center">
                                    <Kbd>"\u{2191}"</Kbd>
                                    <Kbd>"\u{2193}"</Kbd>
                                    "to navigate"
                                </span>
                                <span class="flex gap-1.5 items-center">
                                    <Kbd>"Esc"</Kbd>
                                    "to close"
                                </span>
                            </CommandFooter>
                        </Command>
                    </CommandDialog>
                </CommandDialogProvider>
            </Demo>
        </Section>
    }
}

#[component]
fn ResponsiveSection() -> impl IntoView {
    let is_mobile = use_is_mobile();

    view! {
        <Section
            title="Responsive switch"
            description="use_is_mobile reports whether the viewport is below the md breakpoint — the cue for swapping a dialog for a drawer. Resize the window to watch it flip."
        >
            <Demo col=true>
                <span class="text-sm text-muted-foreground">
                    {move || {
                        if is_mobile.get() {
                            "Mobile layout — a drawer is used here."
                        } else {
                            "Desktop layout — a dialog is used here."
                        }
                    }}
                </span>
                {move || {
                    if is_mobile.get() {
                        view! {
                            <Drawer>
                                <DrawerTrigger>"Adapt: drawer"</DrawerTrigger>
                                <DrawerContent>
                                    <DrawerHandle />
                                    <DrawerBody>
                                        <DrawerHeader>
                                            <DrawerTitle>"Adaptive panel"</DrawerTitle>
                                            <DrawerDescription>
                                                "Rendered as a drawer on narrow viewports."
                                            </DrawerDescription>
                                        </DrawerHeader>
                                        <DrawerFooter>
                                            <DrawerClose>"Close"</DrawerClose>
                                        </DrawerFooter>
                                    </DrawerBody>
                                </DrawerContent>
                            </Drawer>
                        }
                            .into_any()
                    } else {
                        view! {
                            <Dialog>
                                <DialogTrigger>"Adapt: dialog"</DialogTrigger>
                                <DialogContent>
                                    <DialogHeader>
                                        <DialogTitle>"Adaptive panel"</DialogTitle>
                                        <DialogDescription>
                                            "Rendered as a dialog on wide viewports."
                                        </DialogDescription>
                                    </DialogHeader>
                                    <DialogFooter>
                                        <DialogClose>"Close"</DialogClose>
                                    </DialogFooter>
                                </DialogContent>
                            </Dialog>
                        }
                            .into_any()
                    }
                }}
            </Demo>
        </Section>
    }
}
