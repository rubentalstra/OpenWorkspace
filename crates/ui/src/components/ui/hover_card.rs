use crate::{cn, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use std::time::Duration;

/// Delay before a pending open or close commits. Closing is deferred so a
/// pointer crossing the gap between trigger and panel has time to reach the
/// panel and cancel the pending close.
const HOVER_DELAY: Duration = Duration::from_millis(150);

/// Edge of the trigger the panel anchors to.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum HoverCardSide {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

impl HoverCardSide {
    /// Absolute-position and slide-origin classes that place the panel on this
    /// side of the trigger wrapper.
    fn placement(self) -> &'static str {
        match self {
            Self::Bottom => "top-full left-1/2 -translate-x-1/2 mt-2 origin-top",
            Self::Top => "bottom-full left-1/2 -translate-x-1/2 mb-2 origin-bottom",
            Self::Left => "right-full top-1/2 -translate-y-1/2 mr-2 origin-right",
            Self::Right => "left-full top-1/2 -translate-y-1/2 ml-2 origin-left",
        }
    }
}

/// Open state plus the timer and id wiring shared from [`HoverCard`] to its
/// [`HoverCardTrigger`] and [`HoverCardContent`].
#[derive(Clone, Copy)]
struct HoverCardContext {
    open: RwSignal<bool>,
    side: HoverCardSide,
    trigger_id: StoredValue<String>,
    content_id: StoredValue<String>,
    timer: StoredValue<Option<TimeoutHandle>>,
}

impl HoverCardContext {
    fn cancel_pending(self) {
        if let Some(handle) = self.timer.try_update_value(Option::take).flatten() {
            handle.clear();
        }
    }

    /// Schedules `target` after [`HOVER_DELAY`], replacing any pending change so
    /// a quick re-entry cancels the opposite transition.
    fn schedule(self, target: bool) {
        self.cancel_pending();
        let open = self.open;
        let timer = self.timer;
        if let Ok(handle) = set_timeout_with_handle(
            move || {
                _ = open.try_set(target);
                timer.set_value(None);
            },
            HOVER_DELAY,
        ) {
            self.timer.set_value(Some(handle));
        }
    }

    fn close_now(self) {
        self.cancel_pending();
        self.open.set(false);
    }
}

/// Rich card revealed when its [`HoverCardTrigger`] is hovered or focused, after
/// a short delay. Owns the open state and the open/close timer and shares them,
/// with the id wiring, to the nested trigger and [`HoverCardContent`].
#[component]
pub fn HoverCard(
    #[prop(into, optional)] side: HoverCardSide,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = HoverCardContext {
        open: RwSignal::new(false),
        side,
        trigger_id: StoredValue::new(use_random_id_for("hovercard")),
        content_id: StoredValue::new(use_random_id_for("hovercard")),
        timer: StoredValue::new(None),
    };

    on_cleanup(move || ctx.cancel_pending());

    view! {
        <Provider value=ctx>
            <div
                data-name="HoverCard"
                data-state=move || if ctx.open.get() { "open" } else { "closed" }
                class=move || cn!("relative inline-block", class.get())
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Element that opens the [`HoverCard`] on hover or focus and closes it on
/// leave or blur. Wires `aria-describedby` to the panel and reflects the open
/// state through `data-state`.
#[component]
pub fn HoverCardTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<HoverCardContext>();

    view! {
        <span
            data-name="HoverCardTrigger"
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            id=move || ctx.trigger_id.get_value()
            aria-describedby=move || ctx.content_id.get_value()
            class=move || cn!("inline-block outline-none", class.get())
            tabindex="0"
            on:mouseenter=move |_| ctx.schedule(true)
            on:mouseleave=move |_| ctx.schedule(false)
            on:focusin=move |_| ctx.schedule(true)
            on:focusout=move |_| ctx.schedule(false)
        >
            {children()}
        </span>
    }
}

/// Floating panel shown while the [`HoverCard`] is open. Hovering it cancels a
/// pending close so the pointer can move from trigger to panel; leaving it
/// schedules the close. Escape dismisses it. Rendered only while open.
#[component]
pub fn HoverCardContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<HoverCardContext>();
    let children = StoredValue::new(children);

    Effect::new(move |_| {
        if !ctx.open.get() {
            return;
        }
        let handle = window_event_listener(leptos::ev::keydown, move |ev: KeyboardEvent| {
            if ev.key() == "Escape" {
                ctx.close_now();
            }
        });
        on_cleanup(move || handle.remove());
    });

    view! {
        <Show when=move || {
            ctx.open.get()
        }>
            {
                let children = children.get_value();
                view! {
                    <div
                        data-name="HoverCardContent"
                        data-state="open"
                        data-side=match ctx.side {
                            HoverCardSide::Top => "top",
                            HoverCardSide::Bottom => "bottom",
                            HoverCardSide::Left => "left",
                            HoverCardSide::Right => "right",
                        }
                        role="dialog"
                        id=move || ctx.content_id.get_value()
                        aria-labelledby=move || ctx.trigger_id.get_value()
                        class=move || {
                            cn!(
                                "absolute z-50 w-64 rounded-lg border bg-card p-4 text-card-foreground shadow-md animate-in fade-in-0 zoom-in-95 duration-150",
                                ctx.side.placement(),
                                class.get(),
                            )
                        }
                        on:mouseenter=move |_| ctx.cancel_pending()
                        on:mouseleave=move |_| ctx.schedule(false)
                    >
                        {children()}
                    </div>
                }
            }
        </Show>
    }
}
