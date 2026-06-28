use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::{cn, slot};
use leptos::prelude::*;
use leptos_icons::Icon;

/// Shared scroll state for a [`MessageScroller`]: the viewport element plus whether
/// it is currently scrolled to the start/end (driving the jump buttons). The upstream
/// wraps an external scroller package; the structure/classes here are transcribed 1:1
/// and the auto-scroll + button visibility are a pure-Leptos implementation.
#[derive(Clone, Copy)]
struct MessageScrollerCtx {
    viewport: NodeRef<leptos::html::Div>,
    at_start: RwSignal<bool>,
    at_end: RwSignal<bool>,
}

/// MessageScrollerProvider — a passthrough; the [`MessageScroller`] root owns the
/// shared scroll state. Present for API parity with the upstream.
#[component]
pub fn MessageScrollerProvider(children: Children) -> impl IntoView {
    children()
}

/// MessageScroller — a chat transcript container that pins to the latest message.
#[component]
pub fn MessageScroller(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    provide_context(MessageScrollerCtx {
        viewport: NodeRef::new(),
        at_start: RwSignal::new(true),
        at_end: RwSignal::new(true),
    });
    view! {
        <div
            data-slot="message-scroller"
            class=move || {
                cn!(
                    "cn-message-scroller group/message-scroller relative flex size-full min-h-0 flex-col overflow-hidden",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// The scrolling viewport; auto-scrolls to the latest message on mount and tracks
/// whether the user is at the start/end so the jump buttons can show/hide.
#[component]
pub fn MessageScrollerViewport(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MessageScrollerCtx>();
    let measure = move || {
        if let Some(el) = ctx.viewport.get_untracked() {
            let top = f64::from(el.scroll_top());
            let client = f64::from(el.client_height());
            let scroll = f64::from(el.scroll_height());
            ctx.at_start.set(top <= 1.0);
            ctx.at_end.set(top + client >= scroll - 1.0);
        }
    };
    Effect::new(move |_| {
        if let Some(el) = ctx.viewport.get() {
            el.set_scroll_top(el.scroll_height());
        }
        measure();
    });
    view! {
        <div
            node_ref=ctx.viewport
            data-slot="message-scroller-viewport"
            class=move || {
                cn!(
                    "cn-message-scroller-viewport size-full min-h-0 min-w-0 scroll-fade-b scrollbar-thin scrollbar-gutter-stable overflow-y-auto overscroll-contain contain-content data-autoscrolling:scrollbar-none",
                    class.get(),
                )
            }
            on:scroll=move |_| measure()
        >
            {children()}
        </div>
    }
}

slot! {
    MessageScrollerContent, div, "message-scroller-content",
    "cn-message-scroller-content flex h-max min-h-full flex-col"
}
slot! {
    MessageScrollerItem, div, "message-scroller-item",
    "cn-message-scroller-item min-w-0 shrink-0 [contain-intrinsic-size:auto_10rem] [content-visibility:auto]"
}

const MESSAGE_SCROLLER_BUTTON_CLASS: &str = "cn-message-scroller-button absolute inset-s-1/2 -translate-x-1/2 border-border bg-background text-foreground transition-[translate,scale,opacity] duration-200 hover:bg-muted hover:text-foreground data-[active=false]:pointer-events-none data-[active=false]:scale-95 data-[active=false]:opacity-0 data-[active=false]:duration-400 data-[active=false]:ease-[cubic-bezier(0.7,0,0.84,0)] data-[active=true]:translate-y-0 data-[active=true]:scale-100 data-[active=true]:opacity-100 data-[active=true]:ease-[cubic-bezier(0.23,1,0.32,1)] data-[direction=end]:bottom-4 data-[direction=end]:data-[active=false]:translate-y-full data-[direction=start]:top-4 data-[direction=start]:data-[active=false]:-translate-y-full rtl:translate-x-1/2 data-[direction=start]:[&_svg]:rotate-180";

/// A floating jump button; `direction="end"` jumps to the latest message (shown only
/// when the user has scrolled up), `direction="start"` jumps to the top.
#[component]
pub fn MessageScrollerButton(
    #[prop(into, optional, default = String::from("end"))] direction: String,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let ctx = expect_context::<MessageScrollerCtx>();
    let to_end = direction == "end";
    let label = if to_end {
        "Scroll to end"
    } else {
        "Scroll to start"
    };
    let active = move || {
        if to_end {
            !ctx.at_end.get()
        } else {
            !ctx.at_start.get()
        }
    };
    view! {
        <Button
            variant=ButtonVariant::Secondary
            size=ButtonSize::IconSm
            class=Signal::derive(move || cn!(MESSAGE_SCROLLER_BUTTON_CLASS, class.get()))
            attr:data-slot="message-scroller-button"
            attr:data-direction=direction
            attr:data-active=move || active().to_string()
            on:click=move |_| {
                if let Some(el) = ctx.viewport.get_untracked() {
                    el.set_scroll_top(if to_end { el.scroll_height() } else { 0 });
                }
            }
        >
            <Icon icon=icondata::LuArrowDown />
            <span class="sr-only">{label}</span>
        </Button>
    }
}
