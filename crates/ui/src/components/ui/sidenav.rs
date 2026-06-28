use crate::{clx, cn, variants, void};
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

clx! {
    /// Inset surface that sits beside the sidenav and holds the main content.
    SidenavInset, div,
    "bg-background relative flex w-full flex-1 flex-col data-[variant=Inset]:rounded-lg data-[variant=Inset]:border data-[variant=Inset]:border-sidenav-border data-[variant=Inset]:shadow-sm data-[variant=Inset]:m-2"
}
clx! {
    /// Inner panel wrapping the sidenav body; reads `data-variant` for floating chrome.
    SidenavInner, div,
    "flex flex-col w-full h-full bg-sidenav data-[variant=Floating]:rounded-lg data-[variant=Floating]:border data-[variant=Floating]:border-sidenav-border data-[variant=Floating]:shadow-sm"
}
clx! {
    /// Top region of the sidenav, typically a brand or header row.
    SidenavHeader, div, "flex flex-col gap-2 p-2"
}
clx! {
    /// Vertical list container for [`SidenavMenuItem`]s.
    SidenavMenu, ul, "flex flex-col gap-1 w-full min-w-0"
}
clx! {
    /// Nested list for sub-menu entries, indented under a parent item.
    SidenavMenuSub, ul,
    "border-sidenav-border mx-3.5 flex min-w-0 translate-x-px flex-col gap-1 border-l px-2.5 py-0.5 group-data-[collapsible=Icon]:hidden"
}
clx! {
    /// Row wrapping a menu button and its optional action.
    SidenavMenuItem, li, "relative group/menu-item"
}
clx! {
    /// Row wrapping a sub-menu button.
    SidenavMenuSubItem, li, "group/menu-sub-item"
}
clx! {
    /// Scrollable body holding the sidenav's groups and menus.
    SidenavContent, div, "scrollbar__on_hover",
    "flex min-h-0 flex-1 flex-col gap-2 overflow-auto group-data-[collapsible=Icon]:overflow-hidden"
}
clx! {
    /// Labelled section within the sidenav content.
    SidenavGroup, div, "flex relative flex-col p-2 w-full min-w-0"
}
clx! {
    /// Body of a [`SidenavGroup`].
    SidenavGroupContent, div, "w-full text-sm"
}
clx! {
    /// Caption above a [`SidenavGroup`], hidden when collapsed to icons.
    SidenavGroupLabel, div,
    "text-sidenav-foreground/70 ring-sidenav-ring flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium outline-hidden transition-[margin,opacity] duration-200 ease-linear focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0 group-data-[collapsible=Icon]:-mt-8 group-data-[collapsible=Icon]:opacity-0"
}
clx! {
    /// Bottom region of the sidenav, typically account or settings rows.
    SidenavFooter, footer, "flex flex-col gap-2 p-2"
}

void! {
    /// Search field styled for placement inside a [`SidenavHeader`]. Forwards
    /// every native `<input>` attribute, event and binding to the element.
    SidenavInput, input,
    "file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input flex h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
    "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-2",
    "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
    "read-only:bg-muted",
    "w-full h-8 shadow-none bg-background"
}

variants! {
    /// Primary clickable row inside a [`SidenavMenuItem`]. Pass `href` to render
    /// an anchor with active-state `aria-current`; otherwise it is a button.
    SidenavMenuButton {
        base: "peer/menu-button flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm outline-hidden ring-sidenav-ring transition-[width,height,padding] hover:bg-sidenav-accent hover:text-sidenav-accent-foreground focus-visible:ring-2 active:bg-sidenav-accent active:text-sidenav-accent-foreground disabled:pointer-events-none disabled:opacity-50 group-has-data-[sidenav=menu-action]/menu-item:pr-8 aria-disabled:pointer-events-none aria-disabled:opacity-50 aria-[current=page]:bg-sidenav-accent aria-[current=page]:font-medium aria-[current=page]:text-sidenav-accent-foreground data-[state=open]:hover:bg-sidenav-accent data-[state=open]:hover:text-sidenav-accent-foreground [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0 group-data-[collapsible=Icon]:size-8! group-data-[collapsible=Icon]:p-0! [&>svg]:stroke-2 aria-[current=page]:[&>svg]:stroke-[2.7]",
        variants: {
            variant: {
                Default: "",
                Outline: "bg-background shadow-[0_0_0_1px_hsl(var(--sidenav-border))] hover:shadow-[0_0_0_1px_hsl(var(--sidenav-accent))]",
            },
            size: {
                Default: "h-8 text-sm",
                Sm: "h-7 text-xs",
                Lg: "h-12",
            }
        },
        component: { element: button, support_href: true, support_aria_current: true }
    }
}

/// Shared open/closed state for the sidenav, provided by [`SidenavWrapper`] and
/// consumed by [`SidenavTrigger`], [`Sidenav`] and the toggle rail.
#[derive(Clone, Copy)]
pub struct SidenavContext {
    /// `true` when the sidenav is expanded.
    pub open: RwSignal<bool>,
}

impl SidenavContext {
    /// Flips the sidenav between expanded and collapsed.
    pub fn toggle(self) {
        self.open.update(|open| *open = !*open);
    }
}

/// Visual treatment of the [`Sidenav`] container.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SidenavVariant {
    /// Flush against the viewport edge.
    #[default]
    Sidenav,
    /// Detached card with its own border and shadow.
    Floating,
    /// Inset panel that pairs with a [`SidenavInset`] content surface.
    Inset,
}

impl SidenavVariant {
    fn as_data(self) -> &'static str {
        match self {
            Self::Sidenav => "Sidenav",
            Self::Floating => "Floating",
            Self::Inset => "Inset",
        }
    }
}

/// Edge of the viewport the [`Sidenav`] is anchored to.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SidenavSide {
    /// Anchored to the left edge.
    #[default]
    Left,
    /// Anchored to the right edge.
    Right,
}

impl SidenavSide {
    fn as_data(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

/// How the [`Sidenav`] behaves when collapsed.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SidenavCollapsible {
    /// Slides entirely off-canvas.
    #[default]
    Offcanvas,
    /// Cannot collapse; renders a static rail.
    None,
    /// Collapses to an icon-only rail.
    Icon,
}

impl SidenavCollapsible {
    fn as_data(self) -> &'static str {
        match self {
            Self::Offcanvas => "Offcanvas",
            Self::None => "None",
            Self::Icon => "Icon",
        }
    }
}

/// Expanded/collapsed presentation of the [`Sidenav`], used when the sidenav is
/// driven directly rather than through a [`SidenavWrapper`].
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SidenavState {
    /// Full-width, labels visible.
    #[default]
    Expanded,
    /// Hidden or reduced to icons.
    Collapsed,
}

impl SidenavState {
    fn as_data(self) -> &'static str {
        match self {
            Self::Expanded => "Expanded",
            Self::Collapsed => "Collapsed",
        }
    }
}

fn open_state_data(open: bool) -> &'static str {
    if open {
        SidenavState::Expanded.as_data()
    } else {
        SidenavState::Collapsed.as_data()
    }
}

fn active_path(target: &str, path: &str) -> Option<&'static str> {
    (path == target || path.starts_with(&format!("{target}/"))).then_some("page")
}

/// Root layout wrapper that provides [`SidenavContext`] to all descendants.
/// `default_open` seeds the initial expanded state.
#[component]
pub fn SidenavWrapper(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(default = true)] default_open: bool,
    children: Children,
) -> impl IntoView {
    let open = RwSignal::new(default_open);

    view! {
        <Provider value=SidenavContext { open }>
            <div
                data-name="SidenavWrapper"
                class=move || {
                    cn!(
                        "group/sidenav-wrapper has-data-[variant=Inset]:bg-sidenav flex h-full w-full",
                        class.get(),
                    )
                }
            >
                {children()}
            </div>
        </Provider>
    }
}

/// The sidenav itself. Reads [`SidenavContext`] for its open state, falling back
/// to `data_state` when used outside a [`SidenavWrapper`]. `collapsible`,
/// `variant` and `side` drive the responsive layout via `data-*` styling slots.
#[component]
pub fn Sidenav(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] variant: Signal<SidenavVariant>,
    #[prop(into, optional)] data_state: Signal<SidenavState>,
    #[prop(into, optional)] side: Signal<SidenavSide>,
    #[prop(into, optional)] collapsible: Signal<SidenavCollapsible>,
    children: Children,
) -> impl IntoView {
    let ctx = use_context::<SidenavContext>();
    let is_open = Signal::derive(move || {
        ctx.map_or_else(
            || data_state.get() == SidenavState::Expanded,
            |c| c.open.get(),
        )
    });

    if collapsible.get_untracked() == SidenavCollapsible::None {
        view! {
            <aside
                data-name="Sidenav"
                class=move || {
                    cn!(
                        "flex flex-col h-full bg-sidenav text-sidenav-foreground w-(--sidenav-width)",
                        class.get(),
                    )
                }
            >
                {children()}
            </aside>
        }
        .into_any()
    } else {
        // Width/position are computed from the open *state* (not just the
        // collapsible mode): open → full width on-screen; collapsed → icon-width
        // rail (Icon) or slid off-screen (Offcanvas). The container is fixed and
        // `inset-y-0`, so the sidenav always spans the full viewport height while
        // the in-flow gap reserves matching horizontal space for the content.
        let gap_class = move || {
            let width = if is_open.get() {
                "w-(--sidenav-width)"
            } else if collapsible.get() == SidenavCollapsible::Icon {
                "w-(--sidenav-width-icon)"
            } else {
                "w-0"
            };
            cn!(
                "relative h-svh bg-transparent transition-[width] duration-200 ease-linear",
                width,
            )
        };
        let container_class = move || {
            let open = is_open.get();
            let icon = collapsible.get() == SidenavCollapsible::Icon;
            let (anchor, width) = match (side.get(), open, icon) {
                (SidenavSide::Left, true, _) => ("left-0", "w-(--sidenav-width)"),
                (SidenavSide::Left, false, true) => ("left-0", "w-(--sidenav-width-icon)"),
                (SidenavSide::Left, false, false) => {
                    ("left-[calc(var(--sidenav-width)*-1)]", "w-(--sidenav-width)")
                }
                (SidenavSide::Right, true, _) => ("right-0", "w-(--sidenav-width)"),
                (SidenavSide::Right, false, true) => ("right-0", "w-(--sidenav-width-icon)"),
                (SidenavSide::Right, false, false) => {
                    ("right-[calc(var(--sidenav-width)*-1)]", "w-(--sidenav-width)")
                }
            };
            let border = if side.get() == SidenavSide::Left {
                "border-r border-sidenav-border"
            } else {
                "border-l border-sidenav-border"
            };
            cn!(
                "fixed inset-y-0 z-10 hidden h-svh flex-col transition-[left,right,width] duration-200 ease-linear md:flex",
                anchor,
                width,
                border,
                class.get(),
            )
        };

        view! {
            <aside
                data-name="Sidenav"
                data-state=move || open_state_data(is_open.get())
                data-side=move || side.get().as_data()
                data-collapsible=move || collapsible.get().as_data()
                class="hidden md:block group peer text-sidenav-foreground"
            >
                <div data-name="SidenavGap" class=gap_class />
                <div data-name="SidenavContainer" class=container_class>
                    <SidenavInner
                        attr:data-sidenav="Sidenav"
                        attr:data-variant=move || variant.get().as_data()
                    >
                        {children()}
                        <SidenavToggleRail />
                    </SidenavInner>
                </div>
            </aside>
        }
        .into_any()
    }
}

/// Button that toggles the enclosing [`Sidenav`] via [`SidenavContext`].
#[component]
pub fn SidenavTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = use_context::<SidenavContext>();

    view! {
        <button
            type="button"
            data-name="SidenavTrigger"
            aria-label="Toggle sidenav"
            aria-pressed=move || ctx.map(|c| c.open.get().to_string())
            on:click=move |_| {
                if let Some(c) = ctx {
                    c.toggle();
                }
            }
            class=move || {
                cn!(
                    "inline-flex gap-2 justify-center items-center -ml-1 text-sm font-medium whitespace-nowrap rounded-md transition-all outline-none disabled:opacity-50 disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 aria-invalid:ring-destructive/20 aria-invalid:border-destructive size-7 dark:aria-invalid:ring-destructive/40 dark:hover:bg-accent/50 hover:bg-accent hover:text-accent-foreground focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]",
                    class.get(),
                )
            }
        >
            {children()}
        </button>
    }
}

/// Thin rail along the sidenav edge that toggles its open state.
#[component]
fn SidenavToggleRail() -> impl IntoView {
    let ctx = use_context::<SidenavContext>();

    view! {
        <button
            type="button"
            data-name="SidenavToggleRail"
            aria-label="Toggle sidenav"
            tabindex="-1"
            on:click=move |_| {
                if let Some(c) = ctx {
                    c.toggle();
                }
            }
            class="hidden absolute inset-y-0 z-20 w-4 transition-all ease-linear -translate-x-1/2 sm:flex group-data-[side=Left]:-right-4 group-data-[side=Right]:left-0 after:absolute after:inset-y-0 after:left-1/2 after:w-[2px] in-data-[side=Left]:cursor-w-resize in-data-[side=Right]:cursor-e-resize [[data-side=Left][data-state=Collapsed]_&]:cursor-e-resize [[data-side=Right][data-state=Collapsed]_&]:cursor-w-resize group-data-[collapsible=Offcanvas]:translate-x-0 group-data-[collapsible=Offcanvas]:after:left-full [[data-side=Left][data-collapsible=Offcanvas]_&]:-right-2 [[data-side=Right][data-collapsible=Offcanvas]_&]:left-2 hover:after:bg-sidenav-border hover:group-data-[collapsible=Offcanvas]:bg-sidenav"
        />
    }
}

/// Anchor row for top-level navigation. Marks the active route with
/// `aria-current="page"` from the live pathname. Native attributes forward to
/// the `<a>`.
#[component]
pub fn SidenavLink(
    #[prop(into)] href: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let location = use_location();
    let target = href.clone();
    let aria_current = move || active_path(&target, &location.pathname.get());

    view! {
        <a
            data-name="SidenavLink"
            href=href
            aria-current=aria_current
            class=move || {
                cn!(
                    "peer/menu-button flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left outline-hidden ring-sidenav-ring transition-[width,height,padding] focus-visible:ring-2 active:bg-sidenav-accent active:text-sidenav-accent-foreground disabled:pointer-events-none disabled:opacity-50 group-has-data-[sidenav=menu-action]/menu-item:pr-8 aria-disabled:pointer-events-none aria-disabled:opacity-50 aria-[current=page]:bg-sidenav-accent aria-[current=page]:font-semibold aria-[current=page]:text-sidenav-accent-foreground data-[state=open]:hover:bg-sidenav-accent data-[state=open]:hover:text-sidenav-accent-foreground group-data-[collapsible=Icon]:size-8! group-data-[collapsible=Icon]:p-2! [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0 hover:bg-sidenav-accent hover:text-sidenav-accent-foreground h-8 text-sm",
                    class.get(),
                )
            }
        >
            {children()}
        </a>
    }
}

/// Anchor row for [`SidenavMenuSub`] entries. Marks the active route with
/// `aria-current="page"` from the live pathname. Native attributes forward to
/// the `<a>`.
#[component]
pub fn SidenavMenuSubButton(
    #[prop(into)] href: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let location = use_location();
    let target = href.clone();
    let aria_current = move || active_path(&target, &location.pathname.get());

    view! {
        <a
            data-name="SidenavMenuSubButton"
            href=href
            aria-current=aria_current
            class=move || {
                cn!(
                    "text-sidenav-foreground ring-sidenav-ring hover:bg-sidenav-accent hover:text-sidenav-accent-foreground active:bg-sidenav-accent active:text-sidenav-accent-foreground flex h-7 min-w-0 -translate-x-px items-center gap-2 overflow-hidden rounded-md px-2 text-sm outline-hidden focus-visible:ring-2 disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 aria-[current=page]:bg-sidenav-accent aria-[current=page]:font-medium aria-[current=page]:text-sidenav-accent-foreground [&>svg]:size-4 [&>svg]:shrink-0",
                    class.get(),
                )
            }
        >
            {children()}
        </a>
    }
}

/// Action button pinned to the right of a [`SidenavMenuItem`]. With
/// `show_on_hover` it stays hidden until the item is hovered or focused.
#[component]
pub fn SidenavMenuAction(
    #[prop(optional)] show_on_hover: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let hover_class = if show_on_hover {
        "group-focus-within/menu-item:opacity-100 group-hover/menu-item:opacity-100 data-[state=open]:opacity-100 peer-data-[active=true]/menu-button:text-sidenav-accent-foreground md:opacity-0"
    } else {
        ""
    };

    view! {
        <button
            type="button"
            data-name="SidenavMenuAction"
            data-sidenav="menu-action"
            class=move || {
                cn!(
                    "text-sidenav-foreground ring-sidenav-ring hover:bg-sidenav-accent hover:text-sidenav-accent-foreground peer-hover/menu-button:text-sidenav-accent-foreground absolute top-1.5 right-1 flex aspect-square w-5 items-center justify-center rounded-md p-0 outline-hidden transition-transform focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0 after:absolute after:-inset-2 md:after:hidden peer-data-[size=Sm]/menu-button:top-1 peer-data-[size=Default]/menu-button:top-1.5 peer-data-[size=Lg]/menu-button:top-2.5 group-data-[collapsible=Icon]:hidden",
                    hover_class,
                    class.get(),
                )
            }
        >
            {children()}
        </button>
    }
}
