use crate::{clx, cn, use_lock_body_scroll, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;
use web_sys::{Element, HtmlElement};

pub use crate::components::ui::separator::Separator as DropdownMenuSeparator;

clx! {
    /// Non-interactive heading inside a menu, grouping the items beneath it.
    DropdownMenuLabel, div, "px-2 py-1.5 text-sm font-medium text-muted-foreground"
}

/// Where the content panel anchors relative to its [`DropdownMenuTrigger`].
/// Positioning is pure CSS: the panel is absolutely placed inside the
/// `relative` root, so it tracks the trigger without measuring the viewport.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DropdownMenuAlign {
    #[default]
    Start,
    End,
    Center,
}

/// Vertical placement of the content panel relative to its trigger.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DropdownMenuSide {
    #[default]
    Bottom,
    Top,
}

/// Open state plus the wiring that links a menu's trigger, panel and items.
/// Created by [`DropdownMenu`] and shared with every descendant through
/// context, mirroring the accordion/tabs pattern.
#[derive(Clone, Copy)]
struct DropdownMenuContext {
    open: RwSignal<bool>,
    content_id: StoredValue<String>,
    trigger_id: StoredValue<String>,
    trigger_ref: NodeRef<html::Button>,
    content_ref: NodeRef<html::Div>,
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
}

impl DropdownMenuContext {
    fn close_and_refocus(&self) {
        self.open.set(false);
        if let Some(trigger) = self.trigger_ref.get_untracked() {
            _ = trigger.focus();
        }
    }
}

/// Button-triggered menu. Owns the open state and shares it with the nested
/// [`DropdownMenuTrigger`] and [`DropdownMenuContent`] via context. The panel is
/// anchored to the trigger with CSS, so no layout is measured at runtime.
#[component]
pub fn DropdownMenu(
    #[prop(default = DropdownMenuAlign::default())] align: DropdownMenuAlign,
    #[prop(default = DropdownMenuSide::default())] side: DropdownMenuSide,
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = DropdownMenuContext {
        open: RwSignal::new(false),
        content_id: StoredValue::new(use_random_id_for("dropdown")),
        trigger_id: StoredValue::new(use_random_id_for("dropdown")),
        trigger_ref: NodeRef::new(),
        content_ref: NodeRef::new(),
        align,
        side,
    };

    let locked = use_lock_body_scroll(false);
    Effect::new(move |_| locked.set(ctx.open.get()));

    let children = StoredValue::new(children);

    view! {
        <Provider value=ctx>
            <div
                data-name="DropdownMenu"
                data-state=move || if ctx.open.get() { "open" } else { "closed" }
                class=move || cn!("relative inline-block w-fit", class.get())
            >
                {move || children.read_value()()}
            </div>
        </Provider>
    }
}

/// Button that toggles its [`DropdownMenu`]. Carries `aria-haspopup="menu"` and
/// reflects the open state via `aria-expanded`. When `as_child` is set the
/// children render in place of the button — use it when the child is already a
/// button to avoid nesting interactive elements.
#[component]
pub fn DropdownMenuTrigger(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] as_child: bool,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DropdownMenuContext>();

    if as_child {
        return view! {
            <span
                data-name="DropdownMenuTrigger"
                class="contents"
                on:click=move |_| ctx.open.update(|open| *open = !*open)
            >
                {children()}
            </span>
        }
        .into_any();
    }

    let merged = move || {
        cn!(
            "px-4 py-2 h-9 inline-flex justify-center items-center text-sm font-medium whitespace-nowrap rounded-md transition-colors w-fit border bg-background border-input hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&_svg:not([class*='size-'])]:size-4",
            class.get()
        )
    };

    view! {
        <button
            node_ref=ctx.trigger_ref
            type="button"
            data-name="DropdownMenuTrigger"
            id=move || ctx.trigger_id.get_value()
            aria-haspopup="menu"
            aria-expanded=move || ctx.open.get().to_string()
            aria-controls=move || ctx.content_id.get_value()
            class=merged
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
        </button>
    }
    .into_any()
}

/// Popup panel listing the menu items. Rendered only while open via [`Show`],
/// it carries `role="menu"` and implements the WAI-ARIA roving-tabindex: arrow
/// keys move focus between items, Home/End jump to the ends, Escape closes and
/// returns focus to the trigger. A backdrop captures outside clicks.
#[component]
pub fn DropdownMenuContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<DropdownMenuContext>();
    let children = StoredValue::new(children);

    Effect::new(move |_| {
        if let Some(panel_el) = ctx.content_ref.get() {
            match first_menu_item(&panel_el) {
                Some(first) => _ = first.focus(),
                None => _ = panel_el.focus(),
            }
        }
    });

    let panel = move || {
        let align = match ctx.align {
            DropdownMenuAlign::Start => "left-0 origin-top-left",
            DropdownMenuAlign::End => "right-0 origin-top-right",
            DropdownMenuAlign::Center => "left-1/2 -translate-x-1/2 origin-top",
        };
        let side = match ctx.side {
            DropdownMenuSide::Bottom => "top-full mt-1.5",
            DropdownMenuSide::Top => "bottom-full mb-1.5",
        };
        let width = match ctx.align {
            DropdownMenuAlign::Center => "min-w-full",
            _ => "min-w-[12rem]",
        };
        cn!(
            "absolute z-50 p-1 rounded-md border bg-popover text-popover-foreground shadow-md",
            width,
            align,
            side,
            class.get(),
        )
    };

    view! {
        <Show when=move || {
            ctx.open.get()
        }>
            {
                let panel = panel;
                view! {
                    <div
                        aria-hidden="true"
                        class="fixed inset-0 z-40"
                        on:pointerdown=move |_| ctx.close_and_refocus()
                    />
                    <div
                        node_ref=ctx.content_ref
                        data-name="DropdownMenuContent"
                        id=move || ctx.content_id.get_value()
                        role="menu"
                        tabindex="-1"
                        aria-labelledby=move || ctx.trigger_id.get_value()
                        data-state="open"
                        class=panel
                        on:keydown=move |ev: KeyboardEvent| handle_menu_keys(&ev, ctx)
                    >
                        {move || children.read_value()()}
                    </div>
                }
            }
        </Show>
    }
}

/// Logical grouping of menu items under an optional [`DropdownMenuLabel`].
#[component]
pub fn DropdownMenuGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div data-name="DropdownMenuGroup" role="group" class=move || cn!("p-0", class.get())>
            {children()}
        </div>
    }
}

/// Icon + label row for use inside a [`DropdownMenuItem`].
#[component]
pub fn DropdownMenuAction(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            data-name="DropdownMenuAction"
            class=move || cn!("inline-flex flex-1 items-center gap-2", class.get())
        >
            {children()}
        </span>
    }
}

/// Visual style of a [`DropdownMenuItem`].
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DropdownMenuItemVariant {
    #[default]
    Default,
    Destructive,
}

/// A selectable command in the menu. Carries `role="menuitem"`, participates in
/// the roving-tabindex, and closes the menu when activated unless `close_on_select`
/// is cleared. Provide `href` to render a navigating anchor instead of a button.
#[component]
pub fn DropdownMenuItem(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] href: Option<String>,
    #[prop(default = DropdownMenuItemVariant::default())] variant: DropdownMenuItemVariant,
    #[prop(default = true)] close_on_select: bool,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DropdownMenuContext>();

    let merged = move || {
        let tone = match variant {
            DropdownMenuItemVariant::Default => {
                "text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
            }
            DropdownMenuItemVariant::Destructive => {
                "text-destructive hover:bg-destructive/10 hover:text-destructive focus:bg-destructive/10 focus:text-destructive"
            }
        };
        cn!(
            "inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm text-left no-underline cursor-pointer transition-colors outline-none [&_svg:not([class*='size-'])]:size-4",
            tone,
            class.get(),
        )
    };

    if let Some(href) = href {
        return view! {
            <a
                data-name="DropdownMenuItem"
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
            data-name="DropdownMenuItem"
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

/// Open state of a [`DropdownMenuSub`], shared with its trigger and flyout.
#[derive(Clone, Copy)]
struct DropdownMenuSubContext {
    open: RwSignal<bool>,
}

/// A nested submenu inside a [`DropdownMenuContent`]. Owns its own open state and
/// reveals its [`DropdownMenuSubContent`] flyout while hovered (or while the
/// [`DropdownMenuSubTrigger`] is toggled).
#[component]
pub fn DropdownMenuSub(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = RwSignal::new(false);

    view! {
        <Provider value=DropdownMenuSubContext { open }>
            <div
                data-name="DropdownMenuSub"
                class=move || cn!("relative", class.get())
                on:pointerenter=move |_| open.set(true)
                on:pointerleave=move |_| open.set(false)
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Row that opens its enclosing [`DropdownMenuSub`]. Carries a trailing chevron
/// and `aria-haspopup`/`aria-expanded`; toggles the flyout on click and opens it
/// on hover via the parent.
#[component]
pub fn DropdownMenuSubTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let sub = expect_context::<DropdownMenuSubContext>();

    let merged = move || {
        cn!(
            "inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm text-left cursor-pointer transition-colors outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground data-[state=open]:bg-accent data-[state=open]:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    view! {
        <button
            type="button"
            data-name="DropdownMenuSubTrigger"
            role="menuitem"
            tabindex="-1"
            aria-haspopup="menu"
            aria-expanded=move || sub.open.get().to_string()
            data-state=move || if sub.open.get() { "open" } else { "closed" }
            class=merged
            on:click=move |_| sub.open.update(|open| *open = !*open)
        >
            {children()}
            <Icon icon=icondata::LuChevronRight attr:class="ml-auto size-4 text-muted-foreground" />
        </button>
    }
}

/// Flyout panel of a [`DropdownMenuSub`], rendered to the trigger's side while
/// open. Holds [`DropdownMenuItem`]s, which close the whole menu on select.
#[component]
pub fn DropdownMenuSubContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let sub = expect_context::<DropdownMenuSubContext>();
    let children = StoredValue::new(children);

    view! {
        <Show when=move || sub.open.get()>
            <div
                data-name="DropdownMenuSubContent"
                role="menu"
                class=move || {
                    cn!(
                        "absolute top-0 left-full z-50 ml-1 min-w-[11rem] rounded-md border bg-popover p-1 text-popover-foreground shadow-md",
                        class.get(),
                    )
                }
            >
                {move || children.read_value()()}
            </div>
        </Show>
    }
}

/// Selected-value state for a [`DropdownMenuRadioGroup`], shared with its items.
#[derive(Clone)]
struct DropdownMenuRadioContext<T: Clone + PartialEq + Send + Sync + 'static> {
    value: RwSignal<T>,
}

/// A group of radio items where exactly one is selected at a time. The bound
/// `value` signal holds the current selection.
#[component]
pub fn DropdownMenuRadioGroup<T>(
    /// Signal holding the currently selected value.
    value: RwSignal<T>,
    children: Children,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    view! {
        <Provider value=DropdownMenuRadioContext { value }>
            <div data-name="DropdownMenuRadioGroup" role="group" class="p-0">
                {children()}
            </div>
        </Provider>
    }
}

/// A radio item that shows a check when its `value` matches the group's
/// selection. Carries `role="menuitemradio"` and `aria-checked`.
#[component]
pub fn DropdownMenuRadioItem<T>(
    /// Value this item selects when activated.
    value: T,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let ctx = expect_context::<DropdownMenuContext>();
    let group = expect_context::<DropdownMenuRadioContext<T>>();

    let value = StoredValue::new(value);
    let is_selected = move || group.value.with(|v| *v == value.get_value());

    let merged = move || {
        cn!(
            "group inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm text-left cursor-pointer no-underline transition-colors outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground",
            class.get()
        )
    };

    view! {
        <button
            type="button"
            data-name="DropdownMenuRadioItem"
            role="menuitemradio"
            tabindex="-1"
            aria-checked=move || is_selected().to_string()
            class=merged
            on:click=move |_| {
                group.value.set(value.get_value());
                ctx.close_and_refocus();
            }
        >
            {children()}
            <Icon
                icon=icondata::LuCheck
                attr:class="ml-auto opacity-0 size-4 text-muted-foreground group-aria-checked:opacity-100"
            />
        </button>
    }
}

/// A toggleable item that shows a check while its bound `checked` signal is
/// true. Carries `role="menuitemcheckbox"` and `aria-checked`.
#[component]
pub fn DropdownMenuCheckboxItem(
    /// Signal holding this item's checked state.
    checked: RwSignal<bool>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        cn!(
            "group inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm text-left cursor-pointer no-underline transition-colors outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground",
            class.get()
        )
    };

    view! {
        <button
            type="button"
            data-name="DropdownMenuCheckboxItem"
            role="menuitemcheckbox"
            tabindex="-1"
            aria-checked=move || checked.get().to_string()
            class=merged
            on:click=move |_| checked.update(|c| *c = !*c)
        >
            {children()}
            <Icon
                icon=icondata::LuCheck
                attr:class="ml-auto opacity-0 size-4 text-muted-foreground group-aria-checked:opacity-100"
            />
        </button>
    }
}

/// Implements the WAI-ARIA menu roving-tabindex: ArrowUp/Down move focus between
/// items, Home/End jump to the ends, and Escape closes the menu and restores
/// focus to the trigger. Runs only inside the keydown handler, so `web_sys` DOM
/// access never executes during server rendering.
fn handle_menu_keys(ev: &KeyboardEvent, ctx: DropdownMenuContext) {
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
        "Tab" => ctx.open.set(false),
        _ => {}
    }
}

/// Returns the first focusable `role="menuitem*"` element within a panel.
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
