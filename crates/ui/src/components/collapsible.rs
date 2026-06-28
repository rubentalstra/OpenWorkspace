use crate::cn;
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct CollapsibleCtx {
    open: RwSignal<bool>,
}

/// Collapsible — shadcn Base UI `collapsible`. Show/hide a region via its trigger.
/// Controlled with an external `open` signal, or uncontrolled via `default_open`.
#[component]
pub fn Collapsible(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    provide_context(CollapsibleCtx { open });
    view! {
        <div
            data-slot="collapsible"
            data-open=move || open.get().then_some("true")
            class=move || cn!("", class.get())
        >
            {children()}
        </div>
    }
}

/// Toggles the collapsible open/closed.
#[component]
pub fn CollapsibleTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CollapsibleCtx>();
    view! {
        <button
            type="button"
            data-slot="collapsible-trigger"
            aria-expanded=move || ctx.open.get().to_string()
            class=move || cn!("", class.get())
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
        </button>
    }
}

/// The region revealed when open.
#[component]
pub fn CollapsibleContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CollapsibleCtx>();
    let inner = NodeRef::<leptos::html::Div>::new();
    let height = RwSignal::new(0_i32);
    // Measure the always-rendered inner content and transition height 0 ↔ measured.
    Effect::new(move |_| {
        let open = ctx.open.get();
        let measured = inner.get_untracked().map_or(0, |el| el.scroll_height());
        height.set(if open { measured } else { 0 });
    });
    view! {
        <div
            data-slot="collapsible-content"
            data-open=move || ctx.open.get().then_some("true")
            class="overflow-hidden transition-[height] duration-200 ease-out"
            style:height=move || format!("{}px", height.get())
        >
            <div node_ref=inner class=move || cn!("", class.get())>
                {children()}
            </div>
        </div>
    }
}
