use crate::{clx, cn, use_can_scroll_vertical, use_lock_body_scroll, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;
use web_sys::{HtmlElement, Node};

clx! {
    /// Non-interactive heading for a [`SelectGroup`] of options.
    SelectLabel, span, "px-2 py-1.5 text-sm font-medium text-muted-foreground"
}

clx! {
    /// Static, non-selecting row for use inside a [`SelectContent`] popup — a
    /// presentational sibling to [`SelectOption`] for separators or hints.
    SelectItem, div, "inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm text-popover-foreground [&_svg:not([class*='size-'])]:size-4"
}

/// Placement of the [`SelectContent`] popup relative to its trigger.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectPosition {
    #[default]
    Below,
    Above,
}

impl SelectPosition {
    fn as_data(self) -> &'static str {
        match self {
            Self::Below => "below",
            Self::Above => "above",
        }
    }
}

/// Open state, current selection and the wiring that lets the parts cooperate:
/// node refs for outside-click detection and focus return, plus the id base that
/// labels the popup for assistive technology. Provided by [`Select`] to its
/// descendants.
#[derive(Clone, Copy)]
struct SelectContext {
    open: RwSignal<bool>,
    value: RwSignal<Option<String>>,
    on_change: StoredValue<Option<Callback<Option<String>>>>,
    trigger_ref: NodeRef<html::Button>,
    panel_ref: NodeRef<html::Div>,
    id_base: StoredValue<String>,
}

impl SelectContext {
    fn trigger_id(&self) -> String {
        format!("{}-trigger", self.id_base.get_value())
    }

    fn content_id(&self) -> String {
        format!("{}-content", self.id_base.get_value())
    }

    fn select(&self, value: Option<String>) {
        self.value.set(value.clone());
        if let Some(cb) = self.on_change.get_value() {
            cb.run(value);
        }
        self.open.set(false);
    }
}

/// Custom single-select. Owns the open state and selected value (seeded from
/// `default_value`), sharing them with the nested [`SelectTrigger`],
/// [`SelectContent`] and [`SelectOption`]s through context. `on_change` fires
/// with the new value whenever a selection is made.
#[component]
pub fn Select(
    /// Value selected on first render.
    #[prop(into, optional)]
    default_value: Option<String>,
    #[prop(optional)] on_change: Option<Callback<Option<String>>>,
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = SelectContext {
        open: RwSignal::new(false),
        value: RwSignal::new(default_value),
        on_change: StoredValue::new(on_change),
        trigger_ref: NodeRef::new(),
        panel_ref: NodeRef::new(),
        id_base: StoredValue::new(use_random_id_for("select")),
    };

    view! {
        <Provider value=ctx>
            <div
                data-name="Select"
                data-state=move || if ctx.open.get() { "open" } else { "closed" }
                class=move || cn!("relative w-fit", class.get())
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Button that toggles the [`SelectContent`] popup. Carries
/// `aria-haspopup="listbox"` and reflects the open state via `aria-expanded`;
/// receives focus back when the popup closes.
#[component]
pub fn SelectTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();
    let merged = move || {
        cn!(
            "w-full p-2 h-9 inline-flex items-center justify-between text-sm font-medium whitespace-nowrap rounded-md transition-colors border bg-background border-input hover:bg-accent hover:text-accent-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    view! {
        <button
            node_ref=ctx.trigger_ref
            type="button"
            data-name="SelectTrigger"
            id=move || ctx.trigger_id()
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            aria-haspopup="listbox"
            aria-expanded=move || ctx.open.get().to_string()
            aria-controls=move || ctx.content_id()
            class=merged
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
            <Icon
                icon=icondata::LuChevronDown
                attr:class=move || {
                    cn!(
                        "ml-2 text-muted-foreground transition-transform duration-200",
                        if ctx.open.get() { "rotate-180" } else { "" },
                    )
                }
            />
        </button>
    }
}

/// Reflects the current selection inside a [`SelectTrigger`], falling back to
/// `placeholder` when nothing is selected.
#[component]
pub fn SelectValue(#[prop(into, optional)] placeholder: String) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();
    let placeholder = StoredValue::new(placeholder);

    view! {
        <span data-name="SelectValue" class="text-sm truncate">
            {move || ctx.value.get().unwrap_or_else(|| placeholder.get_value())}
        </span>
    }
}

/// Listbox popup holding the [`SelectOption`]s. Rendered inside a [`Show`] while
/// the select is open; focuses itself on open, returns focus to the trigger on
/// close, and dismisses on Escape, an outside click or a selection. Shows
/// top/bottom scroll affordances when the list overflows.
///
/// Placement is set explicitly via `position` rather than measuring viewport
/// space at runtime — a deliberate simplification of the vendored version, which
/// sniffed `getBoundingClientRect` from JavaScript to flip above/below.
#[component]
pub fn SelectContent(
    #[prop(into, optional)] position: Signal<SelectPosition>,
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();
    let children = StoredValue::new(children);
    let (on_scroll, can_scroll_up, can_scroll_down) = use_can_scroll_vertical();

    let lock = use_lock_body_scroll(false);
    Effect::new(move |_| lock.set(ctx.open.get()));

    Effect::new(move |_| {
        if !ctx.open.get() {
            return;
        }
        let keydown = window_event_listener(leptos::ev::keydown, move |ev: KeyboardEvent| {
            if ev.key() == "Escape" {
                ev.prevent_default();
                ctx.open.set(false);
            }
        });
        let pointerdown = window_event_listener(leptos::ev::mousedown, move |ev: MouseEvent| {
            if is_outside(&ev, ctx.panel_ref, ctx.trigger_ref) {
                ctx.open.set(false);
            }
        });
        on_cleanup(move || {
            keydown.remove();
            pointerdown.remove();
        });
    });

    Effect::new(move |was_open: Option<bool>| {
        let open = ctx.open.get();
        if open {
            if let Some(panel) = ctx.panel_ref.get() {
                _ = panel.focus();
            }
        } else if was_open == Some(true)
            && let Some(trigger) = ctx.trigger_ref.get_untracked()
        {
            _ = trigger.focus();
        }
        open
    });

    let merged = Signal::derive(move || {
        cn!(
            "w-[150px] min-w-[8rem] overflow-auto z-50 p-1 rounded-md border bg-card shadow-md h-fit max-h-[300px] absolute left-0 origin-top data-[position=below]:top-[calc(100%+4px)] data-[position=above]:bottom-[calc(100%+4px)] data-[position=above]:origin-bottom outline-none [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
            class.get(),
        )
    });

    view! {
        <Show when=move || ctx.open.get()>
            <div
                node_ref=ctx.panel_ref
                data-name="SelectContent"
                id=move || ctx.content_id()
                role="listbox"
                tabindex="-1"
                aria-labelledby=move || ctx.trigger_id()
                data-state="open"
                data-position=move || position.get().as_data()
                class=merged
                on:scroll=on_scroll.clone()
                on:keydown=move |ev: KeyboardEvent| move_focus(&ev, ctx.panel_ref)
            >
                <Show when=move || can_scroll_up.get()>
                    <div class="sticky -top-1 z-10 flex items-center justify-center py-1 bg-card">
                        <Icon
                            icon=icondata::LuChevronUp
                            attr:class="size-4 text-muted-foreground"
                        />
                    </div>
                </Show>
                {children.get_value()()}
                <Show when=move || can_scroll_down.get()>
                    <div class="sticky -bottom-1 z-10 flex items-center justify-center py-1 bg-card">
                        <Icon
                            icon=icondata::LuChevronDown
                            attr:class="size-4 text-muted-foreground"
                        />
                    </div>
                </Show>
            </div>
        </Show>
    }
}

/// Groups related [`SelectOption`]s under an optional [`SelectLabel`].
#[component]
pub fn SelectGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div data-name="SelectGroup" role="group" class=move || cn!(class.get())>
            {children()}
        </div>
    }
}

/// A selectable row. Sets the [`Select`] value (and fires `on_change`) on click
/// or Enter/Space, then closes the popup. Carries `role="option"` and
/// `aria-selected`; arrow keys move focus between options via the popup's roving
/// handler.
#[component]
pub fn SelectOption(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();
    let value = StoredValue::new(value);
    let is_selected =
        Memo::new(move |_| ctx.value.get().as_deref() == Some(value.get_value().as_str()));
    let merged = move || {
        cn!(
            "group inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm cursor-pointer no-underline transition-colors duration-200 text-popover-foreground hover:bg-accent hover:text-accent-foreground outline-none focus-visible:bg-accent focus-visible:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    view! {
        <div
            data-name="SelectOption"
            role="option"
            tabindex="-1"
            aria-selected=move || is_selected.get().to_string()
            data-state=move || if is_selected.get() { "selected" } else { "unselected" }
            class=merged
            on:click=move |_| ctx.select(Some(value.get_value()))
            on:keydown=move |ev: KeyboardEvent| {
                if matches!(ev.key().as_str(), "Enter" | " ") {
                    ev.prevent_default();
                    ctx.select(Some(value.get_value()));
                }
            }
        >
            {children()}
            <Icon
                icon=icondata::LuCheck
                attr:class=move || {
                    cn!(
                        "ml-auto size-4 text-muted-foreground",
                        if is_selected.get() { "opacity-100" } else { "opacity-0" },
                    )
                }
            />
        </div>
    }
}

/// Reports whether a pointer event landed outside both the popup and its
/// trigger. Runs only inside the pointer handler, so the `web_sys` DOM access
/// never executes during server rendering.
fn is_outside(
    ev: &MouseEvent,
    panel_ref: NodeRef<html::Div>,
    trigger_ref: NodeRef<html::Button>,
) -> bool {
    let Some(target) = ev.target().and_then(|t| t.dyn_into::<Node>().ok()) else {
        return false;
    };
    let in_panel = panel_ref
        .get_untracked()
        .is_some_and(|panel| panel.contains(Some(&target)));
    let in_trigger = trigger_ref
        .get_untracked()
        .is_some_and(|trigger| trigger.contains(Some(&target)));
    !in_panel && !in_trigger
}

/// Roving-focus navigation for the listbox: ArrowUp/Down step between options,
/// Home/End jump to the ends. Runs only inside the keydown handler, so the
/// `web_sys` DOM access never executes during server rendering.
fn move_focus(ev: &KeyboardEvent, panel_ref: NodeRef<html::Div>) {
    let Some(panel) = panel_ref.get_untracked() else {
        return;
    };
    let Ok(options) = panel.query_selector_all("[data-name='SelectOption']") else {
        return;
    };
    let count = options.length();
    if count == 0 {
        return;
    }

    let current = document().active_element().and_then(|el| {
        (0..count).find(|&i| {
            options
                .item(i)
                .is_some_and(|n| &n == el.unchecked_ref::<Node>())
        })
    });

    let target = match ev.key().as_str() {
        "ArrowDown" => Some(current.map_or(0, |i| (i + 1) % count)),
        "ArrowUp" => Some(current.map_or(count - 1, |i| if i == 0 { count - 1 } else { i - 1 })),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => return,
    };
    let Some(target) = target else { return };

    if let Some(el) = options
        .item(target)
        .and_then(|n| n.dyn_into::<HtmlElement>().ok())
    {
        ev.prevent_default();
        _ = el.focus();
    }
}
