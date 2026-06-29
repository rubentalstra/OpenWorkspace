//! sidebar-07 — a faithful Leptos port of shadcn's `sidebar-07` block (the collapsible
//! icon sidebar with a team switcher, collapsible nav groups, a projects list with
//! per-item action menus, and a user menu). The upstream composes via Base UI's
//! `render`/asChild; here the same rendered output is produced with the kit's nested
//! components plus external `open` signals and Leptos attribute forwarding.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;
use ui::{
    Align, Avatar, AvatarFallback, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList,
    BreadcrumbPage, BreadcrumbSeparator, Collapsible, CollapsibleContent, DropdownMenu,
    DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuItemVariant,
    DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuShortcut, Separator,
    SeparatorOrientation, Side, Sidebar, SidebarCollapsible, SidebarContent, SidebarFooter,
    SidebarGroup, SidebarGroupLabel, SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuAction,
    SidebarMenuButton, SidebarMenuButtonSize, SidebarMenuItem, SidebarMenuSub,
    SidebarMenuSubButton, SidebarMenuSubItem, SidebarProvider, SidebarRail, SidebarTrigger,
    use_sidebar,
};

/// sidebar-07 page: provider → app sidebar + inset (header with trigger, separator,
/// breadcrumb; body with three cards and a tall panel).
#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <SidebarProvider>
            <AppSidebar />
            <SidebarInset>
                <header class="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-12">
                    <div class="flex items-center gap-2 px-4">
                        <SidebarTrigger class="-ml-1" />
                        <Separator
                            orientation=SeparatorOrientation::Vertical
                            class="mr-2 data-vertical:h-4 data-vertical:self-auto"
                        />
                        <Breadcrumb>
                            <BreadcrumbList>
                                <BreadcrumbItem class="hidden md:block">
                                    <BreadcrumbLink attr:href="#">
                                        "Build Your Application"
                                    </BreadcrumbLink>
                                </BreadcrumbItem>
                                <BreadcrumbSeparator class="hidden md:block" />
                                <BreadcrumbItem>
                                    <BreadcrumbPage>"Data Fetching"</BreadcrumbPage>
                                </BreadcrumbItem>
                            </BreadcrumbList>
                        </Breadcrumb>
                    </div>
                </header>
                <div class="flex flex-1 flex-col gap-4 p-4 pt-0">
                    <div class="grid auto-rows-min gap-4 md:grid-cols-3">
                        <div class="bg-muted/50 aspect-video rounded-xl"></div>
                        <div class="bg-muted/50 aspect-video rounded-xl"></div>
                        <div class="bg-muted/50 aspect-video rounded-xl"></div>
                    </div>
                    <div class="bg-muted/50 min-h-[100vh] flex-1 rounded-xl md:min-h-min"></div>
                </div>
            </SidebarInset>
        </SidebarProvider>
    }
}

/// The application sidebar: team switcher header, nav groups, user footer, rail.
#[component]
fn AppSidebar() -> impl IntoView {
    view! {
        <Sidebar collapsible=SidebarCollapsible::Icon>
            <SidebarHeader>
                <TeamSwitcher />
            </SidebarHeader>
            <SidebarContent>
                <NavMain />
                <NavProjects />
            </SidebarContent>
            <SidebarFooter>
                <NavUser />
            </SidebarFooter>
            <SidebarRail />
        </Sidebar>
    }
}

const TEAMS: &[(&str, &str, &str)] = &[
    ("Acme Inc", "gallery", "Enterprise"),
    ("Acme Corp.", "audio", "Startup"),
    ("Evil Corp.", "terminal", "Free"),
];

/// A logo glyph for a team, keyed by name (the upstream stores JSX per team).
fn team_logo(key: &str) -> impl IntoView {
    let icon = match key {
        "audio" => icondata::LuAudioLines,
        "terminal" => icondata::LuTerminal,
        _ => icondata::LuGalleryVerticalEnd,
    };
    view! { <Icon icon=icon attr:class="size-4" /> }
}

#[component]
fn TeamSwitcher() -> impl IntoView {
    let open = RwSignal::new(false);
    let active = RwSignal::new(0_usize);
    let active_team = move || TEAMS[active.get()];
    let ctx = use_sidebar();
    let side = Signal::derive(move || {
        if ctx.is_mobile() {
            Side::Bottom
        } else {
            Side::Right
        }
    });
    view! {
        <SidebarMenu>
            <SidebarMenuItem>
                <DropdownMenu open=open class="block w-full">
                    <SidebarMenuButton
                        size=SidebarMenuButtonSize::Lg
                        class=Signal::derive(move || {
                            if open.get() {
                                "bg-sidebar-accent text-sidebar-accent-foreground".to_owned()
                            } else {
                                String::new()
                            }
                        })
                        on_click=Callback::new(move |()| open.update(|value| *value = !*value))
                    >
                        <div class="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg">
                            {move || team_logo(active_team().1)}
                        </div>
                        <div class="grid flex-1 text-left text-sm leading-tight">
                            <span class="truncate font-medium">{move || active_team().0}</span>
                            <span class="truncate text-xs">{move || active_team().2}</span>
                        </div>
                        <Icon icon=icondata::LuChevronsUpDown attr:class="ml-auto" />
                    </SidebarMenuButton>
                    <DropdownMenuContent class="w-fit min-w-56" side=side align=Align::Start>
                        <DropdownMenuGroup>
                            <DropdownMenuLabel class="text-muted-foreground text-xs">
                                "Teams"
                            </DropdownMenuLabel>
                            {TEAMS
                                .iter()
                                .enumerate()
                                .map(|(index, &(name, key, _))| {
                                    let select = Callback::new(move |()| {
                                        active.set(index);
                                        open.set(false);
                                    });
                                    view! {
                                        <DropdownMenuItem on_select=select class="gap-2 p-2">
                                            <div class="flex size-6 items-center justify-center rounded-md border">
                                                {team_logo(key)}
                                            </div>
                                            {name}
                                            <DropdownMenuShortcut>
                                                {format!("⌘{}", index + 1)}
                                            </DropdownMenuShortcut>
                                        </DropdownMenuItem>
                                    }
                                })
                                .collect_view()}
                        </DropdownMenuGroup>
                        <DropdownMenuSeparator />
                        <DropdownMenuGroup>
                            <DropdownMenuItem class="gap-2 p-2">
                                <div class="flex size-6 items-center justify-center rounded-md border bg-transparent">
                                    <Icon icon=icondata::LuPlus attr:class="size-4" />
                                </div>
                                <div class="text-muted-foreground font-medium">"Add team"</div>
                            </DropdownMenuItem>
                        </DropdownMenuGroup>
                    </DropdownMenuContent>
                </DropdownMenu>
            </SidebarMenuItem>
        </SidebarMenu>
    }
}

const NAV_MAIN: &[(&str, &str, bool, &[&str])] = &[
    (
        "Playground",
        "terminal",
        true,
        &["History", "Starred", "Settings"],
    ),
    ("Models", "bot", false, &["Genesis", "Explorer", "Quantum"]),
    (
        "Documentation",
        "book",
        false,
        &["Introduction", "Get Started", "Tutorials", "Changelog"],
    ),
    (
        "Settings",
        "settings",
        false,
        &["General", "Team", "Billing", "Limits"],
    ),
];

fn nav_main_icon(key: &str) -> impl IntoView {
    let icon = match key {
        "bot" => icondata::LuBot,
        "book" => icondata::LuBookOpen,
        "settings" => icondata::LuSettings2,
        _ => icondata::LuSquareTerminal,
    };
    view! { <Icon icon=icon /> }
}

#[component]
fn NavMain() -> impl IntoView {
    view! {
        <SidebarGroup>
            <SidebarGroupLabel>"Platform"</SidebarGroupLabel>
            <SidebarMenu>
                {NAV_MAIN
                    .iter()
                    .map(|&(title, key, is_active, items)| {
                        let open = RwSignal::new(is_active);
                        view! {
                            <SidebarMenuItem>
                                <Collapsible open=open class="group/collapsible">
                                    <SidebarMenuButton
                                        tooltip=title.to_owned()
                                        on_click=Callback::new(move |()| {
                                            open.update(|value| *value = !*value);
                                        })
                                    >
                                        {nav_main_icon(key)}
                                        <span>{title}</span>
                                        <Icon
                                            icon=icondata::LuChevronRight
                                            attr:class="ml-auto transition-transform duration-200 group-data-open/collapsible:rotate-90"
                                        />
                                    </SidebarMenuButton>
                                    <CollapsibleContent>
                                        <SidebarMenuSub>
                                            {items
                                                .iter()
                                                .map(|&sub| {
                                                    view! {
                                                        <SidebarMenuSubItem>
                                                            <SidebarMenuSubButton href="#">
                                                                <span>{sub}</span>
                                                            </SidebarMenuSubButton>
                                                        </SidebarMenuSubItem>
                                                    }
                                                })
                                                .collect_view()}
                                        </SidebarMenuSub>
                                    </CollapsibleContent>
                                </Collapsible>
                            </SidebarMenuItem>
                        }
                    })
                    .collect_view()}
            </SidebarMenu>
        </SidebarGroup>
    }
}

const PROJECTS: &[(&str, &str)] = &[
    ("Design Engineering", "frame"),
    ("Sales & Marketing", "chart"),
    ("Travel", "map"),
];

fn project_icon(key: &str) -> impl IntoView {
    let icon = match key {
        "chart" => icondata::LuChartPie,
        "map" => icondata::LuMap,
        _ => icondata::LuFrame,
    };
    view! { <Icon icon=icon /> }
}

#[component]
fn NavProjects() -> impl IntoView {
    let ctx = use_sidebar();
    let side = Signal::derive(move || {
        if ctx.is_mobile() {
            Side::Bottom
        } else {
            Side::Right
        }
    });
    let align = Signal::derive(move || {
        if ctx.is_mobile() {
            Align::End
        } else {
            Align::Start
        }
    });
    view! {
        <SidebarGroup class="group-data-[collapsible=icon]:hidden">
            <SidebarGroupLabel>"Projects"</SidebarGroupLabel>
            <SidebarMenu>
                {PROJECTS
                    .iter()
                    .map(|&(name, key)| {
                        let open = RwSignal::new(false);
                        view! {
                            <SidebarMenuItem>
                                <SidebarMenuButton href="#">
                                    {project_icon(key)} <span>{name}</span>
                                </SidebarMenuButton>
                                <DropdownMenu open=open>
                                    <SidebarMenuAction
                                        show_on_hover=true
                                        class="aria-expanded:bg-muted"
                                        attr:aria-expanded=move || open.get().to_string()
                                        on_click=Callback::new(move |()| {
                                            open.update(|value| *value = !*value);
                                        })
                                    >
                                        <Icon icon=icondata::LuEllipsis />
                                        <span class="sr-only">"More"</span>
                                    </SidebarMenuAction>
                                    <DropdownMenuContent
                                        class="w-fit min-w-48"
                                        side=side
                                        align=align
                                    >
                                        <DropdownMenuItem>
                                            <Icon icon=icondata::LuFolder />
                                            <span>"View Project"</span>
                                        </DropdownMenuItem>
                                        <DropdownMenuItem>
                                            <Icon icon=icondata::LuArrowRight />
                                            <span>"Share Project"</span>
                                        </DropdownMenuItem>
                                        <DropdownMenuSeparator />
                                        <DropdownMenuItem variant=DropdownMenuItemVariant::Destructive>
                                            <Icon icon=icondata::LuTrash2 />
                                            <span>"Delete Project"</span>
                                        </DropdownMenuItem>
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            </SidebarMenuItem>
                        }
                    })
                    .collect_view()} <SidebarMenuItem>
                    <SidebarMenuButton class="text-sidebar-foreground/70">
                        <Icon icon=icondata::LuEllipsis attr:class="text-sidebar-foreground/70" />
                        <span>"More"</span>
                    </SidebarMenuButton>
                </SidebarMenuItem>
            </SidebarMenu>
        </SidebarGroup>
    }
}

#[component]
fn NavUser() -> impl IntoView {
    let open = RwSignal::new(false);
    let ctx = use_sidebar();
    let side = Signal::derive(move || {
        if ctx.is_mobile() {
            Side::Bottom
        } else {
            Side::Right
        }
    });
    let go = move |path: &str| {
        if let Some(window) = web_sys::window() {
            _ = window.location().set_href(path);
        }
    };
    let open_account = Callback::new(move |()| go("/account"));
    // Sign out, then follow the IdP's RP-initiated logout URL when present.
    let sign_out = Callback::new(move |()| {
        spawn_local(async move {
            let dest = crate::auth::logout()
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "/login".to_owned());
            if let Some(window) = web_sys::window() {
                _ = window.location().set_href(&dest);
            }
        });
    });
    view! {
        <SidebarMenu>
            <SidebarMenuItem>
                <DropdownMenu open=open class="block w-full">
                    <SidebarMenuButton
                        size=SidebarMenuButtonSize::Lg
                        class=Signal::derive(move || {
                            if open.get() { "bg-muted".to_owned() } else { String::new() }
                        })
                        on_click=Callback::new(move |()| open.update(|value| *value = !*value))
                    >
                        <Avatar>
                            <AvatarFallback>"CN"</AvatarFallback>
                        </Avatar>
                        <div class="grid flex-1 text-left text-sm leading-tight">
                            <span class="truncate font-medium">"shadcn"</span>
                            <span class="truncate text-xs">"m@example.com"</span>
                        </div>
                        <Icon icon=icondata::LuChevronsUpDown attr:class="ml-auto size-4" />
                    </SidebarMenuButton>
                    <DropdownMenuContent class="w-fit min-w-56" side=side align=Align::End>
                        <DropdownMenuGroup>
                            <DropdownMenuLabel class="p-0 font-normal">
                                <div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                                    <Avatar>
                                        <AvatarFallback>"CN"</AvatarFallback>
                                    </Avatar>
                                    <div class="grid flex-1 text-left text-sm leading-tight">
                                        <span class="truncate font-medium">"shadcn"</span>
                                        <span class="truncate text-xs">"m@example.com"</span>
                                    </div>
                                </div>
                            </DropdownMenuLabel>
                        </DropdownMenuGroup>
                        <DropdownMenuSeparator />
                        <DropdownMenuGroup>
                            <DropdownMenuItem>
                                <Icon icon=icondata::LuSparkles />
                                "Upgrade to Pro"
                            </DropdownMenuItem>
                        </DropdownMenuGroup>
                        <DropdownMenuSeparator />
                        <DropdownMenuGroup>
                            <DropdownMenuItem on_select=open_account>
                                <Icon icon=icondata::LuBadgeCheck />
                                "Account & security"
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
                        <DropdownMenuItem on_select=sign_out>
                            <Icon icon=icondata::LuLogOut />
                            "Log out"
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </SidebarMenuItem>
        </SidebarMenu>
    }
}
