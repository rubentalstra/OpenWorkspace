use crate::{cn, use_random_id};
use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

/// Selected-tab state shared from [`Tabs`] to every descendant trigger and
/// panel. The id base ties a trigger to its panel for assistive technology.
#[derive(Clone, Copy)]
struct TabsContext {
    selected: RwSignal<String>,
    orientation: Signal<TabsOrientation>,
    id_base: StoredValue<String>,
}

impl TabsContext {
    fn trigger_id(&self, value: &str) -> String {
        format!("{}-trigger-{value}", self.id_base.get_value())
    }

    fn content_id(&self, value: &str) -> String {
        format!("{}-content-{value}", self.id_base.get_value())
    }
}

/// Visual treatment of a [`TabsList`], shared with its triggers so the active
/// indicator matches the list chrome.
#[derive(Clone, Copy)]
struct TabsListContext {
    variant: Signal<TabsVariant>,
}

/// Indicator style for a [`TabsList`]: a filled pill (`Default`) or an
/// underline rule (`Line`).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsVariant {
    #[default]
    Default,
    Line,
}

/// Layout axis for a [`Tabs`] group. `Horizontal` stacks the panel below the
/// list; `Vertical` places the list beside the panel and rebinds the arrow keys
/// to the vertical axis.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl TabsOrientation {
    fn as_data(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// Tabbed container. Owns the selected value (seeded from `default_value`) and
/// shares it with the nested [`TabsList`], [`TabsTrigger`]s and [`TabsContent`]
/// panels through context. Native attributes, events and bindings forward to
/// the root element.
#[component]
pub fn Tabs(
    /// Value selected on first render.
    #[prop(into, optional)]
    default_value: String,
    #[prop(into, optional)] orientation: Signal<TabsOrientation>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let ctx = TabsContext {
        selected: RwSignal::new(default_value),
        orientation,
        id_base: StoredValue::new(use_random_id()),
    };
    let merged = move || {
        let axis = match orientation.get() {
            TabsOrientation::Horizontal => "flex-col",
            TabsOrientation::Vertical => "flex-row",
        };
        cn!("group/tabs flex gap-2", axis, class.get())
    };

    view! {
        <Provider value=ctx>
            <div
                node_ref=node_ref
                data-name="Tabs"
                data-orientation=move || orientation.get().as_data()
                class=merged
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Row of [`TabsTrigger`]s. Implements the WAI-ARIA tablist roving-tabindex and
/// arrow-key navigation; the active trigger holds focus, siblings are skipped in
/// the tab order. Native attributes, events and bindings forward to the root.
#[component]
pub fn TabsList(
    #[prop(into, optional)] variant: Signal<TabsVariant>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<TabsContext>();
    provide_context(TabsListContext { variant });

    let merged = move || {
        let chrome = match variant.get() {
            TabsVariant::Default => "bg-muted",
            TabsVariant::Line => "gap-1 bg-transparent rounded-none p-0",
        };
        cn!(
            "group/tabs-list inline-flex w-fit items-center justify-center rounded-lg p-[3px] text-muted-foreground group-data-[orientation=horizontal]/tabs:h-8 group-data-[orientation=vertical]/tabs:h-fit group-data-[orientation=vertical]/tabs:flex-col",
            chrome,
            class.get(),
        )
    };

    let on_keydown = move |ev: KeyboardEvent| {
        if let Some(list) = node_ref.get() {
            move_focus(&ev, &list, ctx.orientation.get());
        }
    };

    view! {
        <div
            node_ref=node_ref
            role="tablist"
            aria-orientation=move || ctx.orientation.get().as_data()
            data-name="TabsList"
            data-variant=move || match variant.get() {
                TabsVariant::Default => "default",
                TabsVariant::Line => "line",
            }
            class=merged
            on:keydown=on_keydown
        >
            {children()}
        </div>
    }
}

/// Tab button selecting the [`TabsContent`] whose `value` matches. Wires
/// `role="tab"`, `aria-selected`, `aria-controls` and the roving `tabindex`.
/// Native attributes, events and bindings forward to the root.
#[component]
pub fn TabsTrigger(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Button>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<TabsContext>();
    let variant = expect_context::<TabsListContext>().variant;

    let value = StoredValue::new(value);
    let is_active = Memo::new(move |_| ctx.selected.get() == value.get_value());

    let merged = move || {
        let active = is_active.get();
        let is_line = variant.get() == TabsVariant::Line;
        let state = if active {
            "text-foreground"
        } else {
            "text-foreground/60 hover:text-foreground"
        };
        let pill = if !is_line && active {
            "bg-background shadow-sm dark:border-input dark:bg-input/30"
        } else {
            ""
        };
        let underline = if is_line && active {
            "after:opacity-100"
        } else {
            "after:opacity-0"
        };
        cn!(
            "relative inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center gap-1.5 rounded-md border border-transparent px-1.5 py-0.5 text-sm font-medium whitespace-nowrap transition-all cursor-pointer select-none outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 group-data-[orientation=vertical]/tabs:w-full group-data-[orientation=vertical]/tabs:justify-start after:absolute after:bg-foreground after:transition-opacity group-data-[orientation=horizontal]/tabs:after:inset-x-0 group-data-[orientation=horizontal]/tabs:after:bottom-[-5px] group-data-[orientation=horizontal]/tabs:after:h-0.5 group-data-[orientation=vertical]/tabs:after:inset-y-0 group-data-[orientation=vertical]/tabs:after:-right-1 group-data-[orientation=vertical]/tabs:after:w-0.5",
            state,
            pill,
            underline,
            class.get(),
        )
    };

    view! {
        <button
            node_ref=node_ref
            type="button"
            role="tab"
            id=move || ctx.trigger_id(&value.get_value())
            aria-selected=move || is_active.get().to_string()
            aria-controls=move || ctx.content_id(&value.get_value())
            tabindex=move || if is_active.get() { "0" } else { "-1" }
            data-name="TabsTrigger"
            data-state=move || if is_active.get() { "active" } else { "inactive" }
            class=merged
            on:click=move |_| ctx.selected.set(value.get_value())
        >
            {children()}
        </button>
    }
}

/// Panel revealed when its `value` matches the selected tab; hidden otherwise.
/// Wires `role="tabpanel"` and `aria-labelledby` back to its trigger. Native
/// attributes, events and bindings forward to the root element.
#[component]
pub fn TabsContent(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<TabsContext>();
    let value = StoredValue::new(value);
    let is_active = Memo::new(move |_| ctx.selected.get() == value.get_value());

    let merged = move || {
        cn!(
            "flex-1 text-sm outline-none group-data-[orientation=horizontal]/tabs:mt-2 group-data-[orientation=vertical]/tabs:ml-4",
            class.get(),
            if is_active.get() { "" } else { "hidden" },
        )
    };

    view! {
        <div
            node_ref=node_ref
            role="tabpanel"
            tabindex="0"
            id=move || ctx.content_id(&value.get_value())
            aria-labelledby=move || ctx.trigger_id(&value.get_value())
            data-name="TabsContent"
            data-state=move || if is_active.get() { "active" } else { "inactive" }
            class=merged
        >
            {children()}
        </div>
    }
}

/// Moves focus between sibling triggers in response to arrow/Home/End keys,
/// implementing the WAI-ARIA tablist roving-tabindex pattern. Runs only inside
/// the keydown handler, so the `web_sys` DOM access never executes during SSR.
fn move_focus(ev: &KeyboardEvent, list: &Element, orientation: TabsOrientation) {
    let (prev, next) = match orientation {
        TabsOrientation::Horizontal => ("ArrowLeft", "ArrowRight"),
        TabsOrientation::Vertical => ("ArrowUp", "ArrowDown"),
    };
    let Ok(triggers) = list.query_selector_all("[data-name='TabsTrigger']:not([disabled])") else {
        return;
    };
    let count = triggers.length();
    if count == 0 {
        return;
    }

    let current = (0..count).find(|&i| {
        triggers
            .item(i)
            .and_then(|n| n.dyn_into::<Element>().ok())
            .and_then(|el| el.get_attribute("tabindex"))
            .is_some_and(|t| t == "0")
    });

    let target = match ev.key().as_str() {
        k if k == prev => current.map(|i| if i == 0 { count - 1 } else { i - 1 }),
        k if k == next => current.map(|i| (i + 1) % count),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => return,
    };
    let Some(target) = target else { return };

    if let Some(el) = triggers
        .item(target)
        .and_then(|n| n.dyn_into::<HtmlElement>().ok())
    {
        ev.prevent_default();
        _ = el.focus();
        el.click();
    }
}
