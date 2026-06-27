use crate::{clx, cn, use_lock_body_scroll, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;
use web_sys::{Element, HtmlElement};

clx! {
    /// Non-interactive heading inside a context menu, grouping the items beneath it.
    ContextMenuLabel, span, "px-2 py-1.5 text-sm font-medium text-muted-foreground"
}

clx! {
    /// Logical grouping of context-menu items under an optional [`ContextMenuLabel`].
    ContextMenuGroup, ul, "list-none p-0 m-0"
}

clx! {
    /// Anchor row styled to sit inside a [`ContextMenuContent`] panel.
    ContextMenuLink, a, "w-full inline-flex gap-2 items-center no-underline"
}

/// Open state, pointer anchor and the wiring that links a context menu's
/// trigger, panel and items. Created by [`ContextMenu`] and shared with every
/// descendant through context, mirroring the dropdown-menu/tabs pattern.
#[derive(Clone, Copy)]
struct ContextMenuContext {
    open: RwSignal<bool>,
    point: RwSignal<(i32, i32)>,
    content_id: StoredValue<String>,
    trigger_ref: NodeRef<html::Div>,
    content_ref: NodeRef<html::Div>,
    on_close: StoredValue<Option<Callback<()>>>,
}

impl ContextMenuContext {
    fn open_at(&self, x: i32, y: i32) {
        self.point.set((x, y));
        self.open.set(true);
    }

    fn close_and_refocus(&self) {
        if !self.open.get_untracked() {
            return;
        }
        self.open.set(false);
        if let Some(cb) = self.on_close.get_value() {
            cb.run(());
        }
        if let Some(trigger) = self.trigger_ref.get_untracked() {
            _ = trigger.focus();
        }
    }
}

/// Right-click menu. Owns the open state plus the pointer anchor and shares them
/// with the nested [`ContextMenuTrigger`] and [`ContextMenuContent`] via context.
#[component]
pub fn ContextMenu(children: ChildrenFn) -> impl IntoView {
    let ctx = ContextMenuContext {
        open: RwSignal::new(false),
        point: RwSignal::new((0, 0)),
        content_id: StoredValue::new(use_random_id_for("context")),
        trigger_ref: NodeRef::new(),
        content_ref: NodeRef::new(),
        on_close: StoredValue::new(None),
    };

    let locked = use_lock_body_scroll(false);
    Effect::new(move |_| locked.set(ctx.open.get()));

    let children = StoredValue::new(children);

    view! {
        <Provider value=ctx>
            <div
                data-name="ContextMenu"
                data-state=move || if ctx.open.get() { "open" } else { "closed" }
                class="contents"
            >
                {move || children.read_value()()}
            </div>
        </Provider>
    }
}

/// Region whose right-click opens the enclosing [`ContextMenu`]. Captures the
/// pointer coordinates so the panel anchors under the cursor. `on_open` fires
/// when the menu opens.
#[component]
pub fn ContextMenuTrigger(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] on_open: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuContext>();

    view! {
        <div
            node_ref=ctx.trigger_ref
            data-name="ContextMenuTrigger"
            tabindex="-1"
            class=move || cn!("contents", class.get())
            on:contextmenu=move |ev: MouseEvent| {
                ev.prevent_default();
                ctx.open_at(ev.client_x(), ev.client_y());
                if let Some(cb) = on_open {
                    cb.run(());
                }
            }
        >
            {children()}
        </div>
    }
}

/// Popup panel listing the menu items. Rendered only while open via [`Show`] and
/// fixed-positioned at the pointer anchor, clamped to stay within the viewport.
/// Carries `role="menu"` and implements the WAI-ARIA roving-tabindex: arrow keys
/// move focus, Home/End jump to the ends, Escape closes and returns focus to the
/// trigger. A backdrop captures outside clicks and right-clicks. `on_close` fires
/// on any dismissal.
#[component]
pub fn ContextMenuContent(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] on_close: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuContext>();
    ctx.on_close.set_value(on_close);
    let children = StoredValue::new(children);

    let position = RwSignal::new((0.0_f64, 0.0_f64));

    Effect::new(move |_| {
        if !ctx.open.get() {
            return;
        }
        let (px, py) = ctx.point.get();
        let mut x = f64::from(px);
        let mut y = f64::from(py);
        if let Some(panel) = ctx.content_ref.get() {
            let rect = panel.get_bounding_client_rect();
            let viewport_width = window().inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
            let viewport_height = window().inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
            if x + rect.width() > viewport_width {
                x = (x - rect.width()).max(0.0);
            }
            if y + rect.height() > viewport_height {
                y = (y - rect.height()).max(0.0);
            }
            match first_menu_item(&panel) {
                Some(first) => _ = first.focus(),
                None => _ = panel.focus(),
            }
        }
        position.set((x, y));
    });

    let panel = move || {
        cn!(
            "fixed z-50 p-1 rounded-md border bg-popover text-popover-foreground shadow-md min-w-[12rem] origin-top-left",
            class.get(),
        )
    };
    let style = move || {
        let (x, y) = position.get();
        format!("left: {x}px; top: {y}px;")
    };

    view! {
        <Show when=move || {
            ctx.open.get()
        }>
            {
                let panel = panel.clone();
                let style = style.clone();
                view! {
                    <div
                        aria-hidden="true"
                        class="fixed inset-0 z-40"
                        on:pointerdown=move |_| ctx.close_and_refocus()
                        on:contextmenu=move |ev: MouseEvent| {
                            ev.prevent_default();
                            ctx.close_and_refocus();
                        }
                    />
                    <div
                        node_ref=ctx.content_ref
                        data-name="ContextMenuContent"
                        id=move || ctx.content_id.get_value()
                        role="menu"
                        tabindex="-1"
                        data-state="open"
                        class=panel
                        style=style
                        on:keydown=move |ev: KeyboardEvent| handle_menu_keys(&ev, ctx)
                    >
                        {move || children.read_value()()}
                    </div>
                }
            }
        </Show>
    }
}

/// A selectable command in the context menu. Carries `role="menuitem"`,
/// participates in the roving-tabindex, and closes the menu when activated unless
/// `close_on_select` is cleared. Provide `href` to render a navigating anchor
/// instead of a button.
#[component]
pub fn ContextMenuItem(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] href: Option<String>,
    #[prop(default = true)] close_on_select: bool,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuContext>();

    let merged = move || {
        cn!(
            "inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm text-left no-underline cursor-pointer transition-colors outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    if let Some(href) = href {
        return view! {
            <a
                data-name="ContextMenuItem"
                role="menuitem"
                tabindex="-1"
                href=href
                class=merged
                on:click=move |_| {
                    if close_on_select {
                        ctx.open.set(false);
                    }
                }
            >
                {children()}
            </a>
        }
        .into_any();
    }

    view! {
        <button
            type="button"
            data-name="ContextMenuItem"
            role="menuitem"
            tabindex="-1"
            class=merged
            on:click=move |_| {
                if close_on_select {
                    ctx.close_and_refocus();
                }
            }
        >
            {children()}
        </button>
    }
    .into_any()
}

/// A selectable command identical to [`ContextMenuItem`] in behaviour, kept as a
/// distinct name for call-site clarity. Closes the menu when activated unless
/// `close_on_select` is cleared; provide `href` to render a navigating anchor.
/// Forward any extra ARIA from the call site (e.g. `attr:aria-selected`).
#[component]
pub fn ContextMenuAction(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] href: Option<String>,
    #[prop(default = true)] close_on_select: bool,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuContext>();

    let merged = move || {
        cn!(
            "inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm text-left no-underline cursor-pointer transition-colors outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    if let Some(href) = href {
        return view! {
            <a
                data-name="ContextMenuAction"
                role="menuitem"
                tabindex="-1"
                href=href
                class=merged
                on:click=move |_| {
                    if close_on_select {
                        ctx.open.set(false);
                    }
                }
            >
                {children()}
            </a>
        }
        .into_any();
    }

    view! {
        <button
            type="button"
            data-name="ContextMenuAction"
            role="menuitem"
            tabindex="-1"
            class=merged
            on:click=move |_| {
                if close_on_select {
                    ctx.close_and_refocus();
                }
            }
        >
            {children()}
        </button>
    }
    .into_any()
}

/// A nested submenu host. Hovering the row reveals its [`ContextMenuSubContent`]
/// via CSS, so the flyout needs no runtime measurement.
#[component]
pub fn ContextMenuSub(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <li
            data-name="ContextMenuSub"
            class=move || {
                cn!(
                    "group/sub relative inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm no-underline cursor-pointer transition-colors text-popover-foreground hover:bg-accent hover:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
                    class.get(),
                )
            }
        >
            {children()}
        </li>
    }
}

/// Row that opens its [`ContextMenuSub`] on hover. Renders a trailing chevron and
/// carries `aria-haspopup="menu"`.
#[component]
pub fn ContextMenuSubTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            data-name="ContextMenuSubTrigger"
            role="menuitem"
            tabindex="-1"
            aria-haspopup="menu"
            class=move || cn!("flex items-center justify-between w-full", class.get())
        >
            <span class="flex gap-2 items-center">{children()}</span>
            <Icon icon=icondata::LuChevronRight attr:class="opacity-70 size-4" />
        </span>
    }
}

/// Flyout panel revealed when its [`ContextMenuSub`] row is hovered. Carries
/// `role="menu"`; its visibility is driven by the parent's `group/sub` hover
/// state, so it animates in without JavaScript.
#[component]
pub fn ContextMenuSubContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <ul
            data-name="ContextMenuSubContent"
            role="menu"
            class=move || {
                cn!(
                    "list-none m-0 absolute z-[100] left-full top-[-0.25rem] ml-2 min-w-[10rem] p-1 rounded-md border bg-popover text-popover-foreground shadow-lg opacity-0 invisible -translate-x-2 transition-all duration-200 ease-out pointer-events-none group-hover/sub:opacity-100 group-hover/sub:visible group-hover/sub:translate-x-0 group-hover/sub:pointer-events-auto",
                    class.get(),
                )
            }
        >
            {children()}
        </ul>
    }
}

/// A selectable command inside a [`ContextMenuSubContent`]. Carries
/// `role="menuitem"` and closes the whole menu when activated.
#[component]
pub fn ContextMenuSubItem(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuContext>();

    view! {
        <li
            data-name="ContextMenuSubItem"
            role="menuitem"
            tabindex="-1"
            class=move || {
                cn!(
                    "inline-flex gap-2 items-center w-full rounded-sm px-3 py-2 text-sm cursor-pointer transition-all duration-150 text-popover-foreground hover:bg-accent hover:text-accent-foreground hover:translate-x-0.5 [&_svg:not([class*='size-'])]:size-4",
                    class.get(),
                )
            }
            on:click=move |_| ctx.close_and_refocus()
        >
            {children()}
        </li>
    }
}

/// Implements the WAI-ARIA menu roving-tabindex: ArrowUp/Down move focus between
/// items, Home/End jump to the ends, and Escape closes the menu and restores
/// focus to the trigger. Runs only inside the keydown handler, so `web_sys` DOM
/// access never executes during server rendering.
fn handle_menu_keys(ev: &KeyboardEvent, ctx: ContextMenuContext) {
    match ev.key().as_str() {
        "Escape" => {
            ev.prevent_default();
            ctx.close_and_refocus();
        }
        "ArrowDown" | "ArrowUp" | "Home" | "End" => {
            let Some(menu) = ctx.content_ref.get_untracked() else {
                return;
            };
            move_focus(ev, &menu);
        }
        "Tab" => ctx.close_and_refocus(),
        _ => {}
    }
}

/// Returns the first focusable `role="menuitem"` element within a panel.
fn first_menu_item(menu: &Element) -> Option<HtmlElement> {
    menu_items(menu)
        .and_then(|items| items.item(0))
        .and_then(|node| node.dyn_into::<HtmlElement>().ok())
}

/// Collects the menu's enabled item nodes in document order.
fn menu_items(menu: &Element) -> Option<web_sys::NodeList> {
    menu.query_selector_all("[role='menuitem']:not([disabled])").ok()
}

/// Moves DOM focus among the menu's items in response to arrow/Home/End keys,
/// wrapping at the ends.
fn move_focus(ev: &KeyboardEvent, menu: &Element) {
    let Some(items) = menu_items(menu) else {
        return;
    };
    let count = items.length();
    if count == 0 {
        return;
    }

    let active = document().active_element();
    let current = (0..count).find(|&i| {
        items
            .item(i)
            .and_then(|node| node.dyn_into::<Element>().ok())
            .zip(active.clone())
            .is_some_and(|(item, focused)| item == focused)
    });

    let target = match ev.key().as_str() {
        "ArrowDown" => current.map_or(0, |i| (i + 1) % count),
        "ArrowUp" => current.map_or(count - 1, |i| if i == 0 { count - 1 } else { i - 1 }),
        "Home" => 0,
        "End" => count - 1,
        _ => return,
    };

    if let Some(el) = items
        .item(target)
        .and_then(|node| node.dyn_into::<HtmlElement>().ok())
    {
        ev.prevent_default();
        _ = el.focus();
    }
}
