use crate::cn;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::input::Input;
use crate::components::separator::Separator;
use crate::components::sheet::{
    Sheet, SheetContent, SheetDescription, SheetHeader, SheetSide, SheetTitle,
};
use crate::components::skeleton::Skeleton;
use crate::components::tooltip::{Tooltip, TooltipContent, TooltipSide};
use crate::hooks::use_is_mobile::use_is_mobile;
use leptos::ev;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;

const SIDEBAR_COOKIE_NAME: &str = "sidebar_state";
const SIDEBAR_COOKIE_MAX_AGE: i64 = 60 * 60 * 24 * 7;
const SIDEBAR_WIDTH: &str = "16rem";
const SIDEBAR_WIDTH_MOBILE: &str = "18rem";
const SIDEBAR_WIDTH_ICON: &str = "3rem";
/// The key (with ⌘/Ctrl) that toggles the sidebar.
const SIDEBAR_KEYBOARD_SHORTCUT: &str = "b";

/// Persist the open state to the `sidebar_state` cookie so it survives reloads,
/// mirroring shadcn's `document.cookie` write. Browser-only; a no-op on the server.
fn write_sidebar_cookie(open: bool) {
    if let Some(html_doc) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.dyn_into::<web_sys::HtmlDocument>().ok())
    {
        _ = html_doc.set_cookie(&format!(
            "{SIDEBAR_COOKIE_NAME}={open}; path=/; max-age={SIDEBAR_COOKIE_MAX_AGE}"
        ));
    }
}

/// The sidebar state shared with every descendant via context. Obtain it with
/// [`use_sidebar`]. Mirrors shadcn's `SidebarContext` (`state`, `open`/`setOpen`,
/// `openMobile`/`setOpenMobile`, `isMobile`, `toggleSidebar`).
#[derive(Clone, Copy)]
pub struct SidebarContext {
    open: RwSignal<bool>,
    open_mobile: RwSignal<bool>,
    is_mobile: Signal<bool>,
}

impl SidebarContext {
    /// Whether the desktop sidebar is expanded.
    #[must_use]
    pub fn open(self) -> bool {
        self.open.get()
    }

    /// Set the desktop open state and persist it to the cookie.
    pub fn set_open(self, value: bool) {
        self.open.set(value);
        write_sidebar_cookie(value);
    }

    /// Whether the mobile sheet is open.
    #[must_use]
    pub fn open_mobile(self) -> bool {
        self.open_mobile.get()
    }

    /// Set the mobile sheet open state.
    pub fn set_open_mobile(self, value: bool) {
        self.open_mobile.set(value);
    }

    /// Whether the viewport is below the mobile breakpoint.
    #[must_use]
    pub fn is_mobile(self) -> bool {
        self.is_mobile.get()
    }

    /// `"expanded"` or `"collapsed"`, for `data-state` styling.
    #[must_use]
    pub fn state(self) -> &'static str {
        if self.open.get() {
            "expanded"
        } else {
            "collapsed"
        }
    }

    /// Toggle the sidebar — the mobile sheet on small screens, otherwise the
    /// desktop sidebar (persisting the cookie).
    pub fn toggle(self) {
        if self.is_mobile.get_untracked() {
            let next = !self.open_mobile.get_untracked();
            self.open_mobile.set(next);
        } else {
            let next = !self.open.get_untracked();
            self.set_open(next);
        }
    }
}

/// Access the enclosing [`SidebarContext`]; panics if used outside a
/// [`SidebarProvider`], matching shadcn's `useSidebar` invariant.
#[must_use]
pub fn use_sidebar() -> SidebarContext {
    expect_context::<SidebarContext>()
}

/// SidebarProvider — shadcn `SidebarProvider`. Owns the sidebar state, registers the
/// ⌘/Ctrl+B shortcut, persists the open state to a cookie, and injects the
/// `--sidebar-width*` custom properties. Controlled via an external `open` signal or
/// uncontrolled via `default_open`.
#[component]
pub fn SidebarProvider(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = true)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let is_mobile = use_is_mobile();
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let ctx = SidebarContext {
        open,
        open_mobile: RwSignal::new(false),
        is_mobile,
    };
    provide_context(ctx);

    let on_key = window_event_listener(ev::keydown, move |event| {
        if event.key() == SIDEBAR_KEYBOARD_SHORTCUT && (event.meta_key() || event.ctrl_key()) {
            event.prevent_default();
            ctx.toggle();
        }
    });
    on_cleanup(move || on_key.remove());

    view! {
        <div
            data-slot="sidebar-wrapper"
            style=format!(
                "--sidebar-width:{SIDEBAR_WIDTH};--sidebar-width-icon:{SIDEBAR_WIDTH_ICON};",
            )
            class=move || {
                cn!(
                    "group/sidebar-wrapper flex min-h-svh w-full has-data-[variant=inset]:bg-sidebar",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// The side a [`Sidebar`] is docked to.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SidebarSide {
    /// Docked to the left edge (the default).
    #[default]
    Left,
    /// Docked to the right edge.
    Right,
}

impl SidebarSide {
    fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    fn sheet_side(self) -> SheetSide {
        match self {
            Self::Left => SheetSide::Left,
            Self::Right => SheetSide::Right,
        }
    }
}

/// The visual treatment of a [`Sidebar`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SidebarVariant {
    /// Flush against the edge (the default).
    #[default]
    Sidebar,
    /// A floating, inset panel.
    Floating,
    /// A panel whose content area is itself inset.
    Inset,
}

impl SidebarVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sidebar => "sidebar",
            Self::Floating => "floating",
            Self::Inset => "inset",
        }
    }

    fn is_padded(self) -> bool {
        matches!(self, Self::Floating | Self::Inset)
    }
}

/// How a [`Sidebar`] collapses.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SidebarCollapsible {
    /// Slides entirely off-canvas (the default).
    #[default]
    Offcanvas,
    /// Collapses to an icon rail.
    Icon,
    /// Does not collapse.
    None,
}

impl SidebarCollapsible {
    fn as_str(self) -> &'static str {
        match self {
            Self::Offcanvas => "offcanvas",
            Self::Icon => "icon",
            Self::None => "none",
        }
    }
}

/// Sidebar — shadcn `Sidebar`. Renders one of three ways: a plain panel
/// (`collapsible=None`), a [`Sheet`] on mobile, or the gap+container desktop layout
/// that animates between expanded, icon-rail, and off-canvas states.
#[component]
pub fn Sidebar(
    #[prop(into, optional)] side: Signal<SidebarSide>,
    #[prop(into, optional)] variant: Signal<SidebarVariant>,
    #[prop(into, optional)] collapsible: Signal<SidebarCollapsible>,
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = use_sidebar();
    let children = StoredValue::new(children);
    view! {
        {move || {
            if collapsible.get() == SidebarCollapsible::None {
                return view! {
                    <div
                        data-slot="sidebar"
                        class=move || {
                            cn!(
                                "flex h-full w-(--sidebar-width) flex-col bg-sidebar text-sidebar-foreground",
                                class.get(),
                            )
                        }
                    >
                        {children.with_value(|children| children())}
                    </div>
                }
                    .into_any();
            }
            if ctx.is_mobile() {
                return view! {
                    <Sheet open=ctx.open_mobile>
                        <SheetContent
                            side=Signal::derive(move || side.get().sheet_side())
                            show_close=false
                            class="w-(--sidebar-width) bg-sidebar p-0 text-sidebar-foreground [&>button]:hidden"
                        >
                            <SheetHeader class="sr-only">
                                <SheetTitle>"Sidebar"</SheetTitle>
                                <SheetDescription>"Displays the mobile sidebar."</SheetDescription>
                            </SheetHeader>
                            <div
                                data-sidebar="sidebar"
                                data-slot="sidebar"
                                data-mobile="true"
                                style=format!("--sidebar-width:{SIDEBAR_WIDTH_MOBILE};")
                                class="flex h-full w-full flex-col"
                            >
                                {children.with_value(|children| children())}
                            </div>
                        </SheetContent>
                    </Sheet>
                }
                    .into_any();
            }
            let gap_class = move || {
                cn!(
                    "cn-sidebar-gap relative w-(--sidebar-width) bg-transparent group-data-[collapsible=offcanvas]:w-0 group-data-[side=right]:rotate-180",
                    if variant.get().is_padded() {
                        "group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4)))]"
                    } else {
                        "group-data-[collapsible=icon]:w-(--sidebar-width-icon)"
                    },
                )
            };
            let container_class = move || {
                cn!(
                    "fixed inset-y-0 z-10 hidden h-svh w-(--sidebar-width) transition-[left,right,width] duration-200 ease-linear data-[side=left]:left-0 data-[side=left]:group-data-[collapsible=offcanvas]:left-[calc(var(--sidebar-width)*-1)] data-[side=right]:right-0 data-[side=right]:group-data-[collapsible=offcanvas]:right-[calc(var(--sidebar-width)*-1)] md:flex",
                    if variant.get().is_padded() {
                        "p-2 group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4))+2px)]"
                    } else {
                        "group-data-[collapsible=icon]:w-(--sidebar-width-icon) group-data-[side=left]:border-r group-data-[side=right]:border-l"
                    },
                    class.get(),
                )
            };
            view! {
                <div
                    class="group peer hidden text-sidebar-foreground md:block"
                    data-slot="sidebar"
                    data-state=move || ctx.state()
                    data-collapsible=move || {
                        if ctx.state() == "collapsed" { collapsible.get().as_str() } else { "" }
                    }
                    data-variant=move || variant.get().as_str()
                    data-side=move || side.get().as_str()
                >
                    <div data-slot="sidebar-gap" class=gap_class></div>
                    <div
                        data-slot="sidebar-container"
                        data-side=move || side.get().as_str()
                        class=container_class
                    >
                        <div
                            data-sidebar="sidebar"
                            data-slot="sidebar-inner"
                            class="cn-sidebar-inner flex size-full flex-col"
                        >
                            {children.with_value(|children| children())}
                        </div>
                    </div>
                </div>
            }
                .into_any()
        }}
    }
}

/// The control that toggles the sidebar — a ghost icon [`Button`] with the panel glyph.
#[component]
pub fn SidebarTrigger(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let ctx = use_sidebar();
    view! {
        <Button
            variant=ButtonVariant::Ghost
            size=ButtonSize::IconSm
            class=Signal::derive(move || cn!("cn-sidebar-trigger", class.get()))
            attr:r#type="button"
            attr:data-sidebar="trigger"
            attr:data-slot="sidebar-trigger"
            on:click=move |_| ctx.toggle()
        >
            <Icon icon=icondata::LuPanelLeft attr:class="cn-rtl-flip" />
            <span class="sr-only">"Toggle Sidebar"</span>
        </Button>
    }
}

/// The thin draggable rail along the sidebar edge; clicking it toggles the sidebar.
#[component]
pub fn SidebarRail(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let ctx = use_sidebar();
    view! {
        <button
            type="button"
            data-sidebar="rail"
            data-slot="sidebar-rail"
            aria-label="Toggle Sidebar"
            tabindex="-1"
            title="Toggle Sidebar"
            class=move || {
                cn!(
                    "cn-sidebar-rail absolute inset-y-0 z-20 hidden w-4 transition-all ease-linear group-data-[side=left]:-right-4 group-data-[side=right]:left-0 after:absolute after:inset-y-0 after:start-1/2 after:w-[2px] sm:flex in-data-[side=left]:cursor-w-resize in-data-[side=right]:cursor-e-resize [[data-side=left][data-state=collapsed]_&]:cursor-e-resize [[data-side=right][data-state=collapsed]_&]:cursor-w-resize group-data-[collapsible=offcanvas]:translate-x-0 group-data-[collapsible=offcanvas]:after:left-full hover:group-data-[collapsible=offcanvas]:bg-sidebar [[data-side=left][data-collapsible=offcanvas]_&]:-right-2 [[data-side=right][data-collapsible=offcanvas]_&]:-left-2",
                    class.get(),
                )
            }
            on:click=move |_| ctx.toggle()
        ></button>
    }
}

/// The main content area beside the sidebar.
#[component]
pub fn SidebarInset(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <main
            data-slot="sidebar-inset"
            class=move || cn!("cn-sidebar-inset relative flex w-full flex-1 flex-col", class.get())
        >
            {children()}
        </main>
    }
}

/// A search [`Input`] styled for the sidebar.
#[component]
pub fn SidebarInput(
    #[prop(into, optional)] placeholder: Signal<String>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <Input
            class=Signal::derive(move || cn!("cn-sidebar-input", class.get()))
            attr:data-slot="sidebar-input"
            attr:data-sidebar="input"
            attr:placeholder=move || placeholder.get()
        />
    }
}

/// The sidebar's header region (stays pinned above the scrolling content).
#[component]
pub fn SidebarHeader(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="sidebar-header"
            data-sidebar="header"
            class=move || cn!("cn-sidebar-header flex flex-col", class.get())
        >
            {children()}
        </div>
    }
}

/// The sidebar's footer region (stays pinned below the scrolling content).
#[component]
pub fn SidebarFooter(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="sidebar-footer"
            data-sidebar="footer"
            class=move || cn!("cn-sidebar-footer flex flex-col", class.get())
        >
            {children()}
        </div>
    }
}

/// A [`Separator`] styled for the sidebar.
#[component]
pub fn SidebarSeparator(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <Separator
            class=Signal::derive(move || cn!("cn-sidebar-separator w-auto", class.get()))
            attr:data-slot="sidebar-separator"
            attr:data-sidebar="separator"
        />
    }
}

/// The scrolling body that holds the sidebar's groups and menus.
#[component]
pub fn SidebarContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="sidebar-content"
            data-sidebar="content"
            class=move || {
                cn!(
                    "cn-sidebar-content flex min-h-0 flex-1 flex-col overflow-auto group-data-[collapsible=icon]:overflow-hidden",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// A labelled section within the sidebar content.
#[component]
pub fn SidebarGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="sidebar-group"
            data-sidebar="group"
            class=move || cn!("cn-sidebar-group relative flex w-full min-w-0 flex-col", class.get())
        >
            {children()}
        </div>
    }
}

/// The heading for a [`SidebarGroup`]; hidden when the sidebar is icon-collapsed.
#[component]
pub fn SidebarGroupLabel(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="sidebar-group-label"
            data-sidebar="group-label"
            class=move || {
                cn!(
                    "cn-sidebar-group-label flex shrink-0 items-center outline-hidden [&>svg]:shrink-0",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// An action button aligned to a [`SidebarGroup`]'s label.
#[component]
pub fn SidebarGroupAction(
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            data-slot="sidebar-group-action"
            data-sidebar="group-action"
            class=move || {
                cn!(
                    "cn-sidebar-group-action flex aspect-square items-center justify-center outline-hidden transition-transform group-data-[collapsible=icon]:hidden after:absolute after:-inset-2 md:after:hidden [&>svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(());
                }
            }
        >
            {children()}
        </button>
    }
}

/// The content region of a [`SidebarGroup`].
#[component]
pub fn SidebarGroupContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="sidebar-group-content"
            data-sidebar="group-content"
            class=move || cn!("cn-sidebar-group-content w-full", class.get())
        >
            {children()}
        </div>
    }
}

/// The list element wrapping [`SidebarMenuItem`]s.
#[component]
pub fn SidebarMenu(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <ul
            data-slot="sidebar-menu"
            data-sidebar="menu"
            class=move || cn!("cn-sidebar-menu flex w-full min-w-0 flex-col", class.get())
        >
            {children()}
        </ul>
    }
}

/// A single item in a [`SidebarMenu`].
#[component]
pub fn SidebarMenuItem(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <li
            data-slot="sidebar-menu-item"
            data-sidebar="menu-item"
            class=move || cn!("group/menu-item relative", class.get())
        >
            {children()}
        </li>
    }
}

/// Tone of a [`SidebarMenuButton`], surfaced via the nova layer.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SidebarMenuButtonVariant {
    /// The standard ghost-style button.
    #[default]
    Default,
    /// A bordered/outline button.
    Outline,
}

impl SidebarMenuButtonVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-sidebar-menu-button-variant-default",
            Self::Outline => "cn-sidebar-menu-button-variant-outline",
        }
    }
}

/// Height of a [`SidebarMenuButton`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SidebarMenuButtonSize {
    /// The default height.
    #[default]
    Default,
    /// A compact height.
    Sm,
    /// A taller height (e.g. for a two-line label).
    Lg,
}

impl SidebarMenuButtonSize {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-sidebar-menu-button-size-default",
            Self::Sm => "cn-sidebar-menu-button-size-sm",
            Self::Lg => "cn-sidebar-menu-button-size-lg",
        }
    }

    fn data(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Lg => "lg",
        }
    }
}

const SIDEBAR_MENU_BUTTON_BASE: &str = "cn-sidebar-menu-button peer/menu-button group/menu-button flex w-full items-center overflow-hidden outline-hidden disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 [&_svg]:size-4 [&_svg]:shrink-0 [&>span:last-child]:truncate";

/// The primary clickable row in a sidebar menu. Renders an `<a>` when `href` is set
/// (the common nav-link case), otherwise a `<button>`. When `tooltip` is set the row
/// shows a right-anchored tooltip while the sidebar is icon-collapsed on desktop.
#[component]
pub fn SidebarMenuButton(
    #[prop(into, optional)] is_active: Signal<bool>,
    #[prop(into, optional)] variant: Signal<SidebarMenuButtonVariant>,
    #[prop(into, optional)] size: Signal<SidebarMenuButtonSize>,
    #[prop(into, optional)] tooltip: Option<String>,
    #[prop(into, optional)] href: Option<String>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = use_sidebar();
    let children = StoredValue::new(children);
    let row_class = move || {
        cn!(
            SIDEBAR_MENU_BUTTON_BASE,
            variant.get().class(),
            size.get().class(),
            class.get(),
        )
    };
    let row = move || match href.clone() {
        Some(href) => view! {
            <a
                data-slot="sidebar-menu-button"
                data-sidebar="menu-button"
                data-size=move || size.get().data()
                data-active=move || is_active.get().to_string()
                href=href
                class=row_class
                on:click=move |_| {
                    if let Some(cb) = on_click {
                        cb.run(());
                    }
                }
            >
                {children.with_value(|children| children())}
            </a>
        }
        .into_any(),
        None => view! {
            <button
                type="button"
                data-slot="sidebar-menu-button"
                data-sidebar="menu-button"
                data-size=move || size.get().data()
                data-active=move || is_active.get().to_string()
                class=row_class
                on:click=move |_| {
                    if let Some(cb) = on_click {
                        cb.run(());
                    }
                }
            >
                {children.with_value(|children| children())}
            </button>
        }
        .into_any(),
    };

    match tooltip {
        None => row().into_any(),
        Some(tooltip) => {
            let tooltip = StoredValue::new(tooltip);
            view! {
                <Tooltip class="block w-full">
                    {row()} <Show when=move || ctx.state() == "collapsed" && !ctx.is_mobile()>
                        <TooltipContent side=TooltipSide::Right>
                            {tooltip.get_value()}
                        </TooltipContent>
                    </Show>
                </Tooltip>
            }
            .into_any()
        }
    }
}

/// A trailing action button on a [`SidebarMenuItem`]. Set `show_on_hover` to reveal
/// it only on hover/focus of the item.
#[component]
pub fn SidebarMenuAction(
    #[prop(default = false)] show_on_hover: bool,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            data-slot="sidebar-menu-action"
            data-sidebar="menu-action"
            class=move || {
                cn!(
                    "cn-sidebar-menu-action flex items-center justify-center outline-hidden transition-transform group-data-[collapsible=icon]:hidden after:absolute after:-inset-2 md:after:hidden [&>svg]:shrink-0",
                    show_on_hover
                        .then_some(
                            "group-focus-within/menu-item:opacity-100 group-hover/menu-item:opacity-100 peer-data-active/menu-button:text-sidebar-accent-foreground aria-expanded:opacity-100 md:opacity-0",
                        ),
                    class.get(),
                )
            }
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(());
                }
            }
        >
            {children()}
        </button>
    }
}

/// A count/status badge on a [`SidebarMenuItem`]; hidden when icon-collapsed.
#[component]
pub fn SidebarMenuBadge(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="sidebar-menu-badge"
            data-sidebar="menu-badge"
            class=move || {
                cn!(
                    "cn-sidebar-menu-badge flex items-center justify-center tabular-nums select-none group-data-[collapsible=icon]:hidden",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// A loading placeholder for a sidebar menu row. `width` sets the text bar width
/// (the upstream randomises this; we take it as a prop to keep server/client render
/// identical). Set `show_icon` to include a leading icon placeholder.
#[component]
pub fn SidebarMenuSkeleton(
    #[prop(default = false)] show_icon: bool,
    #[prop(into, default = String::from("70%"))] width: String,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <div
            data-slot="sidebar-menu-skeleton"
            data-sidebar="menu-skeleton"
            class=move || cn!("cn-sidebar-menu-skeleton flex items-center", class.get())
        >
            <Show when=move || show_icon>
                <Skeleton
                    class="cn-sidebar-menu-skeleton-icon"
                    attr:data-sidebar="menu-skeleton-icon"
                />
            </Show>
            <Skeleton
                class="cn-sidebar-menu-skeleton-text max-w-(--skeleton-width) flex-1"
                attr:data-sidebar="menu-skeleton-text"
                attr:style=move || format!("--skeleton-width:{width};")
            />
        </div>
    }
}

/// The nested submenu list under a [`SidebarMenuItem`].
#[component]
pub fn SidebarMenuSub(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <ul
            data-slot="sidebar-menu-sub"
            data-sidebar="menu-sub"
            class=move || cn!("cn-sidebar-menu-sub flex min-w-0 flex-col", class.get())
        >
            {children()}
        </ul>
    }
}

/// A single item within a [`SidebarMenuSub`].
#[component]
pub fn SidebarMenuSubItem(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <li
            data-slot="sidebar-menu-sub-item"
            data-sidebar="menu-sub-item"
            class=move || cn!("group/menu-sub-item relative", class.get())
        >
            {children()}
        </li>
    }
}

/// Height of a [`SidebarMenuSubButton`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SidebarMenuSubButtonSize {
    /// A compact sub-row.
    Sm,
    /// The default sub-row (medium).
    #[default]
    Md,
}

impl SidebarMenuSubButtonSize {
    fn data(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
        }
    }
}

/// A link row within a [`SidebarMenuSub`]; hidden when the sidebar is icon-collapsed.
#[component]
pub fn SidebarMenuSubButton(
    #[prop(into, optional)] size: Signal<SidebarMenuSubButtonSize>,
    #[prop(into, optional)] is_active: Signal<bool>,
    #[prop(into, optional)] href: Option<String>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <a
            data-slot="sidebar-menu-sub-button"
            data-sidebar="menu-sub-button"
            data-size=move || size.get().data()
            data-active=move || is_active.get().to_string()
            href=href
            class=move || {
                cn!(
                    "cn-sidebar-menu-sub-button flex min-w-0 -translate-x-px items-center overflow-hidden outline-hidden group-data-[collapsible=icon]:hidden disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 [&>span:last-child]:truncate [&>svg]:shrink-0",
                    class.get(),
                )
            }
        >
            {children()}
        </a>
    }
}
