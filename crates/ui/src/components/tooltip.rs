use crate::cn;
use crate::hooks::use_anchored_position::use_anchor_rect;
use leptos::ev;
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct TooltipCtx {
    open: RwSignal<bool>,
    anchor: NodeRef<leptos::html::Div>,
}

/// Tooltip placement relative to the trigger, surfaced as `data-side` so the nova
/// layer drives the matching slide-in/arrow utilities.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum TooltipSide {
    /// Above the trigger (shadcn default).
    #[default]
    Top,
    /// Below the trigger.
    Bottom,
    /// Left of the trigger.
    Left,
    /// Right of the trigger.
    Right,
}

impl TooltipSide {
    fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// TooltipProvider — shadcn Base UI `tooltip` provider. A passthrough grouping
/// wrapper; the `delay` prop is accepted for API parity (instant open here).
#[component]
pub fn TooltipProvider(#[prop(default = 0)] delay: i32, children: Children) -> impl IntoView {
    let _ = delay;
    view! { <div data-slot="tooltip-provider">{children()}</div> }
}

/// Tooltip — shadcn Base UI `tooltip` root. Anchors trigger + content and opens on
/// pointer-enter/focus, closing on leave/blur/Escape. Controlled via an external
/// `open` signal or uncontrolled via `default_open`.
#[component]
pub fn Tooltip(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let root = NodeRef::<leptos::html::Div>::new();
    provide_context(TooltipCtx { open, anchor: root });
    let on_key = window_event_listener(ev::keydown, move |event| {
        if open.get_untracked() && event.key() == "Escape" {
            open.set(false);
        }
    });
    on_cleanup(move || on_key.remove());
    view! {
        <div
            node_ref=root
            data-slot="tooltip"
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

/// The element the tooltip is anchored to.
#[component]
pub fn TooltipTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<TooltipCtx>();
    view! {
        <button
            type="button"
            data-slot="tooltip-trigger"
            aria-describedby="tooltip"
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            class=move || cn!("", class.get())
        >
            {children()}
        </button>
    }
}

/// The tooltip panel; mounted (and enter-animated) while open. Renders the
/// directional arrow as a small rotated div.
#[component]
pub fn TooltipContent(
    #[prop(into, optional)] side: Signal<TooltipSide>,
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<TooltipCtx>();
    let rect = use_anchor_rect(ctx.open, ctx.anchor);
    let position = Signal::derive(move || match side.get() {
        TooltipSide::Top => format!(
            "position:fixed;top:{}px;left:{}px;transform:translate(-50%,calc(-100% - 6px));",
            rect.top(),
            rect.center_x(),
        ),
        TooltipSide::Bottom => format!(
            "position:fixed;top:{}px;left:{}px;transform:translate(-50%,6px);",
            rect.bottom(),
            rect.center_x(),
        ),
        TooltipSide::Left => format!(
            "position:fixed;top:{}px;left:{}px;transform:translate(calc(-100% - 6px),-50%);",
            rect.center_y(),
            rect.left(),
        ),
        TooltipSide::Right => format!(
            "position:fixed;top:{}px;left:{}px;transform:translate(6px,-50%);",
            rect.center_y(),
            rect.right(),
        ),
    });
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div class="isolate z-50" style=move || position.get()>
                <div
                    role="tooltip"
                    data-slot="tooltip-content"
                    data-open="true"
                    data-side=move || side.get().as_str()
                    class=move || {
                        cn!(
                            "cn-tooltip-content cn-tooltip-content-logical z-50 w-fit max-w-xs origin-(--transform-origin) bg-foreground text-background",
                            class.get(),
                        )
                    }
                >
                    {children()}
                    <div
                        data-slot="tooltip-arrow"
                        data-side=move || side.get().as_str()
                        class="cn-tooltip-arrow cn-tooltip-arrow-logical z-50 bg-foreground fill-foreground data-[side=bottom]:top-1 data-[side=left]:top-1/2! data-[side=left]:-right-1 data-[side=left]:-translate-y-1/2 data-[side=right]:top-1/2! data-[side=right]:-left-1 data-[side=right]:-translate-y-1/2 data-[side=top]:-bottom-2.5"
                    ></div>
                </div>
            </div>
        </Show>
    }
}
