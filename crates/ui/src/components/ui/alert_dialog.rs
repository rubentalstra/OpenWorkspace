use crate::{Button, ButtonSize, ButtonVariant, clx, cn, use_lock_body_scroll, use_random_id_for};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

clx! {
    /// Stacks an alert dialog's title and description; centers on mobile and
    /// left-aligns from the `sm` breakpoint up.
    AlertDialogHeader, div, "flex flex-col gap-2 text-center sm:text-left"
}
clx! {
    /// Action row for an alert dialog. Stacks reversed on mobile so the primary
    /// action sits last, and aligns to the trailing edge from `sm` up.
    AlertDialogFooter, footer, "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end"
}
clx! {
    /// Supporting copy beneath an [`AlertDialogTitle`].
    AlertDialogDescription, p, "text-muted-foreground text-sm"
}

/// Open state and id wiring shared from [`AlertDialog`] to its trigger, content,
/// title and dismissing actions through context.
#[derive(Clone, Copy)]
struct AlertDialogContext {
    open: RwSignal<bool>,
    title_id: StoredValue<String>,
    trigger_id: StoredValue<String>,
}

/// Root that groups an alert dialog's trigger and content and owns the open
/// state.
///
/// Pass `open` to drive the state externally (controlled); otherwise an internal
/// signal seeded from `default_open` is used. Locks body scrolling while open
/// and returns focus to the trigger on close. Unlike [`Dialog`](crate::Dialog),
/// the backdrop does not dismiss on click — an alert dialog requires an explicit
/// choice.
#[component]
pub fn AlertDialog(
    /// External signal driving open/closed; when omitted the dialog manages its
    /// own state seeded from `default_open`.
    #[prop(optional)]
    open: Option<RwSignal<bool>>,
    /// Initial open state when uncontrolled.
    #[prop(default = false)]
    default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let ctx = AlertDialogContext {
        open,
        title_id: StoredValue::new(use_random_id_for("alert_dialog_title")),
        trigger_id: StoredValue::new(use_random_id_for("alert_dialog_trigger")),
    };

    let locked = use_lock_body_scroll(default_open);
    Effect::new(move |was_open: Option<bool>| {
        let is_open = open.get();
        locked.set(is_open);
        if was_open == Some(true)
            && !is_open
            && let Some(el) = document().get_element_by_id(&ctx.trigger_id.get_value())
            && let Some(el) = el.dyn_ref::<web_sys::HtmlElement>()
        {
            _ = el.focus();
        }
        is_open
    });

    view! {
        <Provider value=ctx>
            <div
                data-name="AlertDialog"
                data-state=move || if open.get() { "open" } else { "closed" }
                class=move || cn!("w-fit", class.get())
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Button that opens the enclosing [`AlertDialog`]. Defaults to the outline
/// variant and carries the id the panel focuses on close.
#[component]
pub fn AlertDialogTrigger(
    #[prop(into, optional, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AlertDialogContext>();

    view! {
        <Button
            variant=variant
            size=size
            class=class
            attr:id=move || ctx.trigger_id.get_value()
            attr:aria-haspopup="dialog"
            attr:aria-expanded=move || if ctx.open.get() { "true" } else { "false" }
            on:click=move |_| ctx.open.set(true)
        >
            {children()}
        </Button>
    }
}

/// Modal panel and dimming backdrop for an [`AlertDialog`], rendered only while
/// open. Carries `role="alertdialog"` and `aria-modal`, focuses the panel on
/// open and closes on Escape. The backdrop does not dismiss on click.
#[component]
pub fn AlertDialogContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<AlertDialogContext>();
    let panel_ref = NodeRef::<html::Div>::new();

    Effect::new(move |_| {
        if ctx.open.get()
            && let Some(el) = panel_ref.get()
        {
            _ = el.focus();
        }
    });

    let handle = window_event_listener(ev::keydown, move |event| {
        if event.key() == "Escape" && ctx.open.get_untracked() {
            event.prevent_default();
            ctx.open.set(false);
        }
    });
    on_cleanup(move || handle.remove());

    let panel = move || {
        cn!(
            "relative bg-background border rounded-2xl shadow-lg p-6 w-full max-w-[calc(100%-2rem)] max-h-[85vh] fixed top-[50%] left-[50%] translate-x-[-50%] translate-y-[-50%] z-100 flex flex-col gap-4 outline-none",
            class.get()
        )
    };

    view! {
        <Show when=move || ctx.open.get()>
            <div
                data-name="AlertDialogBackdrop"
                aria-hidden="true"
                class="fixed inset-0 z-60 bg-black/50"
            />
            <div
                node_ref=panel_ref
                data-name="AlertDialogContent"
                role="alertdialog"
                aria-modal="true"
                aria-labelledby=move || ctx.title_id.get_value()
                tabindex="-1"
                class=panel
            >
                {children()}
            </div>
        </Show>
    }
}

/// Prominent heading for an alert dialog's content. Carries the id the panel's
/// `aria-labelledby` points at, so the dialog is correctly named.
#[component]
pub fn AlertDialogTitle(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AlertDialogContext>();

    view! {
        <h2
            data-name="AlertDialogTitle"
            id=move || ctx.title_id.get_value()
            class=move || cn!("text-lg leading-none font-semibold", class.get())
        >
            {children()}
        </h2>
    }
}

/// Primary confirming action. Defaults to the solid variant; closes the dialog
/// when clicked so the consumer can run its handler and dismiss in one step.
#[component]
pub fn AlertDialogAction(
    #[prop(into, optional)] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AlertDialogContext>();

    view! {
        <Button variant=variant size=size class=class on:click=move |_| ctx.open.set(false)>
            {children()}
        </Button>
    }
}

/// Dismissing action. Defaults to the outline variant; closes the dialog when
/// clicked.
#[component]
pub fn AlertDialogCancel(
    #[prop(into, optional, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AlertDialogContext>();

    view! {
        <Button variant=variant size=size class=class on:click=move |_| ctx.open.set(false)>
            {children()}
        </Button>
    }
}
