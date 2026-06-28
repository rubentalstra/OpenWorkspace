use crate::cn;
use crate::hooks::use_anchored_position::use_anchor_rect;
use crate::hooks::use_dismiss::use_dismiss;
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct HoverCardCtx {
    open: RwSignal<bool>,
    anchor: NodeRef<leptos::html::Div>,
}

/// HoverCard — shadcn Base UI `hover-card` (PreviewCard primitive). An anchored
/// popup that opens when the trigger is hovered or focused, rather than clicked.
/// Controlled via an external `open` signal or uncontrolled via `default_open`.
/// The root wraps trigger + content so outside-click/Escape dismissal works.
#[component]
pub fn HoverCard(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let root = NodeRef::<leptos::html::Div>::new();
    provide_context(HoverCardCtx { open, anchor: root });
    use_dismiss(open, root);
    view! {
        <div
            node_ref=root
            data-slot="hover-card"
            class=move || cn!("relative inline-block", class.get())
            on:pointerenter=move |_| open.set(true)
            on:pointerleave=move |_| open.set(false)
            on:focusin=move |_| open.set(true)
            on:focusout=move |_| open.set(false)
        >
            {children()}
        </div>
    }
}

/// The element that, on hover or focus, reveals the hover card.
#[component]
pub fn HoverCardTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<HoverCardCtx>();
    view! {
        <a
            data-slot="hover-card-trigger"
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            class=move || cn!("", class.get())
        >
            {children()}
        </a>
    }
}

/// The hover-card panel; mounted (and enter-animated) while open.
#[component]
pub fn HoverCardContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<HoverCardCtx>();
    let position = use_anchor_rect(ctx.open, ctx.anchor).below();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div class="isolate z-50" style=move || position.get()>
                <div
                    data-slot="hover-card-content"
                    data-open="true"
                    data-side="bottom"
                    class=move || {
                        cn!(
                            "cn-hover-card-content cn-hover-card-content-logical z-50 origin-(--transform-origin) outline-hidden",
                            class.get(),
                        )
                    }
                >
                    {children()}
                </div>
            </div>
        </Show>
    }
}
