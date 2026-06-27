use crate::{cn, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::keydown;
use leptos::prelude::*;

/// Placement of a [`TooltipContent`] panel relative to its trigger.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipPosition {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
}

impl TooltipPosition {
    fn as_data(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Left => "left",
            Self::Right => "right",
            Self::Bottom => "bottom",
        }
    }

    fn content_class(self) -> &'static str {
        match self {
            Self::Top => "left-1/2 bottom-full mb-1 -ml-2.5",
            Self::Right => "bottom-1/2 left-full ml-2.5 -mb-3.5",
            Self::Bottom => "left-1/2 top-full mt-1 -ml-2.5",
            Self::Left => "bottom-1/2 right-full mr-2.5 -mb-3.5",
        }
    }

    fn arrow_class(self) -> &'static str {
        match self {
            Self::Top => "left-1/2 bottom-full -mb-2 border-t-foreground/90",
            Self::Right => "bottom-1/2 left-full -mr-0.5 -mb-1 border-r-foreground/90",
            Self::Bottom => "left-1/2 top-full -mt-2 border-b-foreground/90",
            Self::Left => "bottom-1/2 right-full -mb-1 -ml-0.5 border-l-foreground/90",
        }
    }
}

/// Visibility state shared from [`Tooltip`] to its trigger and content. The id
/// ties the trigger's `aria-describedby` to the content panel for assistive
/// technology.
#[derive(Clone, Copy)]
struct TooltipContext {
    open: RwSignal<bool>,
    content_id: StoredValue<String>,
}

/// Hover/focus tooltip. Owns the visibility state shared with its
/// [`TooltipTrigger`] and [`TooltipContent`]. The tooltip shows while the
/// trigger is hovered or focused and hides on leave, blur or Escape.
#[component]
pub fn Tooltip(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = TooltipContext {
        open: RwSignal::new(false),
        content_id: StoredValue::new(use_random_id_for("tooltip")),
    };

    let merged = move || {
        cn!(
            "inline-block relative mx-0 whitespace-nowrap my-[5px]",
            class.get()
        )
    };

    view! {
        <Provider value=ctx>
            <div
                data-name="Tooltip"
                data-state=move || if ctx.open.get() { "open" } else { "closed" }
                class=merged
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Element that reveals the tooltip while hovered or focused. Reflects the
/// described panel via `aria-describedby` so screen readers announce it.
#[component]
pub fn TooltipTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<TooltipContext>();

    view! {
        <div
            data-name="TooltipTrigger"
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            aria-describedby=move || ctx.content_id.get_value()
            class=move || cn!("inline-flex w-fit", class.get())
            on:pointerenter=move |_| ctx.open.set(true)
            on:pointerleave=move |_| ctx.open.set(false)
            on:focusin=move |_| ctx.open.set(true)
            on:focusout=move |_| ctx.open.set(false)
        >
            {children()}
        </div>
    }
}

/// Floating panel shown while the [`Tooltip`] is open. Carries `role="tooltip"`
/// and an arrow oriented by `position`; Escape hides the tooltip while it is
/// shown. Rendered only when open, so its `children` run under [`Show`].
#[component]
pub fn TooltipContent(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] position: Signal<TooltipPosition>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<TooltipContext>();
    let children = StoredValue::new(children);

    Effect::new(move |_| {
        if !ctx.open.get() {
            return;
        }
        let handle = window_event_listener(keydown, move |ev| {
            if ev.key() == "Escape" {
                ctx.open.set(false);
            }
        });
        on_cleanup(move || handle.remove());
    });

    view! {
        <Show when=move || {
            ctx.open.get()
        }>
            {
                let position = position.get();
                let arrow = cn!(
                    "absolute pointer-events-none z-[1000000] bg-transparent border-transparent border-6",
                    position.arrow_class(),
                );
                let panel = cn!(
                    "absolute pointer-events-none z-[1000000] py-2 px-2.5 text-xs whitespace-nowrap shadow-lg text-background bg-foreground/90",
                    position.content_class(),
                    class.get(),
                );
                view! {
                    <div
                        data-name="TooltipArrow"
                        data-position=position.as_data()
                        aria-hidden="true"
                        class=arrow
                    />
                    <div
                        data-name="TooltipContent"
                        data-position=position.as_data()
                        role="tooltip"
                        id=ctx.content_id.get_value()
                        class=panel
                    >
                        {children.read_value()()}
                    </div>
                }
            }
        </Show>
    }
}
