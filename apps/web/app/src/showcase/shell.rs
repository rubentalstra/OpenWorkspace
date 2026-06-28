use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::Outlet;
use ui::{
    Sidenav, SidenavCollapsible, SidenavContent, SidenavGroup, SidenavGroupContent,
    SidenavGroupLabel, SidenavHeader, SidenavInset, SidenavLink, SidenavMenu, SidenavMenuItem,
    SidenavTrigger, SidenavVariant, SidenavWrapper, ThemeToggle, Toaster, ToasterContext,
};

/// Persistent gallery chrome: a collapsible [`Sidenav`] of category links beside
/// an inset content surface whose [`Outlet`] renders the active page. Mounts one
/// [`Toaster`] so any page below can raise toasts. Active link highlighting is
/// automatic — [`SidenavLink`] sets `aria-current` from the live pathname.
#[component]
pub fn ShowcaseLayout() -> impl IntoView {
    // Provided above the page outlet *and* the toaster so any page below can
    // raise toasts through the shared queue the `Toaster` renders.
    provide_context(ToasterContext::new());

    view! {
        <SidenavWrapper class="min-h-screen bg-sidenav text-foreground">
            <Sidenav variant=SidenavVariant::Inset collapsible=SidenavCollapsible::Icon>
                <SidenavHeader>
                    <div class="flex gap-2 items-center px-1 h-10">
                        <Icon
                            icon=icondata::LuComponent
                            attr:class="size-5 shrink-0 text-primary"
                        />
                        <span class="text-sm font-semibold truncate group-data-[collapsible=Icon]:hidden">
                            "OpenWorkspace UI"
                        </span>
                    </div>
                </SidenavHeader>
                <SidenavContent>
                    <SidenavGroup>
                        <SidenavGroupLabel>"Gallery"</SidenavGroupLabel>
                        <SidenavGroupContent>
                            <SidenavMenu>
                                <NavItem href="/ui" label="Overview" icon=icondata::LuLayoutGrid />
                                <NavItem
                                    href="/ui/buttons"
                                    label="Buttons & actions"
                                    icon=icondata::LuMousePointerClick
                                />
                                <NavItem
                                    href="/ui/inputs"
                                    label="Inputs"
                                    icon=icondata::LuTextCursorInput
                                />
                                <NavItem
                                    href="/ui/forms"
                                    label="Forms"
                                    icon=icondata::LuClipboardList
                                />
                                <NavItem
                                    href="/ui/overlays"
                                    label="Overlays"
                                    icon=icondata::LuLayers
                                />
                                <NavItem
                                    href="/ui/navigation"
                                    label="Navigation"
                                    icon=icondata::LuCompass
                                />
                                <NavItem href="/ui/data" label="Data" icon=icondata::LuTable />
                                <NavItem href="/ui/dates" label="Dates" icon=icondata::LuCalendar />
                                <NavItem
                                    href="/ui/feedback"
                                    label="Feedback"
                                    icon=icondata::LuBell
                                />
                                <NavItem
                                    href="/ui/layout"
                                    label="Layout"
                                    icon=icondata::LuLayoutDashboard
                                />
                                <NavItem href="/ui/theme" label="Theme" icon=icondata::LuPalette />
                                <NavItem href="/ui/hooks" label="Hooks" icon=icondata::LuAnchor />
                            </SidenavMenu>
                        </SidenavGroupContent>
                    </SidenavGroup>
                </SidenavContent>
            </Sidenav>

            <SidenavInset attr:data-variant="Inset">
                <header class="flex sticky top-0 z-20 gap-3 items-center px-4 h-14 border-b backdrop-blur border-sidenav-border bg-background/80">
                    <SidenavTrigger>
                        <Icon icon=icondata::LuPanelLeft attr:class="size-4" />
                    </SidenavTrigger>
                    <div class="flex-1 text-sm font-medium text-muted-foreground">
                        "Component gallery"
                    </div>
                    <ThemeToggle />
                </header>
                <main class="overflow-auto flex-1">
                    <div class="px-6 py-10 mx-auto w-full max-w-6xl">
                        <Outlet />
                    </div>
                </main>
            </SidenavInset>

            <Toaster />
        </SidenavWrapper>
    }
}

/// One sidenav row: a leading glyph and a label that collapses to icon-only.
#[component]
fn NavItem(
    #[prop(into)] href: String,
    #[prop(into)] label: String,
    icon: icondata::Icon,
) -> impl IntoView {
    view! {
        <SidenavMenuItem>
            <SidenavLink href=href>
                <Icon icon=icon attr:class="size-4 shrink-0" />
                <span>{label}</span>
            </SidenavLink>
        </SidenavMenuItem>
    }
}
