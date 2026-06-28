use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_location;
use ui::{
    AccordionContent, AccordionHeader, AccordionItem, AccordionTitle, AccordionTrigger, Avatar,
    AvatarFallback, AvatarSize, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign,
    DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel,
    DropdownMenuSeparator, DropdownMenuSide, DropdownMenuSub, DropdownMenuSubContent,
    DropdownMenuSubTrigger, DropdownMenuTrigger, Sheet, SheetContent,
    SheetDirection, SheetTrigger, Sidenav, SidenavContent, SidenavFooter, SidenavGroup,
    SidenavGroupContent, SidenavGroupLabel, SidenavHeader, SidenavLink, SidenavMenu,
    SidenavMenuItem, SidenavMenuSub, SidenavTrigger, SidenavWrapper, ThemeToggle, Toaster,
    ToasterContext,
};

/// One nav section: a title, a leading glyph, and its category links.
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

/// Persistent gallery chrome modelled on the rust-ui `sidenav07` block: a brand
/// selector header, accordion-grouped category menus, and a footer account menu,
/// beside an [`Outlet`] content area. The desktop [`Sidenav`] collapses via the
/// header rail; on mobile the same nav is served through a [`Sheet`]. One
/// [`Toaster`] is mounted so any page below can raise toasts.
#[component]
pub fn ShowcaseLayout() -> impl IntoView {
    // Provided above the outlet *and* the toaster so any page can raise toasts.
    provide_context(ToasterContext::new());
    let path = use_location().pathname;

    view! {
        <SidenavWrapper attr:style="--sidenav-width: 16rem;" class="min-h-screen bg-sidenav">
            <Sidenav>
                <NavBody active_path=path />
            </Sidenav>

            <main class="flex flex-col flex-1 min-w-0 bg-background">
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
                    <div class="flex-1 text-sm font-medium text-muted-foreground">
                        "Component gallery"
                    </div>
                    <ThemeToggle />
                </header>

                <div class="overflow-auto flex-1">
                    <div class="px-6 py-10 mx-auto w-full max-w-6xl">
                        <Outlet />
                    </div>
                </div>
            </main>

            <Toaster />
        </SidenavWrapper>
    }
}

/// Shared sidenav body — header selector, accordion-grouped menus, footer account
/// menu — rendered by both the desktop [`Sidenav`] and the mobile [`Sheet`].
#[component]
fn NavBody(#[prop(into)] active_path: Signal<String>) -> impl IntoView {
    let sections = NAV_SECTIONS
        .iter()
        .map(|section| {
            view! { <NavGroup section=section active_path=active_path /> }
        })
        .collect_view();

    view! {
        <SidenavHeader class="border-b border-sidenav-border">
            <BrandSelector />
        </SidenavHeader>

        <SidenavContent>
            <SidenavGroup>
                <SidenavMenu>
                    <SidenavMenuItem>
                        <SidenavLink href="/ui">
                            <Icon icon=icondata::LuLayoutGrid attr:class="size-4 shrink-0" />
                            <span>"Overview"</span>
                        </SidenavLink>
                    </SidenavMenuItem>
                </SidenavMenu>
            </SidenavGroup>

            <SidenavGroup>
                <SidenavGroupLabel>"Components"</SidenavGroupLabel>
                <SidenavGroupContent>
                    <SidenavMenu>{sections}</SidenavMenu>
                </SidenavGroupContent>
            </SidenavGroup>
        </SidenavContent>

        <SidenavFooter class="border-t border-sidenav-border">
            <UserMenu />
        </SidenavFooter>
    }
}

/// One accordion-grouped nav section. The group opens by default when the active
/// route is one of its links.
#[component]
fn NavGroup(section: &'static NavSection, #[prop(into)] active_path: Signal<String>) -> impl IntoView {
    let open = {
        let here = active_path.get_untracked();
        section.links.iter().any(|(href, _)| here == *href)
    };
    let links = section
        .links
        .iter()
        .map(|(href, label)| view! { <SidenavLink href=*href>{*label}</SidenavLink> })
        .collect_view();

    view! {
        <SidenavMenuItem>
            <AccordionItem default_open=open>
                <AccordionTrigger class="p-2 rounded-md hover:bg-sidenav-accent hover:text-sidenav-accent-foreground">
                    <AccordionHeader>
                        <Icon icon=section.icon attr:class="size-4" />
                        <AccordionTitle>{section.title}</AccordionTitle>
                    </AccordionHeader>
                </AccordionTrigger>
                <AccordionContent class="p-0">
                    <SidenavMenuSub>{links}</SidenavMenuSub>
                </AccordionContent>
            </AccordionItem>
        </SidenavMenuItem>
    }
}

/// Brand block in the header that doubles as a quick-jump menu.
#[component]
fn BrandSelector() -> impl IntoView {
    view! {
        <DropdownMenu align=DropdownMenuAlign::Start class="block w-full">
            <DropdownMenuTrigger class="flex gap-2 justify-between items-center px-2 w-full h-12 bg-transparent border-0 hover:bg-sidenav-accent">
                <div class="flex gap-2 items-center min-w-0">
                    <div class="flex justify-center items-center rounded-lg bg-primary text-primary-foreground size-8 shrink-0">
                        <Icon icon=icondata::LuComponent attr:class="size-4" />
                    </div>
                    <div class="grid flex-1 text-sm leading-tight text-left">
                        <span class="font-semibold truncate">"OpenWorkspace UI"</span>
                        <span class="text-xs truncate text-muted-foreground">
                            "Component gallery"
                        </span>
                    </div>
                </div>
                <Icon icon=icondata::LuChevronsUpDown attr:class="opacity-60 size-4 shrink-0" />
            </DropdownMenuTrigger>
            <DropdownMenuContent class="w-56">
                <DropdownMenuGroup>
                    <DropdownMenuItem href="/ui">
                        <Icon icon=icondata::LuLayoutGrid />
                        "Overview"
                    </DropdownMenuItem>
                    <DropdownMenuItem href="/">
                        <Icon icon=icondata::LuHouse />
                        "Home"
                    </DropdownMenuItem>
                </DropdownMenuGroup>
            </DropdownMenuContent>
        </DropdownMenu>
    }
}

/// Account menu pinned to the sidenav footer; opens upward.
#[component]
fn UserMenu() -> impl IntoView {
    view! {
        <DropdownMenu align=DropdownMenuAlign::Start side=DropdownMenuSide::Top class="block w-full">
            <DropdownMenuTrigger class="flex gap-2 justify-between items-center px-2 w-full h-12 bg-transparent border-0 hover:bg-sidenav-accent">
                <div class="flex gap-2 items-center min-w-0">
                    <Avatar size=AvatarSize::Sm>
                        <AvatarFallback>"OW"</AvatarFallback>
                    </Avatar>
                    <div class="grid flex-1 text-sm leading-tight text-left">
                        <span class="font-medium truncate">"OpenWorkspace"</span>
                        <span class="text-xs truncate text-muted-foreground">
                            "dev@openworkspace.dev"
                        </span>
                    </div>
                </div>
                <Icon icon=icondata::LuChevronsUpDown attr:class="opacity-60 size-4 shrink-0" />
            </DropdownMenuTrigger>
            <DropdownMenuContent class="min-w-56">
                <DropdownMenuLabel>"Main menu"</DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuGroup>
                    <DropdownMenuItem href="/ui">
                        <Icon icon=icondata::LuLayoutGrid />
                        "Overview"
                    </DropdownMenuItem>
                    <DropdownMenuSub>
                        <DropdownMenuSubTrigger>
                            <Icon icon=icondata::LuSettings />
                            "Settings"
                        </DropdownMenuSubTrigger>
                        <DropdownMenuSubContent>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuUser />
                                "Account"
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuShield />
                                "Privacy"
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuBell />
                                "Notifications"
                            </DropdownMenuItem>
                        </DropdownMenuSubContent>
                    </DropdownMenuSub>
                    <DropdownMenuSub>
                        <DropdownMenuSubTrigger>
                            <Icon icon=icondata::LuWrench />
                            "Tools"
                        </DropdownMenuSubTrigger>
                        <DropdownMenuSubContent>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuDownload />
                                "Export data"
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuUpload />
                                "Import data"
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuTerminal />
                                "Developer tools"
                            </DropdownMenuItem>
                        </DropdownMenuSubContent>
                    </DropdownMenuSub>
                </DropdownMenuGroup>
                <DropdownMenuSeparator />
                <DropdownMenuGroup>
                    <DropdownMenuItem href="https://github.com/rust-ui/ui">
                        <Icon icon=icondata::LuBookOpen />
                        "Reference UI"
                    </DropdownMenuItem>
                    <DropdownMenuItem href="/">
                        <Icon icon=icondata::LuHouse />
                        "Home"
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                        <Icon icon=icondata::LuLogOut />
                        "Sign out"
                    </DropdownMenuItem>
                </DropdownMenuGroup>
            </DropdownMenuContent>
        </DropdownMenu>
    }
}
