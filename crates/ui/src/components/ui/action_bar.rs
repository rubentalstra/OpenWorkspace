use crate::{cn, void};
use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

void! {
    /// Decorative highlight that frames the active [`ActionBarButton`]. Kept as a
    /// presentational element; the moving "liquid glass" effect the vendored
    /// version drove with JavaScript and CSS anchor positioning is dropped in
    /// favour of the reactive per-button active styling.
    LiquidPointerIndicator, div, "block overflow-hidden absolute w-12 h-20 bg-transparent border border-white pointer-events-none rounded-[2rem]"
}

/// Shared state for an [`ActionBar`]. `selected` holds the value of the active
/// [`ActionBarButton`] so exactly one reads as selected at a time; `roving`
/// holds the value of the button that currently owns the toolbar's single tab
/// stop, implementing the WAI-ARIA toolbar roving-tabindex pattern.
#[derive(Clone, Copy)]
struct ActionBarContext {
    selected: RwSignal<Option<String>>,
    roving: RwSignal<Option<String>>,
}

/// Floating contextual action bar. Owns the selection (seeded from
/// `default_value`) and shares it with the nested [`ActionBarButton`]s through
/// context, so picking a button reactively highlights it without any
/// JavaScript. Carries `role="toolbar"` and implements the WAI-ARIA
/// roving-tabindex pattern: the toolbar is a single tab stop and ArrowLeft /
/// ArrowRight / Home / End move focus between buttons. Native attributes, events
/// and bindings forward to the root.
#[component]
pub fn ActionBar(
    /// Value of the [`ActionBarButton`] selected on first render.
    #[prop(into, optional)]
    default_value: Option<String>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let ctx = ActionBarContext {
        selected: RwSignal::new(default_value.clone()),
        roving: RwSignal::new(default_value),
    };

    view! {
        <Provider value=ctx>
            <div
                node_ref=node_ref
                data-name="ActionBar"
                role="toolbar"
                class=move || {
                    cn!(
                        "flex items-center p-2 rounded-2xl border shadow-lg border-input bg-card",
                        class.get(),
                    )
                }
                on:keydown=move |ev: KeyboardEvent| {
                    if let Some(bar) = node_ref.get() {
                        move_focus(&ev, &bar);
                    }
                }
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Selectable button within an [`ActionBar`]. Clicking it sets the enclosing
/// bar's selection to `value`; the active button reflects its state through
/// `aria-pressed` and `data-state` rather than colour alone. Participates in the
/// toolbar roving tabindex: it owns the tab stop when it is the roving target,
/// claims the stop on focus, and is focusable via the arrow keys. Native
/// attributes, events and bindings forward to the underlying button.
#[component]
pub fn ActionBarButton(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ActionBarContext>();
    let value = StoredValue::new(value);

    if ctx.roving.with_untracked(Option::is_none) {
        ctx.roving.set(Some(value.get_value()));
    }

    let is_active = Memo::new(move |_| ctx.selected.get().as_deref() == Some(&value.get_value()));
    let is_tab_stop =
        Memo::new(move |_| ctx.roving.get().as_deref() == Some(&value.get_value()));

    let merged = move || {
        let state = if is_active.get() {
            "bg-accent text-accent-foreground"
        } else {
            "bg-transparent hover:bg-muted focus-visible:bg-muted"
        };
        cn!(
            "flex relative justify-center items-center mx-1 border-0 cursor-pointer select-none outline-none px-[15px] py-[10px] rounded-[50px] transition-colors duration-300 focus-visible:ring-2 focus-visible:ring-ring [&_svg:not([class*='size-'])]:size-4",
            state,
            class.get(),
        )
    };

    view! {
        <button
            type="button"
            data-name="ActionBarButton"
            data-state=move || if is_active.get() { "active" } else { "inactive" }
            aria-pressed=move || is_active.get().to_string()
            tabindex=move || if is_tab_stop.get() { "0" } else { "-1" }
            class=merged
            on:focus=move |_| ctx.roving.set(Some(value.get_value()))
            on:click=move |_| ctx.selected.set(Some(value.get_value()))
        >
            {children()}
        </button>
    }
}

/// Moves focus between sibling buttons in response to arrow/Home/End keys,
/// implementing the WAI-ARIA toolbar roving-tabindex pattern. Runs only inside
/// the keydown handler, so the `web_sys` DOM access never executes during SSR.
fn move_focus(ev: &KeyboardEvent, bar: &Element) {
    let Ok(buttons) = bar.query_selector_all("[data-name='ActionBarButton']:not([disabled])")
    else {
        return;
    };
    let count = buttons.length();
    if count == 0 {
        return;
    }

    let current = (0..count).find(|&i| {
        buttons
            .item(i)
            .and_then(|n| n.dyn_into::<Element>().ok())
            .and_then(|el| el.get_attribute("tabindex"))
            .is_some_and(|t| t == "0")
    });

    let target = match ev.key().as_str() {
        "ArrowLeft" => current.map(|i| if i == 0 { count - 1 } else { i - 1 }),
        "ArrowRight" => current.map(|i| (i + 1) % count),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => return,
    };
    let Some(target) = target else { return };

    if let Some(el) = buttons
        .item(target)
        .and_then(|n| n.dyn_into::<HtmlElement>().ok())
    {
        ev.prevent_default();
        _ = el.focus();
    }
}
