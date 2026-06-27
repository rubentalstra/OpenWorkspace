use crate::{Button, ButtonSize, ButtonVariant, clx, cn};
use leptos::prelude::*;

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
    /// Prominent heading for an alert dialog's content.
    AlertDialogTitle, h2, "text-lg leading-none font-semibold"
}
clx! {
    /// Supporting copy beneath an [`AlertDialogTitle`].
    AlertDialogDescription, p, "text-muted-foreground text-sm"
}

/// Root container that groups an alert dialog's trigger and content. This family
/// is static markup only — open/close behaviour is supplied by the consuming
/// layout. Native attributes and events forward to the root element.
#[component]
pub fn AlertDialog(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div data-name="AlertDialog" class=move || cn!("w-fit", class.get())>
            {children()}
        </div>
    }
}

/// Button that opens the alert dialog. Defaults to the outline variant; native
/// attributes and events forward to the underlying [`Button`].
#[component]
pub fn AlertDialogTrigger(
    #[prop(into, optional, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <Button variant=variant size=size class=class>
            {children()}
        </Button>
    }
}

/// Dimming overlay rendered behind an [`AlertDialogContent`] panel. Kept as its
/// own single-root component so the consuming layout can place it and wire
/// open-state and dismissal; native attributes and events forward to the root.
#[component]
pub fn AlertDialogBackdrop(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <div
            data-name="AlertDialogBackdrop"
            aria-hidden="true"
            class=move || cn!("fixed inset-0 z-60 bg-black/50", class.get())
        />
    }
}

/// Modal panel for an alert dialog. Carries `role="alertdialog"` and
/// `aria-modal`; native attributes and events forward to the panel for wiring
/// open state and labelling. Pair with [`AlertDialogBackdrop`] for the overlay.
#[component]
pub fn AlertDialogContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let panel = move || {
        cn!(
            "relative bg-background border rounded-2xl shadow-lg p-6 w-full max-w-[calc(100%-2rem)] max-h-[85vh] fixed top-[50%] left-[50%] translate-x-[-50%] translate-y-[-50%] z-100 flex flex-col gap-4",
            class.get()
        )
    };

    view! {
        <div data-name="AlertDialogContent" role="alertdialog" aria-modal="true" class=panel>
            {children()}
        </div>
    }
}

/// Primary confirming action. Defaults to the solid variant; native attributes
/// and events forward to the underlying [`Button`].
#[component]
pub fn AlertDialogAction(
    #[prop(into, optional)] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <Button variant=variant size=size class=class>
            {children()}
        </Button>
    }
}

/// Dismissing action. Defaults to the outline variant; native attributes and
/// events forward to the underlying [`Button`].
#[component]
pub fn AlertDialogCancel(
    #[prop(into, optional, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <Button variant=variant size=size class=class>
            {children()}
        </Button>
    }
}
