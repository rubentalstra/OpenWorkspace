use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{
    ActionBar, ActionBarButton, BottomNav, BottomNavButton, BottomNavGrid, BottomNavLabel,
    Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage,
    BreadcrumbSeparator, Footer, FooterBrand, FooterContainer, FooterCopyright, FooterDescription,
    FooterLink, FooterLinks, FooterLinksSection, FooterNavContainer, FooterSection,
    FooterSectionsGrid, FooterTitle, Header, Link, LinkMatch, Menubar, MenubarCheckboxItem,
    MenubarContent, MenubarItem, MenubarLabel, MenubarMenu, MenubarRadioGroup, MenubarRadioItem,
    MenubarSeparator, MenubarShortcut, MenubarSub, MenubarSubContent, MenubarSubItem,
    MenubarSubTrigger, MenubarTrigger, NavMenu, NavMenuContent, NavMenuContentInset, NavMenuFixed,
    NavMenuHomeLink, NavMenuItem, NavMenuLink, NavMenuLinkDescription, NavMenuLinkGrid,
    NavMenuLinkTitle, NavMenuList, NavMenuTitle, NavMenuTrigger, NavMenuWrapper, NavigationMenu,
    NavigationMenuContent, NavigationMenuItem, NavigationMenuLink, NavigationMenuList,
    NavigationMenuTrigger, Pagination, PaginationContent, PaginationEllipsis, PaginationItem,
    PaginationLink, PaginationLinkSize, PaginationNext, PaginationPrevious, Sidenav,
    SidenavCollapsible, SidenavContent, SidenavFooter, SidenavGroup, SidenavGroupContent,
    SidenavGroupLabel, SidenavHeader, SidenavMenu, SidenavMenuButton, SidenavMenuButtonSize,
    SidenavMenuItem, Tabs, TabsContent, TabsList, TabsOrientation, TabsTrigger, TabsVariant,
    use_pagination,
};

use super::{Demo, Page, Section};

/// Tabs, breadcrumbs, pagination, menubars, nav bars and the route-aware link.
#[component]
pub fn NavigationPage() -> impl IntoView {
    view! {
        <Page title="Navigation" subtitle="Tabs, breadcrumbs, pagination, menubars and nav bars.">
            <TabsSection />
            <BreadcrumbSection />
            <PaginationSection />
            <LinkSection />
            <MenubarSection />
            <NavigationMenuSection />
            <ActionBarSection />
            <SidenavSection />
            <HeaderSection />
            <BottomNavSection />
            <FooterSection_ />
        </Page>
    }
}

#[component]
fn TabsSection() -> impl IntoView {
    view! {
        <Section
            title="Tabs"
            description="The selected value is seeded from default_value and held internally; triggers and panels match by value. Pill (default) and underline (Line) chrome, horizontal and vertical orientation."
        >
            <Demo label="Pill (default)" col=true>
                <Tabs default_value="overview" class="w-full max-w-md">
                    <TabsList>
                        <TabsTrigger value="overview">"Overview"</TabsTrigger>
                        <TabsTrigger value="bookings">"Bookings"</TabsTrigger>
                        <TabsTrigger value="settings">"Settings"</TabsTrigger>
                    </TabsList>
                    <TabsContent value="overview">
                        <p class="text-muted-foreground">"Site occupancy at a glance."</p>
                    </TabsContent>
                    <TabsContent value="bookings">
                        <p class="text-muted-foreground">"Upcoming desk reservations."</p>
                    </TabsContent>
                    <TabsContent value="settings">
                        <p class="text-muted-foreground">"Workspace preferences."</p>
                    </TabsContent>
                </Tabs>
            </Demo>
            <Demo label="Underline (Line)" col=true>
                <Tabs default_value="desks" class="w-full max-w-md">
                    <TabsList variant=TabsVariant::Line>
                        <TabsTrigger value="desks">"Desks"</TabsTrigger>
                        <TabsTrigger value="rooms">"Rooms"</TabsTrigger>
                        <TabsTrigger value="parking">"Parking"</TabsTrigger>
                    </TabsList>
                    <TabsContent value="desks">
                        <p class="text-muted-foreground">"Hot desks and assigned seats."</p>
                    </TabsContent>
                    <TabsContent value="rooms">
                        <p class="text-muted-foreground">"Meeting and focus rooms."</p>
                    </TabsContent>
                    <TabsContent value="parking">
                        <p class="text-muted-foreground">"Parking bays per site."</p>
                    </TabsContent>
                </Tabs>
            </Demo>
            <Demo label="Vertical orientation" col=true>
                <Tabs
                    default_value="profile"
                    orientation=TabsOrientation::Vertical
                    class="w-full max-w-md"
                >
                    <TabsList>
                        <TabsTrigger value="profile">"Profile"</TabsTrigger>
                        <TabsTrigger value="security">"Security"</TabsTrigger>
                        <TabsTrigger value="billing">"Billing"</TabsTrigger>
                    </TabsList>
                    <TabsContent value="profile">
                        <p class="text-muted-foreground">"Name, avatar and contact details."</p>
                    </TabsContent>
                    <TabsContent value="security">
                        <p class="text-muted-foreground">"Password and multi-factor auth."</p>
                    </TabsContent>
                    <TabsContent value="billing">
                        <p class="text-muted-foreground">"Plan, seats and invoices."</p>
                    </TabsContent>
                </Tabs>
            </Demo>
        </Section>
    }
}

#[component]
fn BreadcrumbSection() -> impl IntoView {
    view! {
        <Section
            title="Breadcrumbs"
            description="A composed trail with links, a chevron separator, a collapsed ellipsis and the current page."
        >
            <Demo label="Full trail">
                <Breadcrumb>
                    <BreadcrumbList>
                        <BreadcrumbItem>
                            <BreadcrumbLink attr:href="#">"Home"</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbLink attr:href="#">"Sites"</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbLink attr:href="#">"Amsterdam"</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbPage>"Floor 3"</BreadcrumbPage>
                        </BreadcrumbItem>
                    </BreadcrumbList>
                </Breadcrumb>
            </Demo>
            <Demo label="Collapsed with ellipsis">
                <Breadcrumb>
                    <BreadcrumbList>
                        <BreadcrumbItem>
                            <BreadcrumbLink attr:href="#">"Home"</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbEllipsis />
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbPage>"Booking #4821"</BreadcrumbPage>
                        </BreadcrumbItem>
                    </BreadcrumbList>
                </Breadcrumb>
            </Demo>
        </Section>
    }
}

#[component]
fn PaginationSection() -> impl IntoView {
    let pager = use_pagination();
    let current = pager.current_page;
    let prev_href = pager.prev_href;
    let next_href = pager.next_href;
    let is_first = pager.is_first_page;
    let page_href = pager.page_href;

    let pages = [1_u32, 2, 3];
    let links = pages
        .into_iter()
        .map(move |page| {
            let is_active = Signal::derive(move || current.get() == page);
            let href = Signal::derive(move || page_href.run(page));
            view! {
                <PaginationItem>
                    <PaginationLink is_active=is_active attr:href=href>
                        {page.to_string()}
                    </PaginationLink>
                </PaginationItem>
            }
        })
        .collect_view();

    view! {
        <Section
            title="Pagination"
            description="Driven by use_pagination, which reads the current page from the ?page= query parameter and builds hrefs that change only that parameter. Append ?page=2 to the URL to see the active page move."
        >
            <Demo col=true>
                <Pagination>
                    <PaginationContent>
                        <PaginationItem>
                            <PaginationPrevious
                                attr:href=prev_href
                                attr:aria-disabled=move || is_first.get().to_string()
                            />
                        </PaginationItem>
                        {links}
                        <PaginationItem>
                            <PaginationEllipsis />
                        </PaginationItem>
                        <PaginationItem>
                            <PaginationLink
                                size=PaginationLinkSize::Icon
                                attr:href=move || page_href.run(24)
                            >
                                "24"
                            </PaginationLink>
                        </PaginationItem>
                        <PaginationItem>
                            <PaginationNext attr:href=next_href />
                        </PaginationItem>
                    </PaginationContent>
                </Pagination>
                <p class="text-sm text-center text-muted-foreground">
                    {move || format!("Current page: {}", current.get())}
                </p>
            </Demo>
        </Section>
    }
}

#[component]
fn LinkSection() -> impl IntoView {
    view! {
        <Section
            title="Route-aware link"
            description="Link performs client-side navigation and highlights itself when the live route matches href per its match strategy. The router sets aria-current=page on the active anchor."
        >
            <Demo>
                <Link href="/ui/navigation" match_type=LinkMatch::Exact>
                    "Exact (this page)"
                </Link>
                <Link href="/ui" match_type=LinkMatch::Prefix>
                    "Prefix (/ui section)"
                </Link>
                <Link href="navigation" match_type=LinkMatch::Contains>
                    "Contains (navigation)"
                </Link>
                <Link href="/ui/buttons">"Buttons page"</Link>
            </Demo>
        </Section>
    }
}

#[component]
fn MenubarSection() -> impl IntoView {
    let word_wrap = RwSignal::new(true);
    let minimap = RwSignal::new(false);
    let layout = RwSignal::new("comfortable".to_string());

    view! {
        <Section
            title="Menubar"
            description="A horizontal bar of menus with keyboard navigation: arrow keys move between menus and items, Escape closes. Includes items with shortcuts, checkbox and radio items, a separator and a nested submenu."
        >
            <Demo>
                <Menubar>
                    <MenubarMenu>
                        <MenubarTrigger>"File"</MenubarTrigger>
                        <MenubarContent>
                            <MenubarItem>
                                "New booking" <MenubarShortcut>"\u{2318}N"</MenubarShortcut>
                            </MenubarItem>
                            <MenubarItem>
                                "Open" <MenubarShortcut>"\u{2318}O"</MenubarShortcut>
                            </MenubarItem>
                            <MenubarSeparator />
                            <MenubarSub>
                                <MenubarSubTrigger>"Export as"</MenubarSubTrigger>
                                <MenubarSubContent>
                                    <MenubarSubItem>"CSV"</MenubarSubItem>
                                    <MenubarSubItem>"PDF"</MenubarSubItem>
                                    <MenubarSubItem>"iCal"</MenubarSubItem>
                                </MenubarSubContent>
                            </MenubarSub>
                            <MenubarSeparator />
                            <MenubarItem>
                                "Quit" <MenubarShortcut>"\u{2318}Q"</MenubarShortcut>
                            </MenubarItem>
                        </MenubarContent>
                    </MenubarMenu>
                    <MenubarMenu>
                        <MenubarTrigger>"View"</MenubarTrigger>
                        <MenubarContent>
                            <MenubarLabel>"Appearance"</MenubarLabel>
                            <MenubarCheckboxItem checked=word_wrap>"Word wrap"</MenubarCheckboxItem>
                            <MenubarCheckboxItem checked=minimap>"Minimap"</MenubarCheckboxItem>
                            <MenubarSeparator />
                            <MenubarRadioGroup value=layout>
                                <MenubarRadioItem value="compact"
                                    .to_string()>"Compact"</MenubarRadioItem>
                                <MenubarRadioItem value="comfortable"
                                    .to_string()>"Comfortable"</MenubarRadioItem>
                            </MenubarRadioGroup>
                        </MenubarContent>
                    </MenubarMenu>
                    <MenubarMenu>
                        <MenubarTrigger>"Help"</MenubarTrigger>
                        <MenubarContent>
                            <MenubarItem href="#".to_string()>"Documentation"</MenubarItem>
                            <MenubarItem href="#".to_string()>"Keyboard shortcuts"</MenubarItem>
                        </MenubarContent>
                    </MenubarMenu>
                </Menubar>
            </Demo>
        </Section>
    }
}

#[component]
fn NavigationMenuSection() -> impl IntoView {
    view! {
        <Section
            title="Navigation menu"
            description="A hover/focus dropdown bar. Panels are absolutely positioned under the bar; hover a trigger to reveal its links."
        >
            <Demo>
                <div class="overflow-visible relative pb-44 w-full">
                    <NavigationMenu>
                        <NavigationMenuList>
                            <NavigationMenuItem>
                                <NavigationMenuTrigger>"Products"</NavigationMenuTrigger>
                                <NavigationMenuContent class="w-72">
                                    <ul class="flex flex-col gap-2">
                                        <li>
                                            <NavigationMenuLink
                                                href="#"
                                                class="block p-2 rounded-md hover:bg-accent"
                                            >
                                                <div class="font-medium text-foreground">"Desks"</div>
                                                <p class="text-xs text-muted-foreground">
                                                    "Book hot desks and assigned seats."
                                                </p>
                                            </NavigationMenuLink>
                                        </li>
                                        <li>
                                            <NavigationMenuLink
                                                href="#"
                                                class="block p-2 rounded-md hover:bg-accent"
                                            >
                                                <div class="font-medium text-foreground">"Rooms"</div>
                                                <p class="text-xs text-muted-foreground">
                                                    "Reserve meeting and focus rooms."
                                                </p>
                                            </NavigationMenuLink>
                                        </li>
                                    </ul>
                                </NavigationMenuContent>
                            </NavigationMenuItem>
                            <NavigationMenuItem>
                                <NavigationMenuTrigger>"Resources"</NavigationMenuTrigger>
                                <NavigationMenuContent class="w-56">
                                    <ul class="flex flex-col gap-1">
                                        <li>
                                            <NavigationMenuLink
                                                href="#"
                                                class="block p-2 rounded-md hover:bg-accent"
                                            >
                                                "Documentation"
                                            </NavigationMenuLink>
                                        </li>
                                        <li>
                                            <NavigationMenuLink
                                                href="#"
                                                class="block p-2 rounded-md hover:bg-accent"
                                            >
                                                "Changelog"
                                            </NavigationMenuLink>
                                        </li>
                                    </ul>
                                </NavigationMenuContent>
                            </NavigationMenuItem>
                        </NavigationMenuList>
                    </NavigationMenu>
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn ActionBarSection() -> impl IntoView {
    view! {
        <Section
            title="Action bar"
            description="A floating contextual toolbar with WAI-ARIA roving tabindex: the bar is one tab stop and arrow keys move between buttons. Exactly one button reads as selected."
        >
            <Demo>
                <ActionBar default_value="add">
                    <ActionBarButton value="add">
                        <Icon icon=icondata::LuPlus attr:class="size-4" />
                        "Add"
                    </ActionBarButton>
                    <ActionBarButton value="copy">
                        <Icon icon=icondata::LuCopy attr:class="size-4" />
                        "Copy"
                    </ActionBarButton>
                    <ActionBarButton value="settings">
                        <Icon icon=icondata::LuSettings attr:class="size-4" />
                        "Settings"
                    </ActionBarButton>
                    <ActionBarButton value="delete">
                        <Icon icon=icondata::LuTrash2 attr:class="size-4" />
                        "Delete"
                    </ActionBarButton>
                </ActionBar>
            </Demo>
        </Section>
    }
}

#[component]
fn SidenavSection() -> impl IntoView {
    view! {
        <Section
            title="Sidenav"
            description="The collapsible app sidenav. Shown here with collapsible=None so it renders as a static, contained aside; in the app shell it is a fixed-position, icon-collapsible rail. The whole gallery is wrapped in one already."
        >
            <Demo>
                <div class="overflow-hidden w-64 rounded-lg border h-80 border-border bg-sidenav text-sidenav-foreground">
                    <Sidenav collapsible=SidenavCollapsible::None class="w-full">
                        <SidenavHeader>
                            <div class="flex gap-2 items-center px-1 h-10">
                                <Icon
                                    icon=icondata::LuComponent
                                    attr:class="size-5 shrink-0 text-primary"
                                />
                                <span class="text-sm font-semibold">"Workspace"</span>
                            </div>
                        </SidenavHeader>
                        <SidenavContent>
                            <SidenavGroup>
                                <SidenavGroupLabel>"Manage"</SidenavGroupLabel>
                                <SidenavGroupContent>
                                    <SidenavMenu>
                                        <SidenavMenuItem>
                                            <SidenavMenuButton>
                                                <Icon
                                                    icon=icondata::LuLayoutDashboard
                                                    attr:class="size-4"
                                                />
                                                <span>"Dashboard"</span>
                                            </SidenavMenuButton>
                                        </SidenavMenuItem>
                                        <SidenavMenuItem>
                                            <SidenavMenuButton href="#".to_string()>
                                                <Icon icon=icondata::LuCalendar attr:class="size-4" />
                                                <span>"Bookings"</span>
                                            </SidenavMenuButton>
                                        </SidenavMenuItem>
                                        <SidenavMenuItem>
                                            <SidenavMenuButton size=SidenavMenuButtonSize::Sm>
                                                <Icon icon=icondata::LuTable attr:class="size-4" />
                                                <span>"Desks (small)"</span>
                                            </SidenavMenuButton>
                                        </SidenavMenuItem>
                                    </SidenavMenu>
                                </SidenavGroupContent>
                            </SidenavGroup>
                        </SidenavContent>
                        <SidenavFooter>
                            <SidenavMenu>
                                <SidenavMenuItem>
                                    <SidenavMenuButton>
                                        <Icon icon=icondata::LuSettings attr:class="size-4" />
                                        <span>"Settings"</span>
                                    </SidenavMenuButton>
                                </SidenavMenuItem>
                            </SidenavMenu>
                        </SidenavFooter>
                    </Sidenav>
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn HeaderSection() -> impl IntoView {
    view! {
        <Section
            title="Header / nav bar"
            description="The marketing-style top bar. NavMenuFixed is fixed to the viewport, so it is rendered inside a transformed preview frame (a transform ancestor contains fixed children) and floats once the page scrolls past the threshold."
        >
            <Demo>
                <div class="relative w-full h-40 rounded-lg border transform-gpu overflow-hidden border-border bg-muted/30">
                    <Header>
                        <NavMenuFixed class="absolute! pt-3!">
                            <NavMenuWrapper class="flex justify-between items-center py-2">
                                <NavMenuHomeLink>
                                    <Icon
                                        icon=icondata::LuComponent
                                        attr:class="size-5 text-primary"
                                    />
                                    <span class="text-sm font-semibold">"OpenWorkspace"</span>
                                </NavMenuHomeLink>
                                <NavMenu>
                                    <NavMenuList>
                                        <NavMenuItem>
                                            <NavMenuTrigger>"Product"</NavMenuTrigger>
                                            <NavMenuContent>
                                                <NavMenuContentInset class="w-56">
                                                    <div class="p-1">
                                                        <NavMenuTitle>"Features"</NavMenuTitle>
                                                        <NavMenuLinkGrid attr:href="#">
                                                            <Icon icon=icondata::LuCalendar attr:class="size-4" />
                                                            <div>
                                                                <NavMenuLinkTitle>"Bookings"</NavMenuLinkTitle>
                                                                <NavMenuLinkDescription>
                                                                    "Reserve desks and rooms."
                                                                </NavMenuLinkDescription>
                                                            </div>
                                                        </NavMenuLinkGrid>
                                                    </div>
                                                </NavMenuContentInset>
                                            </NavMenuContent>
                                        </NavMenuItem>
                                        <NavMenuItem>
                                            <NavMenuLink attr:href="#">"Pricing"</NavMenuLink>
                                        </NavMenuItem>
                                    </NavMenuList>
                                </NavMenu>
                            </NavMenuWrapper>
                        </NavMenuFixed>
                    </Header>
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn BottomNavSection() -> impl IntoView {
    view! {
        <Section
            title="Bottom navigation"
            description="A touch-first bottom bar of equal-width destinations. Mark the active entry with aria-current=page."
        >
            <Demo>
                <div class="w-full max-w-lg [--bottom__nav__height:4rem]">
                    <BottomNav>
                        <BottomNavGrid>
                            <BottomNavButton attr:aria-current="page">
                                <Icon icon=icondata::LuLayoutDashboard attr:class="size-5" />
                                <BottomNavLabel>"Home"</BottomNavLabel>
                            </BottomNavButton>
                            <BottomNavButton>
                                <Icon icon=icondata::LuSearch attr:class="size-5" />
                                <BottomNavLabel>"Search"</BottomNavLabel>
                            </BottomNavButton>
                            <BottomNavButton>
                                <Icon icon=icondata::LuCalendar attr:class="size-5" />
                                <BottomNavLabel>"Bookings"</BottomNavLabel>
                            </BottomNavButton>
                            <BottomNavButton>
                                <Icon icon=icondata::LuSettings attr:class="size-5" />
                                <BottomNavLabel>"Settings"</BottomNavLabel>
                            </BottomNavButton>
                        </BottomNavGrid>
                    </BottomNav>
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn FooterSection_() -> impl IntoView {
    view! {
        <Section
            title="Footer"
            description="The composed site footer: a brand block, columns of links, a centered nav row and a copyright line."
        >
            <Demo>
                <div class="overflow-hidden w-full rounded-lg border border-border bg-card">
                    <Footer class="px-2 py-6">
                        <FooterContainer>
                            <FooterSectionsGrid class="md:grid-cols-3">
                                <FooterBrand>
                                    <div class="flex gap-2 items-center">
                                        <Icon
                                            icon=icondata::LuComponent
                                            attr:class="size-5 text-primary"
                                        />
                                        <span class="font-semibold">"OpenWorkspace"</span>
                                    </div>
                                    <FooterDescription class="mt-3">
                                        "Self-hosted workspace booking for every site."
                                    </FooterDescription>
                                </FooterBrand>
                                <FooterLinksSection>
                                    <FooterTitle>"Product"</FooterTitle>
                                    <FooterLinks>
                                        <FooterLink attr:href="#">"Desks"</FooterLink>
                                        <FooterLink attr:href="#">"Rooms"</FooterLink>
                                        <FooterLink attr:href="#">"Parking"</FooterLink>
                                    </FooterLinks>
                                </FooterLinksSection>
                                <FooterLinksSection>
                                    <FooterTitle>"Company"</FooterTitle>
                                    <FooterLinks>
                                        <FooterLink attr:href="#">"About"</FooterLink>
                                        <FooterLink attr:href="#">"Contact"</FooterLink>
                                    </FooterLinks>
                                </FooterLinksSection>
                            </FooterSectionsGrid>
                            <FooterNavContainer>
                                <FooterLink attr:href="#">"Privacy"</FooterLink>
                                <FooterLink attr:href="#">"Terms"</FooterLink>
                                <FooterLink attr:href="#">"Status"</FooterLink>
                            </FooterNavContainer>
                            <FooterSection class="border-t">
                                <FooterCopyright>"\u{00A9} 2026 OpenWorkspace"</FooterCopyright>
                            </FooterSection>
                        </FooterContainer>
                    </Footer>
                </div>
            </Demo>
        </Section>
    }
}
