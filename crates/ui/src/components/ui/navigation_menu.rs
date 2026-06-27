use crate::{cn, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;
use std::time::Duration;
use web_sys::{Element, HtmlElement, Node};

/// Delay before a pending close commits, so a pointer crossing the gap between a
/// trigger and its panel can reach the panel and cancel the close.
const HOVER_DELAY: Duration = Duration::from_millis(150);

/// Shared trigger chrome. Exposed so a [`NavigationMenuLink`] can adopt the same
/// look as a [`NavigationMenuTrigger`] when it stands in for one.
pub fn navigation_menu_trigger_style() -> &'static str {
    "group inline-flex h-9 w-max items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground focus:outline-none disabled:pointer-events-none disabled:opacity-50 data-[state=open]:bg-accent/50"
}

/// Open state for the whole menu, plus the close timer, the root node used to
/// detect outside clicks, and the trigger to refocus when the menu closes via
/// the keyboard. A single item is open at a time, keyed by its id; `None` means
/// the menu is closed. Provided by [`NavigationMenu`] to every descendant
/// trigger and content panel.
#[derive(Clone, Copy)]
struct NavigationMenuContext {
    active: RwSignal<Option<String>>,
    timer: StoredValue<Option<TimeoutHandle>>,
    root_ref: NodeRef<html::Nav>,
    active_trigger: StoredValue<Option<NodeRef<html::Button>>>,
    pending_focus: RwSignal<bool>,
}

impl NavigationMenuContext {
    fn cancel_pending(self) {
        if let Some(handle) = self.timer.try_update_value(Option::take).flatten() {
            handle.clear();
        }
    }

    fn open(self, id: String, trigger: NodeRef<html::Button>) {
        self.cancel_pending();
        self.active_trigger.set_value(Some(trigger));
        self.active.set(Some(id));
    }

    /// Opens a panel and requests that its first link take focus once the panel
    /// mounts. Used for keyboard intent (ArrowDown), where hover-style opening
    /// must not steal focus.
    fn open_with_focus(self, id: String, trigger: NodeRef<html::Button>) {
        self.pending_focus.set(true);
        self.open(id, trigger);
    }

    /// Schedules the menu to close after [`HOVER_DELAY`]; a re-entry that calls
    /// [`Self::open`] or [`Self::cancel_pending`] first cancels it.
    fn schedule_close(self) {
        self.cancel_pending();
        let active = self.active;
        let timer = self.timer;
        if let Ok(handle) = set_timeout_with_handle(
            move || {
                _ = active.try_set(None);
                timer.set_value(None);
            },
            HOVER_DELAY,
        ) {
            self.timer.set_value(Some(handle));
        }
    }

    fn close_now(self) {
        self.cancel_pending();
        self.active.set(None);
    }

    /// Closes the menu and returns focus to the trigger that opened it, so a
    /// keyboard user is not stranded after dismissing the panel.
    fn close_and_refocus(self) {
        if let Some(trigger) = self
            .active_trigger
            .get_value()
            .and_then(|node| node.get_untracked())
        {
            _ = trigger.focus();
        }
        self.close_now();
    }

    fn is_open(self, id: &str) -> bool {
        self.active.get().as_deref() == Some(id)
    }
}

/// Per-item id wiring, shared from [`NavigationMenuItem`] to its trigger and
/// content so they can drive and read the one shared open state, and so Escape
/// can restore focus to the trigger.
#[derive(Clone, Copy)]
struct NavigationMenuItemContext {
    item_id: StoredValue<String>,
    trigger_ref: NodeRef<html::Button>,
    content_ref: NodeRef<html::Div>,
}

/// Root navigation bar. Owns the shared open state and the close timer and
/// provides them, with the root node ref, to the nested list, items, triggers
/// and content panels. Content panels are absolutely positioned relative to this
/// element, giving every panel the same anchor below the bar. Hovering a trigger
/// opens its panel; leaving the bar, pressing Escape, or clicking outside closes.
#[component]
pub fn NavigationMenu(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = NavigationMenuContext {
        active: RwSignal::new(None),
        timer: StoredValue::new(None),
        root_ref: NodeRef::new(),
        active_trigger: StoredValue::new(None),
        pending_focus: RwSignal::new(false),
    };

    on_cleanup(move || ctx.cancel_pending());

    Effect::new(move |_| {
        if ctx.active.get().is_none() {
            return;
        }
        let keydown = window_event_listener(leptos::ev::keydown, move |ev: KeyboardEvent| {
            if ev.key() == "Escape" {
                ev.prevent_default();
                ctx.close_and_refocus();
            }
        });
        let pointerdown = window_event_listener(leptos::ev::mousedown, move |ev: MouseEvent| {
            if is_outside(&ev, ctx.root_ref) {
                ctx.close_now();
            }
        });
        on_cleanup(move || {
            keydown.remove();
            pointerdown.remove();
        });
    });

    view! {
        <Provider value=ctx>
            <nav
                node_ref=ctx.root_ref
                data-name="NavigationMenu"
                data-state=move || if ctx.active.get().is_some() { "open" } else { "closed" }
                class=move || {
                    cn!(
                        "relative z-10 flex max-w-max flex-1 items-center justify-center",
                        class.get(),
                    )
                }
                on:mouseleave=move |_| ctx.schedule_close()
            >
                {children()}
            </nav>
        </Provider>
    }
}

/// Horizontal list of [`NavigationMenuItem`]s.
#[component]
pub fn NavigationMenuList(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <ul
            data-name="NavigationMenuList"
            class=move || {
                cn!("group flex flex-1 list-none items-center justify-center gap-1", class.get())
            }
        >
            {children()}
        </ul>
    }
}

/// A single menu entry. Mints the id shared by its [`NavigationMenuTrigger`] and
/// [`NavigationMenuContent`], and the node refs that tie them together for focus
/// management. Intentionally not positioned, so the absolutely positioned content
/// escapes to the [`NavigationMenu`] root and every panel shares one anchor below
/// the bar.
#[component]
pub fn NavigationMenuItem(children: Children) -> impl IntoView {
    let item_ctx = NavigationMenuItemContext {
        item_id: StoredValue::new(use_random_id_for("navitem")),
        trigger_ref: NodeRef::new(),
        content_ref: NodeRef::new(),
    };

    view! {
        <Provider value=item_ctx>
            <li data-name="NavigationMenuItem">{children()}</li>
        </Provider>
    }
}

/// Button that opens its item's [`NavigationMenuContent`] on hover or focus.
/// Reflects the open state through `data-state`, `aria-expanded` and a rotating
/// chevron, wires `aria-haspopup`/`aria-controls` to the panel, and opens into
/// the panel on `ArrowDown` for keyboard users.
#[component]
pub fn NavigationMenuTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<NavigationMenuContext>();
    let item = expect_context::<NavigationMenuItemContext>();
    let id = item.item_id;
    let trigger_ref = item.trigger_ref;
    let is_open = move || ctx.is_open(&id.get_value());

    view! {
        <button
            node_ref=trigger_ref
            type="button"
            data-name="NavigationMenuTrigger"
            data-state=move || if is_open() { "open" } else { "closed" }
            aria-haspopup="menu"
            aria-expanded=move || is_open().to_string()
            aria-controls=move || id.get_value()
            class=move || {
                cn!(
                    navigation_menu_trigger_style(),
                    "cursor-default select-none",
                    class.get(),
                )
            }
            on:mouseenter=move |_| ctx.open(id.get_value(), trigger_ref)
            on:focusin=move |_| ctx.open(id.get_value(), trigger_ref)
            on:click=move |_| {
                if is_open() {
                    ctx.close_now();
                } else {
                    ctx.open(id.get_value(), trigger_ref);
                }
            }
            on:keydown=move |ev: KeyboardEvent| {
                match ev.key().as_str() {
                    "ArrowDown" => {
                        ev.prevent_default();
                        ctx.open(id.get_value(), trigger_ref);
                        if let Some(panel) = item.content_ref.get_untracked()
                            && let Some(first) = first_menu_item(&panel)
                        {
                            _ = first.focus();
                        }
                    }
                    "Escape" => ctx.close_now(),
                    _ => {}
                }
            }
        >
            {children()}
            <Icon
                icon=icondata::LuChevronDown
                attr:class="relative ml-1 transition duration-300 top-[1px] size-3 group-data-[state=open]:rotate-180"
            />
        </button>
    }
}

/// Panel revealed while its item is open. Rendered only when open. Absolutely
/// positioned relative to the [`NavigationMenu`] root so all panels share one
/// anchor below the bar. Carries `role="menu"` and implements the WAI-ARIA
/// roving focus: ArrowUp/Down move between links, Home/End jump to the ends, and
/// Escape closes the panel and returns focus to the trigger. Hovering it cancels
/// the pending close so the pointer can travel from trigger to panel; leaving the
/// bar schedules the close.
#[component]
pub fn NavigationMenuContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<NavigationMenuContext>();
    let item = expect_context::<NavigationMenuItemContext>();
    let id = item.item_id;
    let content_ref = item.content_ref;
    let children = StoredValue::new(children);

    view! {
        <Show when=move || ctx.is_open(&id.get_value())>
            <div
                node_ref=content_ref
                data-name="NavigationMenuContent"
                data-state="open"
                id=move || id.get_value()
                role="menu"
                tabindex="-1"
                class=move || {
                    cn!(
                        "absolute left-0 top-full mt-1.5 z-50 w-full rounded-md border bg-popover p-4 shadow-md md:w-auto animate-in fade-in-0 zoom-in-95 duration-200",
                        class.get(),
                    )
                }
                on:mouseenter=move |_| ctx.cancel_pending()
                on:keydown=move |ev: KeyboardEvent| handle_menu_keys(&ev, ctx, content_ref)
            >
                {children.get_value()()}
            </div>
        </Show>
    }
}

/// Navigation link styled to sit inside a content panel. Carries `role="menuitem"`
/// so it participates in the panel's menu semantics and keyboard navigation, and
/// `tabindex="-1"` so the roving focus owns the tab order within the panel.
#[component]
pub fn NavigationMenuLink(
    #[prop(into, optional)] href: Signal<String>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <a
            data-name="NavigationMenuLink"
            role="menuitem"
            tabindex="-1"
            href=move || href.get()
            class=move || {
                cn!(
                    "inline-flex items-center rounded-sm text-sm font-medium transition-colors hover:text-foreground text-foreground/70 focus:outline-none",
                    class.get(),
                )
            }
        >
            {children()}
        </a>
    }
}

/// Routes keydowns inside an open panel. ArrowUp/Down and Home/End rove focus
/// among the panel's links, and Escape closes the panel and restores focus to
/// the trigger. Runs only inside the keydown handler, so the `web_sys` DOM access
/// never executes during server rendering.
fn handle_menu_keys(
    ev: &KeyboardEvent,
    ctx: NavigationMenuContext,
    content_ref: NodeRef<html::Div>,
) {
    match ev.key().as_str() {
        "Escape" => {
            ev.prevent_default();
            ctx.close_and_refocus();
        }
        "ArrowDown" | "ArrowUp" | "Home" | "End" => {
            if let Some(panel) = content_ref.get_untracked() {
                move_focus(ev, &panel);
            }
        }
        _ => {}
    }
}

/// Returns the first focusable `role="menuitem"` element within a panel.
fn first_menu_item(panel: &Element) -> Option<HtmlElement> {
    menu_items(panel)
        .and_then(|items| items.item(0))
        .and_then(|node| node.dyn_into::<HtmlElement>().ok())
}

/// Collects the panel's enabled menu-item nodes in document order.
fn menu_items(panel: &Element) -> Option<web_sys::NodeList> {
    panel
        .query_selector_all("[role='menuitem']:not([aria-disabled='true'])")
        .ok()
}

/// Moves DOM focus among the panel's links in response to arrow/Home/End keys,
/// wrapping at the ends.
fn move_focus(ev: &KeyboardEvent, panel: &Element) {
    let Some(items) = menu_items(panel) else {
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

/// Reports whether a pointer event landed outside the navigation root. Runs only
/// inside the pointer handler, so the `web_sys` DOM access never executes during
/// server rendering.
fn is_outside(ev: &MouseEvent, root_ref: NodeRef<html::Nav>) -> bool {
    let Some(target) = ev.target().and_then(|t| t.dyn_into::<Node>().ok()) else {
        return false;
    };
    !root_ref
        .get_untracked()
        .is_some_and(|root| root.contains(Some(&target)))
}
