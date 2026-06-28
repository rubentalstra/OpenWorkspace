use crate::components::button::{Button, ButtonVariant};
use crate::{cn, slot};
use leptos::ev;
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct AlertDialogCtx {
    open: RwSignal<bool>,
}

/// AlertDialog — shadcn Base UI `alert-dialog`. A modal that interrupts the user for
/// a confirmation. Unlike [`Dialog`](crate::Dialog) it has no close affordance and is
/// not dismissed by a backdrop click — the user must pick an action; Escape cancels.
/// Controlled via an external `open` signal or uncontrolled via `default_open`.
#[component]
pub fn AlertDialog(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    provide_context(AlertDialogCtx { open });
    let on_key = window_event_listener(ev::keydown, move |event| {
        if open.get_untracked() && event.key() == "Escape" {
            open.set(false);
        }
    });
    on_cleanup(move || on_key.remove());
    view! { <div data-slot="alert-dialog">{children()}</div> }
}

/// Opens the alert dialog.
#[component]
pub fn AlertDialogTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AlertDialogCtx>();
    view! {
        <button
            type="button"
            data-slot="alert-dialog-trigger"
            class=move || cn!("", class.get())
            on:click=move |_| ctx.open.set(true)
        >
            {children()}
        </button>
    }
}

/// Alert-dialog sizing, surfaced as `data-size` for the nova layer.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AlertDialogSize {
    /// The standard width.
    #[default]
    Default,
    /// A compact two-column-footer variant.
    Sm,
}

impl AlertDialogSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
        }
    }
}

/// The modal panel + backdrop; mounted (and enter-animated) while open. The backdrop
/// does not dismiss — the user must choose an action.
#[component]
pub fn AlertDialogContent(
    #[prop(into, optional)] size: Signal<AlertDialogSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<AlertDialogCtx>();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                data-slot="alert-dialog-overlay"
                data-open="true"
                class="cn-alert-dialog-overlay fixed inset-0 isolate z-50"
            ></div>
            <div
                role="alertdialog"
                aria-modal="true"
                data-slot="alert-dialog-content"
                data-open="true"
                data-size=move || size.get().as_str()
                class=move || {
                    cn!(
                        "cn-alert-dialog-content group/alert-dialog-content fixed top-1/2 left-1/2 z-50 grid w-full -translate-x-1/2 -translate-y-1/2 outline-none",
                        class.get(),
                    )
                }
            >
                {children()}
            </div>
        </Show>
    }
}

slot! { AlertDialogHeader, div, "alert-dialog-header", "cn-alert-dialog-header" }
slot! {
    AlertDialogFooter, div, "alert-dialog-footer",
    "cn-alert-dialog-footer flex flex-col-reverse gap-2 group-data-[size=sm]/alert-dialog-content:grid group-data-[size=sm]/alert-dialog-content:grid-cols-2 sm:flex-row sm:justify-end"
}
slot! { AlertDialogMedia, div, "alert-dialog-media", "cn-alert-dialog-media" }
slot! { AlertDialogTitle, h2, "alert-dialog-title", "cn-alert-dialog-title cn-font-heading" }
slot! { AlertDialogDescription, p, "alert-dialog-description", "cn-alert-dialog-description" }

/// The primary action button; runs `on_click` (if set) then closes the dialog.
#[component]
pub fn AlertDialogAction(
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AlertDialogCtx>();
    view! {
        <Button
            class=Signal::derive(move || cn!("cn-alert-dialog-action", class.get()))
            attr:r#type="button"
            attr:data-slot="alert-dialog-action"
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(());
                }
                ctx.open.set(false);
            }
        >
            {children()}
        </Button>
    }
}

/// The cancel button (outline); closes the dialog without taking an action.
#[component]
pub fn AlertDialogCancel(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AlertDialogCtx>();
    view! {
        <Button
            variant=ButtonVariant::Outline
            class=Signal::derive(move || cn!("cn-alert-dialog-cancel", class.get()))
            attr:r#type="button"
            attr:data-slot="alert-dialog-cancel"
            on:click=move |_| ctx.open.set(false)
        >
            {children()}
        </Button>
    }
}
