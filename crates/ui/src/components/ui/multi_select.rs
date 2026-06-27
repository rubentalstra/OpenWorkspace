use std::collections::HashSet;

use crate::{cn, use_lock_body_scroll, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;
use web_sys::{Element, HtmlElement, Node};

pub use crate::components::ui::select::{
    SelectGroup as MultiSelectGroup, SelectItem as MultiSelectItem, SelectLabel as MultiSelectLabel,
};

/// Horizontal placement of a [`MultiSelectContent`] popup relative to its
/// trigger. The panel is absolutely positioned within the [`MultiSelect`] root,
/// so the alignment maps to a fixed edge.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum MultiSelectAlign {
    /// Left edges aligned.
    Start,
    /// Horizontally centered.
    #[default]
    Center,
    /// Right edges aligned.
    End,
}

impl MultiSelectAlign {
    fn placement(self) -> &'static str {
        match self {
            Self::Start => "left-0 origin-top-left",
            Self::Center => "left-1/2 -translate-x-1/2 origin-top",
            Self::End => "right-0 origin-top-right",
        }
    }
}

/// Shared multi-select state: the open flag, the set of currently selected
/// values, the popup alignment, the id wiring that labels the listbox, and node
/// refs used for outside-click detection and focus return. Provided by
/// [`MultiSelect`] to its descendant parts through context.
#[derive(Clone, Copy)]
struct MultiSelectContext {
    open: RwSignal<bool>,
    values: RwSignal<HashSet<String>>,
    align: Signal<MultiSelectAlign>,
    listbox_id: StoredValue<String>,
    trigger_ref: NodeRef<html::Button>,
    panel_ref: NodeRef<html::Div>,
}

/// Multi-selection dropdown built on the [`select`](crate) parts. Owns the open
/// state and the selected-value set and shares them with the nested
/// [`MultiSelectTrigger`] and [`MultiSelectContent`] through context; the trigger
/// toggles the popup and each [`MultiSelectOption`] toggles its membership in the
/// set, so several options stay selected at once.
///
/// Pass `values` to drive the selection externally (controlled); otherwise an
/// internal set is used.
#[expect(
    clippy::implicit_hasher,
    reason = "the public component prop uses the standard hasher"
)]
#[component]
pub fn MultiSelect(
    /// External signal holding the selected values; when omitted the component
    /// manages its own empty set.
    #[prop(optional)]
    values: Option<RwSignal<HashSet<String>>>,
    #[prop(into, optional)] align: Signal<MultiSelectAlign>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = MultiSelectContext {
        open: RwSignal::new(false),
        values: values.unwrap_or_else(|| RwSignal::new(HashSet::new())),
        align,
        listbox_id: StoredValue::new(use_random_id_for("multi_select")),
        trigger_ref: NodeRef::new(),
        panel_ref: NodeRef::new(),
    };

    view! {
        <Provider value=ctx>
            <div
                data-name="MultiSelect"
                data-state=move || if ctx.open.get() { "open" } else { "closed" }
                class=move || cn!("relative w-fit", class.get())
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Summary of the current selection for use inside a [`MultiSelectTrigger`].
/// Shows the placeholder when nothing is selected and a count otherwise.
#[component]
pub fn MultiSelectValue(#[prop(into, optional)] placeholder: String) -> impl IntoView {
    let ctx = expect_context::<MultiSelectContext>();
    let placeholder = StoredValue::new(placeholder);

    view! {
        <span data-name="MultiSelectValue" class="text-sm text-muted-foreground truncate">
            {move || {
                let count = ctx.values.with(HashSet::len);
                if count == 0 { placeholder.get_value() } else { format!("{count} selected") }
            }}
        </span>
    }
}

/// Button that toggles the enclosing [`MultiSelect`] popup. Reflects the open
/// state via `aria-expanded`, advertises the listbox through
/// `aria-haspopup="listbox"`, and carries the ref the panel returns focus to on
/// close.
#[component]
pub fn MultiSelectTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MultiSelectContext>();
    let merged = move || {
        cn!(
            "w-full p-2 h-9 inline-flex items-center justify-between text-sm font-medium whitespace-nowrap rounded-md transition-colors outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&_svg:not(:last-child)]:mr-2 [&_svg:not(:first-child)]:ml-2 [&_svg:not([class*='size-'])]:size-4 border bg-background border-input hover:bg-accent hover:text-accent-foreground",
            class.get(),
        )
    };

    view! {
        <button
            node_ref=ctx.trigger_ref
            type="button"
            data-name="MultiSelectTrigger"
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            aria-haspopup="listbox"
            aria-expanded=move || ctx.open.get().to_string()
            aria-controls=move || ctx.listbox_id.get_value()
            class=merged
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
            <Icon icon=icondata::LuChevronDown attr:class="text-muted-foreground" />
        </button>
    }
}

/// Listbox popup revealed while the enclosing [`MultiSelect`] is open. Renders
/// inside a [`Show`], carries `role="listbox"` with `aria-multiselectable`,
/// locks body scrolling, focuses the first option on open, returns focus to the
/// trigger on close, dismisses on Escape or a pointer press outside the panel and
/// trigger, and moves focus between options with the arrow/Home/End keys.
#[component]
pub fn MultiSelectContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<MultiSelectContext>();
    let children = StoredValue::new(children);

    let locked = use_lock_body_scroll(false);

    Effect::new(move |was_open: Option<bool>| {
        let open = ctx.open.get();
        locked.set(open);
        if open {
            if let Some(panel) = ctx.panel_ref.get() {
                focus_first_option(&panel);
            }
        } else if was_open == Some(true)
            && let Some(trigger) = ctx.trigger_ref.get_untracked()
        {
            _ = trigger.focus();
        }
        open
    });

    Effect::new(move |_| {
        if !ctx.open.get() {
            return;
        }
        let keydown = window_event_listener(leptos::ev::keydown, move |ev: KeyboardEvent| {
            if ev.key() == "Escape" {
                ev.prevent_default();
                ctx.open.set(false);
            } else if let Some(panel) = ctx.panel_ref.get_untracked() {
                move_focus(&ev, &panel);
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

    let merged = move || {
        cn!(
            "w-[150px] min-w-full overflow-auto z-50 p-1 rounded-md border bg-card shadow-md h-fit max-h-[300px] absolute top-[calc(100%+4px)] outline-none [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
            ctx.align.get().placement(),
            class.get(),
        )
    };

    view! {
        <Show when=move || ctx.open.get()>
            <div
                node_ref=ctx.panel_ref
                data-name="MultiSelectContent"
                id=move || ctx.listbox_id.get_value()
                role="listbox"
                aria-multiselectable="true"
                tabindex="-1"
                class=merged
            >
                {children.get_value()()}
            </div>
        </Show>
    }
}

/// Selectable row inside a [`MultiSelectContent`]. Toggles its `value` in the
/// shared set when clicked, reflects membership via `aria-selected` (and the
/// trailing check), and participates in the listbox roving tabindex so the arrow
/// keys can step through it.
#[component]
pub fn MultiSelectOption(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MultiSelectContext>();
    let value = StoredValue::new(value);
    let is_selected = Memo::new(move |_| ctx.values.with(|set| set.contains(&value.get_value())));

    let merged = move || {
        cn!(
            "group inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm text-left cursor-pointer transition-colors duration-200 outline-none text-popover-foreground hover:bg-accent hover:text-accent-foreground focus-visible:bg-accent focus-visible:text-accent-foreground disabled:cursor-not-allowed disabled:opacity-50 [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    view! {
        <button
            type="button"
            data-name="MultiSelectOption"
            role="option"
            aria-selected=move || is_selected.get().to_string()
            class=merged
            on:click=move |ev: MouseEvent| {
                ev.prevent_default();
                ctx.values
                    .update(|set| {
                        let val = value.get_value();
                        if !set.remove(&val) {
                            set.insert(val);
                        }
                    });
            }
        >
            {children()}
            <Icon
                icon=icondata::LuCheck
                attr:class="ml-auto opacity-0 size-4 text-muted-foreground group-aria-selected:opacity-100"
            />
        </button>
    }
}

/// Focuses the first selectable option in the listbox panel. Runs only inside an
/// effect on open, so the `web_sys` DOM access never executes during server
/// rendering.
fn focus_first_option(panel: &Element) {
    let Ok(options) = panel.query_selector_all("[data-name='MultiSelectOption']:not([disabled])")
    else {
        return;
    };
    if let Some(el) = options
        .item(0)
        .and_then(|n| n.dyn_into::<HtmlElement>().ok())
    {
        _ = el.focus();
    }
}

/// Moves focus between sibling options in response to arrow/Home/End keys,
/// implementing the WAI-ARIA listbox roving pattern. Runs only inside the keydown
/// handler, so the `web_sys` DOM access never executes during server rendering.
fn move_focus(ev: &KeyboardEvent, panel: &Element) {
    let Ok(options) = panel.query_selector_all("[data-name='MultiSelectOption']:not([disabled])")
    else {
        return;
    };
    let count = options.length();
    if count == 0 {
        return;
    }

    let active = document().active_element();
    let current = (0..count).find(|&i| {
        options
            .item(i)
            .and_then(|n| n.dyn_into::<Element>().ok())
            .zip(active.clone())
            .is_some_and(|(el, focused)| el == focused)
    });

    let target = match ev.key().as_str() {
        "ArrowUp" => current.map(|i| if i == 0 { count - 1 } else { i - 1 }),
        "ArrowDown" => current.map_or(Some(0), |i| Some((i + 1) % count)),
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

/// Reports whether a pointer event landed outside both the listbox panel and its
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
