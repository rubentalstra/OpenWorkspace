use crate::{Button, ButtonSize, ButtonVariant, cn, use_press_hold};
use leptos::prelude::*;

const BUTTON_ACTION_BASE: &str =
    "relative overflow-hidden select-none transition-transform active:scale-[0.99]";

const PROGRESS_OVERLAY: &str =
    "pointer-events-none absolute inset-y-0 left-0 rounded-[inherit] bg-black/25";

/// Confirmation button that fires `on_complete` only after a press-and-hold of
/// `duration_ms`. A progress overlay fills left-to-right while held and drains
/// once released; releasing early aborts without firing. Native attributes and
/// events forward to the underlying [`Button`].
#[component]
pub fn ButtonAction(
    #[prop(into)] on_complete: Callback<()>,
    #[prop(optional, default = 2000)] duration_ms: u32,
    #[prop(into, optional, default = ButtonVariant::Destructive.into())] variant: Signal<
        ButtonVariant,
    >,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let hold = use_press_hold(duration_ms, on_complete, disabled);

    let merged = move || cn!(BUTTON_ACTION_BASE, class.get());
    let progress_style = move || format!("width: {:.1}%", hold.progress_signal.get() * 100.0);

    view! {
        <Button
            variant=variant
            size=size
            class=merged
            attr:data-name="ButtonAction"
            attr:disabled=disabled
            on:pointerdown=move |_| hold.on_pointer_down()
            on:pointerup=move |_| hold.on_pointer_up()
            on:pointerleave=move |_| hold.on_pointer_up()
            on:pointercancel=move |_| hold.on_pointer_up()
        >
            <span aria-hidden="true" class=PROGRESS_OVERLAY style=progress_style />
            <span class="relative z-10 inline-flex items-center gap-2">{children()}</span>
        </Button>
    }
}
