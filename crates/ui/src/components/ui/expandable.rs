use crate::cn;
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_icons::Icon;

#[derive(Clone, Copy)]
struct ExpandableContext {
    expanded: RwSignal<bool>,
}

fn data_state(expanded: RwSignal<bool>) -> &'static str {
    if expanded.get() {
        "expanded"
    } else {
        "collapsed"
    }
}

/// Surface that morphs from a compact trigger into a larger content panel,
/// sharing its expanded state with [`ExpandableTrigger`] and
/// [`ExpandableContent`] through context.
///
/// Pass `expanded` to drive the state externally (controlled); otherwise an
/// internal signal seeded from `default_expanded` is used. Native attributes,
/// events and bindings forward to the root element.
#[component]
pub fn Expandable(
    /// External signal driving the expanded state; when omitted the region
    /// manages its own state seeded from `default_expanded`.
    #[prop(optional)]
    expanded: Option<RwSignal<bool>>,
    /// Initial expanded state when uncontrolled.
    #[prop(default = false)]
    default_expanded: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let expanded = expanded.unwrap_or_else(|| RwSignal::new(default_expanded));

    view! {
        <Provider value=ExpandableContext { expanded }>
            <div
                data-name="Expandable"
                data-state=move || data_state(expanded)
                class=move || cn!("relative overflow-hidden rounded-lg", class.get())
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Compact surface that expands the enclosing [`Expandable`] when clicked.
/// Hidden once the region is expanded, and reflects the state via
/// `aria-expanded` and `data-state` for styling.
#[component]
pub fn ExpandableTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ExpandableContext>();

    view! {
        <button
            type="button"
            data-name="ExpandableTrigger"
            data-state=move || data_state(ctx.expanded)
            aria-expanded=move || ctx.expanded.get().to_string()
            class=move || {
                cn!(
                    "bg-primary text-primary-foreground transition-opacity duration-200 hover:bg-primary/90 data-[state=expanded]:pointer-events-none data-[state=expanded]:opacity-0",
                    class.get(),
                )
            }
            on:click=move |_| ctx.expanded.set(true)
        >
            {children()}
        </button>
    }
}

/// Presentational wrapper that fades and scales between the trigger and content
/// states of the enclosing [`Expandable`], driven by `data-state`.
#[component]
pub fn ExpandableTransition(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ExpandableContext>();

    view! {
        <div
            data-name="ExpandableTransition"
            data-state=move || data_state(ctx.expanded)
            class=move || {
                cn!(
                    "transition-[transform,opacity] duration-300 ease-[cubic-bezier(0.4,0,0.2,1)] data-[state=collapsed]:scale-[0.55] data-[state=collapsed]:opacity-0 data-[state=expanded]:scale-100 data-[state=expanded]:opacity-100",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// Expanded panel revealed when the enclosing [`Expandable`] is open. Renders a
/// Rust-handled close button that collapses the region back to its trigger.
#[component]
pub fn ExpandableContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<ExpandableContext>();

    view! {
        <Show when=move || ctx.expanded.get()>
            <div
                data-name="ExpandableContent"
                data-state=move || data_state(ctx.expanded)
                class=move || {
                    cn!(
                        "bg-muted relative h-full w-full transition-opacity duration-300 ease-[cubic-bezier(0.4,0,0.2,1)]",
                        class.get(),
                    )
                }
            >
                <button
                    type="button"
                    aria-label="Collapse"
                    class="absolute top-1 right-1 flex items-center justify-center"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        ctx.expanded.set(false);
                    }
                >
                    <Icon icon=icondata::LuX attr:class="size-6" />
                </button>
                {children()}
            </div>
        </Show>
    }
}
