use crate::{cn, slot};
use leptos::ev;
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct DrawerCtx {
    open: RwSignal<bool>,
}

/// Drawer — shadcn Base UI `drawer` (vaul). A bottom modal sheet that slides up
/// from the bottom edge. Controlled via an external `open` signal or uncontrolled
/// via `default_open`; closes on backdrop click, Escape, or a close button.
#[component]
pub fn Drawer(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    provide_context(DrawerCtx { open });
    let on_key = window_event_listener(ev::keydown, move |event| {
        if open.get_untracked() && event.key() == "Escape" {
            open.set(false);
        }
    });
    on_cleanup(move || on_key.remove());
    view! { <div data-slot="drawer">{children()}</div> }
}

/// Opens the drawer.
#[component]
pub fn DrawerTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DrawerCtx>();
    view! {
        <button
            type="button"
            data-slot="drawer-trigger"
            class=move || cn!("", class.get())
            on:click=move |_| ctx.open.set(true)
        >
            {children()}
        </button>
    }
}

/// The drawer panel + backdrop; mounted (and enter-animated) while open. The
/// drag handle is shown at the top of the bottom-direction panel.
#[component]
pub fn DrawerContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<DrawerCtx>();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                data-slot="drawer-overlay"
                data-open="true"
                class="cn-drawer-overlay fixed inset-0 z-50"
                on:click=move |_| ctx.open.set(false)
            ></div>
            <div
                data-slot="drawer-content"
                data-open="true"
                data-vaul-drawer-direction="bottom"
                role="dialog"
                aria-modal="true"
                class=move || {
                    cn!("cn-drawer-content group/drawer-content fixed z-50", class.get())
                }
            >
                <div class="cn-drawer-handle mx-auto hidden shrink-0 group-data-[vaul-drawer-direction=bottom]/drawer-content:block"></div>
                {children()}
            </div>
        </Show>
    }
}

/// A button that closes the drawer (use inside the footer/content).
#[component]
pub fn DrawerClose(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DrawerCtx>();
    view! {
        <button
            type="button"
            data-slot="drawer-close"
            class=move || cn!("", class.get())
            on:click=move |_| ctx.open.set(false)
        >
            {children()}
        </button>
    }
}

slot! { DrawerHeader, div, "drawer-header", "cn-drawer-header flex flex-col" }
slot! { DrawerFooter, div, "drawer-footer", "cn-drawer-footer mt-auto flex flex-col" }
slot! { DrawerTitle, h2, "drawer-title", "cn-drawer-title cn-font-heading" }
slot! { DrawerDescription, p, "drawer-description", "cn-drawer-description" }
