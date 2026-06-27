use std::sync::atomic::{AtomicU64, Ordering};

use crate::{clx, cn, use_lock_body_scroll, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;
use web_sys::{Element, HtmlElement};

pub use crate::components::ui::separator::Separator as MenubarSeparator;

clx! {
    /// Logical grouping of items inside a [`MenubarContent`] panel.
    MenubarGroup, ul, "p-0"
}
clx! {
    /// Non-interactive heading labelling the items beneath it.
    MenubarLabel, div, "px-2 py-1.5 text-sm font-medium text-muted-foreground"
}

/// Trailing keyboard hint shown at the end of a [`MenubarItem`].
#[component]
pub fn MenubarShortcut(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            data-name="MenubarShortcut"
            class=move || cn!("ml-auto text-xs tracking-widest text-muted-foreground", class.get())
        >
            {children()}
        </span>
    }
}

/// Bar-wide open state shared from [`Menubar`] to every menu. `active` holds the
/// id of the menu currently open, or `None` when the bar is closed; the
/// `menubar_id` ties triggers and content back to their bar for keyboard
/// navigation across menus.
#[derive(Clone, Copy)]
struct MenubarContext {
    active: RwSignal<Option<u64>>,
    bar_ref: NodeRef<html::Div>,
}

impl MenubarContext {
    fn close(&self) {
        self.active.set(None);
    }
}

/// Horizontal bar of menus with keyboard navigation. Owns the bar-wide open
/// state and shares it with each nested [`MenubarMenu`] via context, so opening
/// one menu closes the others and arrow keys can move between menus. The panels
/// are anchored to their triggers with CSS, so no layout is measured at runtime.
#[component]
pub fn Menubar(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = MenubarContext {
        active: RwSignal::new(None),
        bar_ref: NodeRef::new(),
    };

    let locked = use_lock_body_scroll(false);
    Effect::new(move |_| locked.set(ctx.active.get().is_some()));

    let children = StoredValue::new(children);

    view! {
        <Provider value=ctx>
            <div
                node_ref=ctx.bar_ref
                data-name="Menubar"
                role="menubar"
                data-state=move || if ctx.active.get().is_some() { "active" } else { "idle" }
                class=move || {
                    cn!(
                        "flex h-8 items-center gap-0.5 rounded-lg border bg-background p-[3px]",
                        class.get(),
                    )
                }
            >
                {move || children.read_value()()}
            </div>
        </Provider>
    }
}

/// Per-menu open state plus the wiring that links its trigger and panel. Created
/// by [`MenubarMenu`] and shared with the nested [`MenubarTrigger`] and
/// [`MenubarContent`]; `is_open` derives from the bar's active menu.
#[derive(Clone, Copy)]
struct MenubarMenuContext {
    menu_id: u64,
    content_id: StoredValue<String>,
    trigger_id: StoredValue<String>,
    trigger_ref: NodeRef<html::Button>,
    content_ref: NodeRef<html::Div>,
    bar: MenubarContext,
}

impl MenubarMenuContext {
    fn is_open(&self) -> bool {
        self.bar.active.get() == Some(self.menu_id)
    }

    fn open(&self) {
        self.bar.active.set(Some(self.menu_id));
    }

    fn close_and_refocus(&self) {
        self.bar.close();
        if let Some(trigger) = self.trigger_ref.get_untracked() {
            _ = trigger.focus();
        }
    }
}

/// A single menu within a [`Menubar`]: a trigger and its content panel. Owns its
/// identity within the bar and provides it to the nested [`MenubarTrigger`] and
/// [`MenubarContent`].
#[component]
pub fn MenubarMenu(children: ChildrenFn) -> impl IntoView {
    let bar = expect_context::<MenubarContext>();

    let ctx = MenubarMenuContext {
        menu_id: next_menu_id(),
        content_id: StoredValue::new(use_random_id_for("menubar")),
        trigger_id: StoredValue::new(use_random_id_for("menubar")),
        trigger_ref: NodeRef::new(),
        content_ref: NodeRef::new(),
        bar,
    };

    let children = StoredValue::new(children);

    view! {
        <Provider value=ctx>
            <div data-name="MenubarMenu" class="relative">
                {move || children.read_value()()}
            </div>
        </Provider>
    }
}

/// Button that opens its [`MenubarMenu`]. Carries `aria-haspopup="menu"` and
/// reflects the open state via `aria-expanded`. Clicking toggles the menu; while
/// the bar already has a menu open, pointer-entering this trigger switches the
/// open menu to it, matching native menubar behaviour.
#[component]
pub fn MenubarTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MenubarMenuContext>();

    let merged = move || {
        cn!(
            "flex items-center rounded-sm px-2 py-[2px] text-sm font-medium outline-none select-none cursor-default transition-colors hover:bg-muted aria-expanded:bg-muted focus-visible:ring-1 focus-visible:ring-ring",
            class.get(),
        )
    };

    view! {
        <button
            node_ref=ctx.trigger_ref
            type="button"
            data-name="MenubarTrigger"
            id=move || ctx.trigger_id.get_value()
            aria-haspopup="menu"
            aria-expanded=move || ctx.is_open().to_string()
            aria-controls=move || ctx.content_id.get_value()
            class=merged
            on:click=move |_| {
                if ctx.is_open() {
                    ctx.bar.close();
                } else {
                    ctx.open();
                }
            }
            on:pointerenter=move |_| {
                if ctx.bar.active.get_untracked().is_some() {
                    ctx.open();
                }
            }
        >
            {children()}
        </button>
    }
}

/// Popup panel listing the menu's items. Rendered only while open via [`Show`],
/// it carries `role="menu"` and implements the WAI-ARIA roving-tabindex:
/// ArrowUp/Down move focus between items, Home/End jump to the ends,
/// ArrowLeft/Right move to the previous/next menu in the bar, Escape closes and
/// returns focus to the trigger. A backdrop captures outside clicks.
#[component]
pub fn MenubarContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<MenubarMenuContext>();
    let children = StoredValue::new(children);

    Effect::new(move |_| {
        if !ctx.is_open() {
            return;
        }
        if let Some(panel) = ctx.content_ref.get() {
            match first_menu_item(&panel) {
                Some(first) => _ = first.focus(),
                None => _ = panel.focus(),
            }
        }
    });

    let panel = move || {
        cn!(
            "absolute top-full left-0 mt-1 z-50 p-1 min-w-36 rounded-md border bg-popover text-popover-foreground shadow-md origin-top-left",
            class.get(),
        )
    };

    view! {
        <Show when=move || {
            ctx.is_open()
        }>
            {
                let panel = panel.clone();
                view! {
                    <div
                        aria-hidden="true"
                        class="fixed inset-0 z-40"
                        on:pointerdown=move |_| ctx.close_and_refocus()
                    />
                    <ul
                        node_ref=ctx.content_ref
                        data-name="MenubarContent"
                        id=move || ctx.content_id.get_value()
                        role="menu"
                        tabindex="-1"
                        aria-labelledby=move || ctx.trigger_id.get_value()
                        data-state="open"
                        class=panel
                        on:keydown=move |ev: KeyboardEvent| handle_menu_keys(&ev, ctx)
                    >
                        {move || children.read_value()()}
                    </ul>
                }
            }
        </Show>
    }
}

/// A selectable command in the menu. Carries `role="menuitem"`, participates in
/// the roving-tabindex, and closes the menu when activated unless
/// `close_on_select` is cleared. Provide `href` to render a navigating anchor
/// instead of a button.
#[component]
pub fn MenubarItem(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] href: Option<String>,
    #[prop(default = true)] close_on_select: bool,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MenubarMenuContext>();

    let merged = move || {
        cn!(
            "relative inline-flex gap-1.5 items-center w-full rounded-sm px-1.5 py-1 text-sm cursor-default no-underline transition-colors outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    if let Some(href) = href {
        return view! {
            <li role="none" class="contents">
                <a
                    data-name="MenubarItem"
                    role="menuitem"
                    tabindex="-1"
                    href=href
                    class=merged
                    on:click=move |_| {
                        if close_on_select {
                            ctx.bar.close();
                        }
                    }
                >
                    {children()}
                </a>
            </li>
        }
        .into_any();
    }

    view! {
        <li role="none" class="contents">
            <button
                type="button"
                data-name="MenubarItem"
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
        </li>
    }
    .into_any()
}

/// A toggleable item that shows a check while its bound `checked` signal is true.
/// Carries `role="menuitemcheckbox"` and `aria-checked`, and participates in the
/// roving-tabindex.
#[component]
pub fn MenubarCheckboxItem(
    /// Signal holding this item's checked state.
    checked: RwSignal<bool>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        cn!(
            "group relative inline-flex gap-1.5 items-center w-full rounded-sm pl-7 pr-1.5 py-1 text-sm cursor-default transition-colors outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    view! {
        <li role="none" class="contents">
            <button
                type="button"
                data-name="MenubarCheckboxItem"
                role="menuitemcheckbox"
                tabindex="-1"
                aria-checked=move || checked.get().to_string()
                class=merged
                on:click=move |_| checked.update(|v| *v = !*v)
            >
                <span class="flex absolute left-1.5 justify-center items-center pointer-events-none size-4">
                    <Icon
                        icon=icondata::LuCheck
                        attr:class="opacity-0 group-aria-checked:opacity-100 size-3.5"
                    />
                </span>
                {children()}
            </button>
        </li>
    }
}

/// Selected-value state for a [`MenubarRadioGroup`], shared with its items.
#[derive(Clone)]
struct MenubarRadioContext<T: Clone + PartialEq + Send + Sync + 'static> {
    value: RwSignal<T>,
}

/// A group of radio items where exactly one is selected at a time. The bound
/// `value` signal holds the current selection.
#[component]
pub fn MenubarRadioGroup<T>(
    /// Signal holding the currently selected value.
    value: RwSignal<T>,
    children: Children,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    view! {
        <Provider value=MenubarRadioContext { value }>
            <ul data-name="MenubarRadioGroup" role="group" class="p-0">
                {children()}
            </ul>
        </Provider>
    }
}

/// A radio item that shows a check when its `value` matches the group's
/// selection. Carries `role="menuitemradio"` and `aria-checked`, and
/// participates in the roving-tabindex.
#[component]
pub fn MenubarRadioItem<T>(
    /// Value this item selects when activated.
    value: T,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let group = expect_context::<MenubarRadioContext<T>>();

    let value = StoredValue::new(value);
    let is_selected = move || group.value.with(|v| *v == value.get_value());

    let merged = move || {
        cn!(
            "group relative inline-flex gap-1.5 items-center w-full rounded-sm pl-7 pr-1.5 py-1 text-sm cursor-default transition-colors outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    view! {
        <li role="none" class="contents">
            <button
                type="button"
                data-name="MenubarRadioItem"
                role="menuitemradio"
                tabindex="-1"
                aria-checked=move || is_selected().to_string()
                class=merged
                on:click=move |_| group.value.set(value.get_value())
            >
                <span class="flex absolute left-1.5 justify-center items-center pointer-events-none size-4">
                    <Icon
                        icon=icondata::LuCheck
                        attr:class="opacity-0 group-aria-checked:opacity-100 size-3.5"
                    />
                </span>
                {children()}
            </button>
        </li>
    }
}

/// Open state for a single [`MenubarSub`] flyout, shared with its trigger and
/// content.
#[derive(Clone, Copy)]
struct MenubarSubContext {
    open: RwSignal<bool>,
}

/// A nested submenu inside a [`MenubarContent`]. Owns its own open state, opening
/// the flyout on hover or keyboard focus and providing the state to the nested
/// [`MenubarSubTrigger`] and [`MenubarSubContent`].
#[component]
pub fn MenubarSub(children: ChildrenFn) -> impl IntoView {
    let ctx = MenubarSubContext {
        open: RwSignal::new(false),
    };
    let children = StoredValue::new(children);

    view! {
        <Provider value=ctx>
            <li
                data-name="MenubarSub"
                role="none"
                class="relative"
                on:pointerenter=move |_| ctx.open.set(true)
                on:pointerleave=move |_| ctx.open.set(false)
                on:focusin=move |_| ctx.open.set(true)
                on:focusout=move |_| ctx.open.set(false)
            >
                {move || children.read_value()()}
            </li>
        </Provider>
    }
}

/// Row that opens its [`MenubarSub`] flyout. Carries `role="menuitem"` with
/// `aria-haspopup="menu"`, reflects the flyout state via `aria-expanded`, and
/// shows a trailing chevron.
#[component]
pub fn MenubarSubTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MenubarSubContext>();

    let merged = move || {
        cn!(
            "relative inline-flex gap-1.5 justify-between items-center w-full rounded-sm px-1.5 py-1 text-sm cursor-default no-underline transition-colors outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    view! {
        <button
            type="button"
            data-name="MenubarSubTrigger"
            role="menuitem"
            tabindex="-1"
            aria-haspopup="menu"
            aria-expanded=move || ctx.open.get().to_string()
            class=merged
            on:click=move |_| ctx.open.update(|v| *v = !*v)
        >
            <span class="flex gap-1.5 items-center">{children()}</span>
            <Icon icon=icondata::LuChevronRight attr:class="opacity-70 size-4" />
        </button>
    }
}

/// Flyout panel of a [`MenubarSub`], rendered only while the submenu is open via
/// [`Show`]. Carries `role="menu"` and is positioned to the trailing edge of its
/// trigger with CSS.
#[component]
pub fn MenubarSubContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<MenubarSubContext>();
    let children = StoredValue::new(children);

    let panel = move || {
        cn!(
            "absolute top-[-4px] left-full ml-2 z-[100] p-1 min-w-40 rounded-md border bg-popover text-popover-foreground shadow-lg origin-top-left",
            class.get(),
        )
    };

    view! {
        <Show when=move || {
            ctx.open.get()
        }>
            {
                let panel = panel.clone();
                view! {
                    <ul data-name="MenubarSubContent" role="menu" class=panel>
                        {move || children.read_value()()}
                    </ul>
                }
            }
        </Show>
    }
}

/// A selectable command inside a [`MenubarSubContent`] flyout. Carries
/// `role="menuitem"` and closes the whole menubar when activated.
#[component]
pub fn MenubarSubItem(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] href: Option<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MenubarMenuContext>();

    let merged = move || {
        cn!(
            "inline-flex gap-1.5 items-center w-full rounded-sm px-3 py-2 text-sm no-underline cursor-default transition-all duration-150 outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground hover:translate-x-[2px] [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    if let Some(href) = href {
        return view! {
            <li role="none" class="contents">
                <a
                    data-name="MenubarSubItem"
                    role="menuitem"
                    tabindex="-1"
                    href=href
                    class=merged
                    on:click=move |_| ctx.bar.close()
                >
                    {children()}
                </a>
            </li>
        }
        .into_any();
    }

    view! {
        <li role="none" class="contents">
            <button
                type="button"
                data-name="MenubarSubItem"
                role="menuitem"
                tabindex="-1"
                class=merged
                on:click=move |_| ctx.close_and_refocus()
            >
                {children()}
            </button>
        </li>
    }
    .into_any()
}

/// Routes menu keydowns. ArrowUp/Down and Home/End rove focus within the panel,
/// ArrowLeft/Right step to the previous/next menu in the bar, and Escape closes
/// the menu and restores focus to its trigger. Runs only inside the keydown
/// handler, so `web_sys` DOM access never executes during server rendering.
fn handle_menu_keys(ev: &KeyboardEvent, ctx: MenubarMenuContext) {
    match ev.key().as_str() {
        "Escape" => {
            ev.prevent_default();
            ctx.close_and_refocus();
        }
        "ArrowDown" | "ArrowUp" | "Home" | "End" => {
            if let Some(menu) = ctx.content_ref.get_untracked() {
                move_focus(ev, &menu);
            }
        }
        "ArrowRight" | "ArrowLeft" => {
            ev.prevent_default();
            move_to_sibling_menu(ev, ctx);
        }
        "Tab" => ctx.bar.close(),
        _ => {}
    }
}

/// Opens the previous/next menu in the bar and focuses its trigger, wrapping at
/// the ends. Implements the WAI-ARIA menubar horizontal arrow navigation.
fn move_to_sibling_menu(ev: &KeyboardEvent, ctx: MenubarMenuContext) {
    let Some(bar) = ctx.bar.bar_ref.get_untracked() else {
        return;
    };
    let Ok(triggers) = bar.query_selector_all("[data-name='MenubarTrigger']") else {
        return;
    };
    let count = triggers.length();
    if count == 0 {
        return;
    }

    let trigger_id = ctx.trigger_id.get_value();
    let current = (0..count).find(|&i| {
        triggers
            .item(i)
            .and_then(|node| node.dyn_into::<Element>().ok())
            .and_then(|el| el.get_attribute("id"))
            .is_some_and(|id| id == trigger_id)
    });

    let target = match ev.key().as_str() {
        "ArrowRight" => current.map_or(0, |i| (i + 1) % count),
        "ArrowLeft" => current.map_or(count - 1, |i| if i == 0 { count - 1 } else { i - 1 }),
        _ => return,
    };

    if let Some(el) = triggers
        .item(target)
        .and_then(|node| node.dyn_into::<HtmlElement>().ok())
    {
        _ = el.focus();
        el.click();
    }
}

/// Returns a process-unique identity for a menu, used to track which menu in a
/// bar is open. Distinct within the process is sufficient — the value never
/// reaches the DOM.
fn next_menu_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Returns the first focusable item within a menu panel.
fn first_menu_item(menu: &Element) -> Option<HtmlElement> {
    menu_items(menu)
        .and_then(|items| items.item(0))
        .and_then(|node| node.dyn_into::<HtmlElement>().ok())
}

/// Collects the menu's enabled item nodes in document order.
fn menu_items(menu: &Element) -> Option<web_sys::NodeList> {
    menu.query_selector_all(
        "[role='menuitem']:not([disabled]),[role='menuitemradio']:not([disabled]),[role='menuitemcheckbox']:not([disabled])",
    )
    .ok()
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
