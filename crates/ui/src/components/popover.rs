use crate::hooks::use_dismiss::use_dismiss;
use crate::{cn, slot};
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct PopoverCtx {
    open: RwSignal<bool>,
}

/// Popover — shadcn Base UI `popover`. An anchored, dismissible popup. Controlled
/// via an external `open` signal or uncontrolled via `default_open`. The root wraps
/// trigger + content so outside-click/Escape dismissal works.
#[component]
pub fn Popover(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let root = NodeRef::<leptos::html::Div>::new();
    provide_context(PopoverCtx { open });
    use_dismiss(open, root);
    view! {
        <div
            node_ref=root
            data-slot="popover"
            class=move || cn!("relative inline-block", class.get())
        >
            {children()}
        </div>
    }
}

/// The control that toggles the popover.
#[component]
pub fn PopoverTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<PopoverCtx>();
    view! {
        <button
            type="button"
            data-slot="popover-trigger"
            aria-haspopup="dialog"
            aria-expanded=move || ctx.open.get().to_string()
            class=move || cn!("", class.get())
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
        </button>
    }
}

/// The popover panel; mounted (and enter-animated) while open.
#[component]
pub fn PopoverContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<PopoverCtx>();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                data-slot="popover-content"
                data-open="true"
                data-side="bottom"
                class=move || {
                    cn!(
                        "cn-popover-content cn-popover-content-logical absolute top-full left-1/2 z-50 mt-1 w-72 -translate-x-1/2 origin-(--transform-origin) outline-hidden",
                        class.get(),
                    )
                }
            >
                {children()}
            </div>
        </Show>
    }
}

slot! { PopoverHeader, div, "popover-header", "cn-popover-header" }
slot! { PopoverTitle, div, "popover-title", "cn-popover-title" }
slot! { PopoverDescription, p, "popover-description", "cn-popover-description" }
