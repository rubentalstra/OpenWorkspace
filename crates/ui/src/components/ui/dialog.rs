use crate::{Button, ButtonSize, ButtonVariant, clx, cn, use_lock_body_scroll, use_random_id_for};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;

clx! {
    /// Vertical stack for an open dialog's main content; pairs with
    /// [`DialogHeader`] and [`DialogFooter`].
    DialogBody, div, "flex flex-col gap-4"
}
clx! {
    /// Heading block for a dialog. Centers on mobile and left-aligns from the
    /// `sm` breakpoint up.
    DialogHeader, div, "flex flex-col gap-2 text-center sm:text-left"
}
clx! {
    /// Supporting copy beneath a [`DialogTitle`].
    DialogDescription, p, "text-muted-foreground text-sm"
}
clx! {
    /// Action row for a dialog. Stacks reversed on mobile so the primary action
    /// sits last, and aligns to the trailing edge from `sm` up.
    DialogFooter, footer, "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end"
}

/// Open state and id wiring shared from [`Dialog`] to its trigger, content and
/// dismissing actions through context. `title_id` labels the panel for
/// assistive technology; `trigger_id` lets the panel return focus on close.
#[derive(Clone, Copy)]
struct DialogContext {
    open: RwSignal<bool>,
    title_id: StoredValue<String>,
    trigger_id: StoredValue<String>,
}

/// Root that groups a dialog's trigger and content and owns the open state.
///
/// Pass `open` to drive the state externally (controlled); otherwise an
/// internal signal seeded from `default_open` is used. Locks body scrolling
/// while open and returns focus to the trigger on close.
#[component]
pub fn Dialog(
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
    let ctx = DialogContext {
        open,
        title_id: StoredValue::new(use_random_id_for("dialog_title")),
        trigger_id: StoredValue::new(use_random_id_for("dialog_trigger")),
    };

    let locked = use_lock_body_scroll(default_open);
    Effect::new(move |was_open: Option<bool>| {
        let is_open = open.get();
        locked.set(is_open);
        if was_open == Some(true) && !is_open {
            if let Some(el) = document().get_element_by_id(&ctx.trigger_id.get_value()) {
                if let Some(el) = el.dyn_ref::<web_sys::HtmlElement>() {
                    _ = el.focus();
                }
            }
        }
        is_open
    });

    view! {
        <Provider value=ctx>
            <div
                data-name="Dialog"
                data-state=move || if open.get() { "open" } else { "closed" }
                class=move || cn!("w-fit", class.get())
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Button that opens the enclosing [`Dialog`]. Defaults to the outline variant
/// and carries the id the panel focuses on close.
#[component]
pub fn DialogTrigger(
    #[prop(into, optional, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DialogContext>();

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

/// Modal panel and dimming backdrop for a [`Dialog`], rendered only while open.
///
/// Carries `role="dialog"` and `aria-modal`, focuses the panel on open, closes
/// on Escape and (unless `close_on_backdrop_click` is false) on a backdrop
/// click, and renders an optional close button.
#[component]
pub fn DialogContent(
    #[prop(into, optional)] class: Signal<String>,
    /// Render the trailing close button.
    #[prop(default = true)]
    show_close_button: bool,
    /// Dismiss the dialog when the backdrop is clicked.
    #[prop(default = true)]
    close_on_backdrop_click: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<DialogContext>();
    let panel_ref = NodeRef::<html::Div>::new();

    Effect::new(move |_| {
        if ctx.open.get() {
            if let Some(el) = panel_ref.get() {
                _ = el.focus();
            }
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
                data-name="DialogBackdrop"
                aria-hidden="true"
                class="fixed inset-0 z-60 bg-black/50"
                on:click=move |_| {
                    if close_on_backdrop_click {
                        ctx.open.set(false);
                    }
                }
            />

            <div
                node_ref=panel_ref
                data-name="DialogContent"
                role="dialog"
                aria-modal="true"
                aria-labelledby=move || ctx.title_id.get_value()
                tabindex="-1"
                class=panel
            >
                <Show when=move || show_close_button>
                    <button
                        type="button"
                        aria-label="Close dialog"
                        class="absolute top-4 right-4 p-1 rounded-sm outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-ring [&_svg:not([class*='size-'])]:size-4"
                        on:click=move |_| ctx.open.set(false)
                    >
                        <Icon icon=icondata::LuX />
                    </button>
                </Show>

                {children()}
            </div>
        </Show>
    }
}

/// Prominent heading for a dialog's content. Carries the id that the panel's
/// `aria-labelledby` points at, so the dialog is correctly named for assistive
/// technology.
#[component]
pub fn DialogTitle(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DialogContext>();

    view! {
        <h3
            data-name="DialogTitle"
            id=move || ctx.title_id.get_value()
            class=move || cn!("text-lg leading-none font-semibold", class.get())
        >
            {children()}
        </h3>
    }
}

/// Dismissing action for a [`Dialog`]. Defaults to the outline variant; closes
/// the dialog when clicked.
#[component]
pub fn DialogClose(
    #[prop(into, optional, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DialogContext>();

    view! {
        <Button
            variant=variant
            size=size
            class=class
            attr:aria-label="Close dialog"
            on:click=move |_| ctx.open.set(false)
        >
            {children()}
        </Button>
    }
}

/// Confirming action for a [`Dialog`]. Defaults to the solid variant; closes the
/// dialog when clicked so the consumer can run its handler and dismiss in one
/// step.
#[component]
pub fn DialogAction(
    #[prop(into, optional)] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DialogContext>();

    view! {
        <Button variant=variant size=size class=class on:click=move |_| ctx.open.set(false)>
            {children()}
        </Button>
    }
}
