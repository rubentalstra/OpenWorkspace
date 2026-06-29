//! Anchored popups and modals.

use leptos::prelude::*;
use ui::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger, DatePicker, Dialog,
    DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    DialogTrigger, DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuItemVariant,
    DropdownMenuLabel, DropdownMenuTrigger, Popover, PopoverContent, PopoverDescription,
    PopoverTitle, PopoverTrigger, Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
    Sheet, SheetContent, SheetDescription, SheetHeader, SheetSide, SheetTitle, SheetTrigger,
    Tooltip, TooltipContent, TooltipTrigger,
};

use super::{Demo, PageShell};

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
