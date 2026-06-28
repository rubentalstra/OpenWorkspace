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

pub use components::alert::{Alert, AlertAction, AlertDescription, AlertTitle, AlertVariant};
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
pub use components::card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardSize, CardTitle,
};
pub use components::checkbox::Checkbox;
pub use components::empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
pub use components::input::Input;
pub use components::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia,
    ItemMediaVariant, ItemSeparator, ItemSize, ItemTitle, ItemVariant,
};
pub use components::kbd::{Kbd, KbdGroup};
pub use components::label::Label;
pub use components::native_select::{
    NativeSelect, NativeSelectOptGroup, NativeSelectOption, NativeSelectSize,
};
pub use components::pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink,
    PaginationNext, PaginationPrevious,
};
pub use components::progress::Progress;
pub use components::separator::{Separator, SeparatorOrientation};
pub use components::skeleton::Skeleton;
pub use components::slider::Slider;
pub use components::spinner::Spinner;
pub use components::switch::{Switch, SwitchSize};
pub use components::table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
pub use components::textarea::Textarea;
pub use components::toggle::{Toggle, ToggleSize, ToggleVariant};
pub use hooks::use_is_mobile::{MOBILE_BREAKPOINT, use_is_mobile};
pub use hooks::use_media_query::use_media_query;
pub use hooks::use_theme_mode::{ThemeMode, use_theme_mode};
