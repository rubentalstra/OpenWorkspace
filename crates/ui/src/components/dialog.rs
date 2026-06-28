use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::{cn, slot};
use leptos::ev;
use leptos::prelude::*;
use leptos_icons::Icon;

#[derive(Clone, Copy)]
struct DialogCtx {
    open: RwSignal<bool>,
}

/// Dialog — shadcn Base UI `dialog`. A modal overlay. Controlled via an external
/// `open` signal or uncontrolled via `default_open`; closes on backdrop click,
/// Escape, or the close button.
#[component]
pub fn Dialog(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    provide_context(DialogCtx { open });
    let on_key = window_event_listener(ev::keydown, move |event| {
        if open.get_untracked() && event.key() == "Escape" {
            open.set(false);
        }
    });
    on_cleanup(move || on_key.remove());
    view! { <div data-slot="dialog">{children()}</div> }
}

/// Opens the dialog.
#[component]
pub fn DialogTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DialogCtx>();
    view! {
        <button
            type="button"
            data-slot="dialog-trigger"
            class=move || cn!("", class.get())
            on:click=move |_| ctx.open.set(true)
        >
            {children()}
        </button>
    }
}

/// The modal panel + backdrop; mounted (and enter-animated) while open.
#[component]
pub fn DialogContent(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(default = true)] show_close: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<DialogCtx>();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                data-slot="dialog-overlay"
                data-open="true"
                class="cn-dialog-overlay fixed inset-0 isolate z-50"
                on:click=move |_| ctx.open.set(false)
            ></div>
            <div
                data-slot="dialog-content"
                data-open="true"
                role="dialog"
                aria-modal="true"
                class=move || {
                    cn!(
                        "cn-dialog-content fixed top-1/2 left-1/2 z-50 w-full -translate-x-1/2 -translate-y-1/2 outline-none",
                        class.get(),
                    )
                }
            >
                {children()}
                {show_close
                    .then(|| {
                        view! {
                            <Button
                                variant=ButtonVariant::Ghost
                                size=ButtonSize::IconSm
                                class="cn-dialog-close"
                                attr:r#type="button"
                                attr:data-slot="dialog-close"
                                attr:aria-label="Close"
                                on:click=move |_| ctx.open.set(false)
                            >
                                <Icon icon=icondata::LuX />
                                <span class="sr-only">"Close"</span>
                            </Button>
                        }
                    })}
            </div>
        </Show>
    }
}

/// A button that closes the dialog (use inside the footer/content).
#[component]
pub fn DialogClose(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DialogCtx>();
    view! {
        <button
            type="button"
            data-slot="dialog-close"
            class=move || cn!("", class.get())
            on:click=move |_| ctx.open.set(false)
        >
            {children()}
        </button>
    }
}

slot! { DialogHeader, div, "dialog-header", "cn-dialog-header flex flex-col" }
slot! {
    DialogFooter, div, "dialog-footer",
    "cn-dialog-footer flex flex-col-reverse gap-2 sm:flex-row sm:justify-end"
}
slot! { DialogTitle, h2, "dialog-title", "cn-dialog-title cn-font-heading" }
slot! { DialogDescription, p, "dialog-description", "cn-dialog-description" }
