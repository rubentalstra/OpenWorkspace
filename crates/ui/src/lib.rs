//! ui — the OpenWorkspace design system: a 1:1 Leptos port of shadcn/ui (the
//! **Base UI** flavour, **nova** style). Components emit semantic `cn-*` +
//! `data-slot` classes themed by `apps/web/style/nova.css`; class merging is
//! `tw_merge` behind the first-party `cn!` facade (plus the `slot!`/`variants!`
//! macros under `tw/`); icons are `leptos_icons` + `icondata` (Lucide). Stable
//! Leptos 0.8, zero JavaScript.
//!
//! Components and hooks are re-exported flat: `ui::Button`, `ui::Card`,
//! `ui::use_is_mobile`, … The port grows wave by wave from the archived Base UI
//! source in `crates/ui/reference/shadcn` (see the rewrite plan and that README).

#[doc(hidden)]
pub mod components;
#[doc(hidden)]
pub mod hooks;
mod tw;

// The `variants!`/`cn!` macros expand `$crate::paste` / `$crate::tw_merge`, so
// those crates must be reachable at this crate's root.
#[doc(hidden)]
pub use paste;
#[doc(hidden)]
pub use tw_merge;

pub use components::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
pub use components::alert::{Alert, AlertAction, AlertDescription, AlertTitle, AlertVariant};
pub use components::alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogFooter, AlertDialogHeader, AlertDialogMedia, AlertDialogSize, AlertDialogTitle,
    AlertDialogTrigger,
};
pub use components::aspect_ratio::AspectRatio;
pub use components::avatar::{
    Avatar, AvatarBadge, AvatarFallback, AvatarGroup, AvatarGroupCount, AvatarImage, AvatarSize,
};
pub use components::badge::{Badge, BadgeVariant};
pub use components::breadcrumb::{
    Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage,
    BreadcrumbSeparator,
};
pub use components::button::{Button, ButtonSize, ButtonVariant};
pub use components::button_group::{
    ButtonGroup, ButtonGroupOrientation, ButtonGroupSeparator, ButtonGroupText,
};
pub use components::calendar::Calendar;
pub use components::card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardSize, CardTitle,
};
pub use components::checkbox::Checkbox;
pub use components::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
pub use components::combobox::{
    Combobox, ComboboxContent, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem,
    ComboboxLabel, ComboboxList, ComboboxSeparator, ComboboxTrigger, ComboboxValue,
};
pub use components::command::{
    Command, CommandDialog, CommandEmpty, CommandInput, CommandItem, CommandList,
};
pub use components::context_menu::{
    ContextMenu, ContextMenuCheckboxItem, ContextMenuContent, ContextMenuGroup, ContextMenuItem,
    ContextMenuItemVariant, ContextMenuLabel, ContextMenuRadioGroup, ContextMenuRadioItem,
    ContextMenuSeparator, ContextMenuShortcut, ContextMenuSub, ContextMenuSubContent,
    ContextMenuSubTrigger, ContextMenuTrigger,
};
pub use components::date_picker::DatePicker;
pub use components::dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    DialogTrigger,
};
pub use components::direction::{Direction, DirectionProvider, use_direction};
pub use components::drawer::{
    Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerTitle,
    DrawerTrigger,
};
pub use components::dropdown_menu::{
    DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuItem,
    DropdownMenuItemVariant, DropdownMenuLabel, DropdownMenuRadioGroup, DropdownMenuRadioItem,
    DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,
};
pub use components::empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
pub use components::field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldOrientation, FieldSeparator, FieldSet, FieldTitle,
};
pub use components::hover_card::{HoverCard, HoverCardContent, HoverCardTrigger};
pub use components::input::Input;
pub use components::input_group::{
    InputGroup, InputGroupAddon, InputGroupAddonAlign, InputGroupButton, InputGroupButtonSize,
    InputGroupInput, InputGroupText, InputGroupTextarea,
};
pub use components::input_otp::{InputOtp, InputOtpGroup, InputOtpSeparator, InputOtpSlot};
pub use components::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia,
    ItemMediaVariant, ItemSeparator, ItemSize, ItemTitle, ItemVariant,
};
pub use components::kbd::{Kbd, KbdGroup};
pub use components::label::Label;
pub use components::menubar::{
    Menubar, MenubarCheckboxItem, MenubarContent, MenubarItem, MenubarItemVariant, MenubarLabel,
    MenubarMenu, MenubarRadioGroup, MenubarRadioItem, MenubarSub, MenubarSubContent,
    MenubarSubTrigger, MenubarTrigger,
};
pub use components::native_select::{
    NativeSelect, NativeSelectOptGroup, NativeSelectOption, NativeSelectSize,
};
pub use components::navigation_menu::{
    NAVIGATION_MENU_TRIGGER_STYLE, NavigationMenu, NavigationMenuContent, NavigationMenuIndicator,
    NavigationMenuItem, NavigationMenuLink, NavigationMenuList, NavigationMenuTrigger,
    NavigationMenuViewport,
};
pub use components::pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink,
    PaginationNext, PaginationPrevious,
};
pub use components::popover::{
    Popover, PopoverContent, PopoverDescription, PopoverHeader, PopoverTitle, PopoverTrigger,
};
pub use components::progress::Progress;
pub use components::radio_group::{RadioGroup, RadioGroupItem};
pub use components::resizable::{ResizableHandle, ResizablePanel, ResizablePanelGroup};
pub use components::scroll_area::ScrollArea;
pub use components::select::{
    Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectTrigger,
    SelectValue,
};
pub use components::separator::{Separator, SeparatorOrientation};
pub use components::sheet::{
    Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide,
    SheetTitle, SheetTrigger,
};
pub use components::skeleton::Skeleton;
pub use components::slider::Slider;
pub use components::sonner::{Toast, ToastVariant, Toaster, ToasterContext, provide_toaster};
pub use components::spinner::Spinner;
pub use components::switch::{Switch, SwitchSize};
pub use components::table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
pub use components::tabs::{
    Tabs, TabsContent, TabsList, TabsListVariant, TabsOrientation, TabsTrigger,
};
pub use components::textarea::Textarea;
pub use components::toggle::{Toggle, ToggleSize, ToggleVariant};
pub use components::toggle_group::{ToggleGroup, ToggleGroupItem};
pub use components::tooltip::{
    Tooltip, TooltipContent, TooltipProvider, TooltipSide, TooltipTrigger,
};
pub use hooks::use_dismiss::use_dismiss;
pub use hooks::use_is_mobile::{MOBILE_BREAKPOINT, use_is_mobile};
pub use hooks::use_media_query::use_media_query;
pub use hooks::use_theme_mode::{ThemeMode, use_theme_mode};
