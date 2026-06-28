use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_location;
use ui::{
    Avatar, AvatarFallback, AvatarSize, ButtonSize, ButtonVariant, Collapsible, CollapsibleContent,
    DropdownMenu, DropdownMenuAlign, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem,
    DropdownMenuItemVariant, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuSide,
    DropdownMenuTrigger, Separator, SeparatorOrientation, Sheet, SheetContent, SheetDirection,
    SheetTrigger, Sidenav, SidenavCollapsible, SidenavContent, SidenavFooter, SidenavGroup,
    SidenavGroupContent, SidenavGroupLabel, SidenavHeader, SidenavInset, SidenavMenu,
    SidenavMenuAction, SidenavMenuButton, SidenavMenuButtonSize, SidenavMenuItem, SidenavMenuSub,
    SidenavMenuSubButton, SidenavMenuSubItem, SidenavTrigger, SidenavWrapper, ThemeToggle, Toaster,
    ToasterContext,
};

/// One collapsible nav section: a title, a leading glyph, and its links.
struct NavSection {
    title: &'static str,
    icon: icondata::Icon,
    links: &'static [(&'static str, &'static str)],
}

const NAV_SECTIONS: &[NavSection] = &[
    NavSection {
        title: "Actions & inputs",
        icon: icondata::LuMousePointerClick,
        links: &[
            ("/ui/buttons", "Buttons & actions"),
            ("/ui/inputs", "Inputs"),
            ("/ui/forms", "Forms"),
        ],
    },
    NavSection {
        title: "Overlays & navigation",
        icon: icondata::LuLayers,
        links: &[("/ui/overlays", "Overlays"), ("/ui/navigation", "Navigation")],
    },
    NavSection {
        title: "Data & dates",
        icon: icondata::LuTable,
        links: &[("/ui/data", "Data"), ("/ui/dates", "Dates")],
    },
    NavSection {
        title: "Feedback & layout",
        icon: icondata::LuBell,
        links: &[("/ui/feedback", "Feedback"), ("/ui/layout", "Layout")],
    },
    NavSection {
        title: "Foundations",
        icon: icondata::LuPalette,
        links: &[("/ui/theme", "Theme"), ("/ui/hooks", "Hooks")],
    },
];

const PROJECTS: &[(&str, icondata::Icon)] = &[
    ("Design system", icondata::LuFrame),
    ("Booking app", icondata::LuChartPie),
    ("Floor plans", icondata::LuMap),
];

/// Faithful recreation of shadcn's `sidebar-07` (a sidebar that collapses to
/// icons) on our `ui` components: a team-switcher header, collapsible nav
/// groups, a projects list with per-item menus, and a footer account menu —
/// beside an inset content area. Collapses to an icon rail with tooltips; on
/// mobile the same nav is served through a [`Sheet`]. Mounts one [`Toaster`].
#[component]
pub fn ShowcaseLayout() -> impl IntoView {
    provide_context(ToasterContext::new());
    let path = use_location().pathname;

    view! {
        <SidenavWrapper
            attr:style="--sidenav-width: 16rem; --sidenav-width-icon: 3rem;"
            class="min-h-screen bg-sidenav"
        >
            <Sidenav collapsible=SidenavCollapsible::Icon>
                <NavBody active_path=path />
            </Sidenav>

            <SidenavInset>
                <header class="flex sticky top-0 z-20 gap-2 items-center px-4 h-14 border-b backdrop-blur border-sidenav-border bg-background/80">
                    <div class="hidden md:block">
                        <SidenavTrigger>
                            <Icon icon=icondata::LuPanelLeft attr:class="size-4" />
                        </SidenavTrigger>
                    </div>
                    <div class="md:hidden">
                        <Sheet>
                            <SheetTrigger
                                variant=ButtonVariant::Ghost
                                size=ButtonSize::Icon
                                class="size-8"
                            >
                                <Icon icon=icondata::LuPanelLeft attr:class="size-4" />
                            </SheetTrigger>
                            <SheetContent
                                direction=SheetDirection::Left
                                show_close_button=false
                                class="p-0 w-72 bg-sidenav text-sidenav-foreground"
                            >
                                <NavBody active_path=path />
                            </SheetContent>
                        </Sheet>
                    </div>
                    <Separator orientation=SeparatorOrientation::Vertical class="mr-1 h-4" />
                    <span class="text-sm font-medium text-foreground">"Component gallery"</span>
                    <div class="flex-1" />
                    <ThemeToggle />
                </header>

                <div class="overflow-auto flex-1">
                    <div class="px-6 py-10 mx-auto w-full max-w-6xl">
                        <Outlet />
                    </div>
                </div>
            </SidenavInset>

            <Toaster />
        </SidenavWrapper>
    }
}

/// Shared sidebar body used by the desktop [`Sidenav`] and the mobile [`Sheet`].
#[component]
fn NavBody(#[prop(into)] active_path: Signal<String>) -> impl IntoView {
    view! {
        <SidenavHeader>
            <TeamSwitcher />
        </SidenavHeader>
        <SidenavContent>
            <NavMain active_path=active_path />
            <NavProjects />
        </SidenavContent>
        <SidenavFooter>
            <NavUser />
        </SidenavFooter>
    }
}

/// Header brand/quick-jump menu (shadcn `TeamSwitcher`).
#[component]
fn TeamSwitcher() -> impl IntoView {
    view! {
        <SidenavMenu>
            <SidenavMenuItem>
                <DropdownMenu
                    align=DropdownMenuAlign::Start
                    side=DropdownMenuSide::Bottom
                    class="block w-full"
                >
                    <DropdownMenuTrigger as_child=true>
                        <SidenavMenuButton
                            size=SidenavMenuButtonSize::Lg
                            attr:title="OpenWorkspace UI"
                            class="data-[state=open]:bg-sidenav-accent"
                        >
                            <div class="flex justify-center items-center rounded-lg bg-primary text-primary-foreground aspect-square size-8">
                                <Icon icon=icondata::LuComponent attr:class="size-4" />
                            </div>
                            <div class="grid flex-1 text-sm leading-tight text-left">
                                <span class="font-semibold truncate">"OpenWorkspace UI"</span>
                                <span class="text-xs truncate">"Component gallery"</span>
                            </div>
                            <Icon icon=icondata::LuChevronsUpDown attr:class="ml-auto size-4" />
                        </SidenavMenuButton>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent class="min-w-56">
                        <DropdownMenuLabel>"OpenWorkspace"</DropdownMenuLabel>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem href="/ui">
                            <Icon icon=icondata::LuLayoutGrid />
                            "Overview"
                        </DropdownMenuItem>
                        <DropdownMenuItem href="/">
                            <Icon icon=icondata::LuHouse />
                            "Home"
                        </DropdownMenuItem>
                        <DropdownMenuItem href="https://github.com/rust-ui/ui">
                            <Icon icon=icondata::LuBookOpen />
                            "Reference UI"
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </SidenavMenuItem>
        </SidenavMenu>
    }
}

/// Collapsible nav groups (shadcn `NavMain`). Built on [`Collapsible`] +
/// [`SidenavMenuButton`], so each group shrinks to an icon (with a `title`
/// tooltip) when the sidebar collapses.
#[component]
fn NavMain(#[prop(into)] active_path: Signal<String>) -> impl IntoView {
    let groups = NAV_SECTIONS
        .iter()
        .map(|section| view! { <NavGroup section=section active_path=active_path /> })
        .collect_view();

    view! {
        <SidenavGroup>
            <SidenavGroupLabel>"Platform"</SidenavGroupLabel>
            <SidenavMenu>
                <SidenavMenuItem>
                    <SidenavMenuButton href="/ui".to_string() tooltip="Overview">
                        <Icon icon=icondata::LuLayoutGrid />
                        <span>"Overview"</span>
                    </SidenavMenuButton>
                </SidenavMenuItem>
                {groups}
            </SidenavMenu>
        </SidenavGroup>
    }
}

/// One collapsible group with a chevron that rotates on open.
#[component]
fn NavGroup(section: &'static NavSection, #[prop(into)] active_path: Signal<String>) -> impl IntoView {
    let open = RwSignal::new(
        section
            .links
            .iter()
            .any(|(href, _)| active_path.get_untracked() == *href),
    );
    let links = section
        .links
        .iter()
        .map(|(href, label)| {
            view! {
                <SidenavMenuSubItem>
                    <SidenavMenuSubButton href=*href>{*label}</SidenavMenuSubButton>
                </SidenavMenuSubItem>
            }
        })
        .collect_view();

    view! {
        <SidenavMenuItem>
            <Collapsible open=open class="group/collapsible">
                <SidenavMenuButton
                    tooltip=section.title
                    on:click=move |_| open.update(|o| *o = !*o)
                >
                    <Icon icon=section.icon />
                    <span>{section.title}</span>
                    <Icon
                        icon=icondata::LuChevronRight
                        attr:class="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90"
                    />
                </SidenavMenuButton>
                <CollapsibleContent>
                    <SidenavMenuSub>{links}</SidenavMenuSub>
                </CollapsibleContent>
            </Collapsible>
        </SidenavMenuItem>
    }
}

/// Projects list with a per-item action menu (shadcn `NavProjects`). Hidden when
/// the sidebar is collapsed to icons.
#[component]
fn NavProjects() -> impl IntoView {
    let items = PROJECTS
        .iter()
        .map(|(name, icon)| {
            view! {
                <SidenavMenuItem>
                    <SidenavMenuButton href="#".to_string()>
                        <Icon icon=*icon />
                        <span>{*name}</span>
                    </SidenavMenuButton>
                    <DropdownMenu
                        align=DropdownMenuAlign::Start
                        side=DropdownMenuSide::Bottom
                        class="contents"
                    >
                        <DropdownMenuTrigger as_child=true>
                            <SidenavMenuAction show_on_hover=true>
                                <Icon icon=icondata::LuEllipsis />
                            </SidenavMenuAction>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent class="w-48">
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuFolder />
                                "View project"
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuForward />
                                "Share project"
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem variant=DropdownMenuItemVariant::Destructive>
                                <Icon icon=icondata::LuTrash2 />
                                "Delete project"
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                </SidenavMenuItem>
            }
        })
        .collect_view();

    view! {
        <SidenavGroup class="group-data-[collapsible=Icon]:hidden">
            <SidenavGroupLabel>"Projects"</SidenavGroupLabel>
            <SidenavGroupContent>
                <SidenavMenu>{items}</SidenavMenu>
            </SidenavGroupContent>
        </SidenavGroup>
    }
}

/// Footer account menu (shadcn `NavUser`); opens upward.
#[component]
fn NavUser() -> impl IntoView {
    view! {
        <SidenavMenu>
            <SidenavMenuItem>
                <DropdownMenu
                    align=DropdownMenuAlign::Start
                    side=DropdownMenuSide::Top
                    class="block w-full"
                >
                    <DropdownMenuTrigger as_child=true>
                        <SidenavMenuButton
                            size=SidenavMenuButtonSize::Lg
                            attr:title="OpenWorkspace"
                            class="data-[state=open]:bg-sidenav-accent"
                        >
                            <Avatar size=AvatarSize::Sm>
                                <AvatarFallback>"OW"</AvatarFallback>
                            </Avatar>
                            <div class="grid flex-1 text-sm leading-tight text-left">
                                <span class="font-semibold truncate">"OpenWorkspace"</span>
                                <span class="text-xs truncate">"dev@openworkspace.dev"</span>
                            </div>
                            <Icon icon=icondata::LuChevronsUpDown attr:class="ml-auto size-4" />
                        </SidenavMenuButton>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent class="min-w-56">
                        <DropdownMenuLabel>"dev@openworkspace.dev"</DropdownMenuLabel>
                        <DropdownMenuSeparator />
                        <DropdownMenuGroup>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuSparkles />
                                "Upgrade to Pro"
                            </DropdownMenuItem>
                        </DropdownMenuGroup>
                        <DropdownMenuSeparator />
                        <DropdownMenuGroup>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuBadgeCheck />
                                "Account"
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuCreditCard />
                                "Billing"
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuBell />
                                "Notifications"
                            </DropdownMenuItem>
                        </DropdownMenuGroup>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem>
                            <Icon icon=icondata::LuLogOut />
                            "Log out"
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </SidenavMenuItem>
        </SidenavMenu>
    }
}
