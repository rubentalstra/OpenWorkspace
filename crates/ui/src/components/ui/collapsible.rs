use crate::cn;
use leptos::context::Provider;
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct CollapsibleContext {
    open: RwSignal<bool>,
}

fn data_state(open: RwSignal<bool>) -> &'static str {
    if open.get() { "open" } else { "closed" }
}

/// Disclosure region whose open state is shared with its
/// [`CollapsibleTrigger`] and [`CollapsibleContent`] through context.
///
/// Pass `open` to drive the state externally (controlled); otherwise an
/// internal signal seeded from `default_open` is used. Native attributes,
/// events and bindings forward to the root element.
#[component]
pub fn Collapsible(
    /// External signal driving open/closed; when omitted the region manages
    /// its own state seeded from `default_open`.
    #[prop(optional)]
    open: Option<RwSignal<bool>>,
    /// Initial open state when uncontrolled.
    #[prop(default = false)]
    default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));

    view! {
        <Provider value=CollapsibleContext { open }>
            <div
                data-name="Collapsible"
                data-state=move || data_state(open)
                class=move || cn!(class.get())
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Button that toggles the enclosing [`Collapsible`]. Reflects the region
/// state via `aria-expanded` and `data-state` for styling.
#[component]
pub fn CollapsibleTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CollapsibleContext>();

    view! {
        <button
            type="button"
            data-name="CollapsibleTrigger"
            data-state=move || data_state(ctx.open)
            aria-expanded=move || ctx.open.get().to_string()
            class=move || cn!(class.get())
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
        </button>
    }
}

/// Animated panel revealed when the enclosing [`Collapsible`] is open, using
/// the CSS grid-rows reveal so height animates without JavaScript. `class`
/// styles the inner content (padding, gap); `outer_class` targets the
/// animated grid wrapper (e.g. `col-span-full`).
#[component]
pub fn CollapsibleContent(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] outer_class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CollapsibleContext>();

    view! {
        <div
            data-name="CollapsibleContent"
            data-state=move || data_state(ctx.open)
            class=move || {
                cn!(
                    "grid overflow-hidden transition-all duration-300 data-[state=closed]:grid-rows-[0fr] data-[state=open]:grid-rows-[1fr]",
                    outer_class.get(),
                )
            }
        >
            <div class=move || cn!("min-h-0", class.get())>{children()}</div>
        </div>
    }
}
