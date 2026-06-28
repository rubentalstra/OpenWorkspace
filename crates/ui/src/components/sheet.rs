use crate::{cn, slot};
use leptos::ev;
use leptos::prelude::*;
use leptos_icons::Icon;

#[derive(Clone, Copy)]
struct SheetCtx {
    open: RwSignal<bool>,
}

/// Sheet — shadcn Base UI `sheet`. A modal side panel. Controlled via an external
/// `open` signal or uncontrolled via `default_open`; closes on backdrop click,
/// Escape, or the close button.
#[component]
pub fn Sheet(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    provide_context(SheetCtx { open });
    let on_key = window_event_listener(ev::keydown, move |event| {
        if open.get_untracked() && event.key() == "Escape" {
            open.set(false);
        }
    });
    on_cleanup(move || on_key.remove());
    view! { <div data-slot="sheet">{children()}</div> }
}

/// Opens the sheet.
#[component]
pub fn SheetTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<SheetCtx>();
    view! {
        <button
            type="button"
            data-slot="sheet-trigger"
            class=move || cn!("", class.get())
            on:click=move |_| ctx.open.set(true)
        >
            {children()}
        </button>
    }
}

/// The edge a [`SheetContent`] panel slides in from.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SheetSide {
    /// Slides in from the left edge.
    Left,
    /// Slides in from the right edge (the default).
    #[default]
    Right,
    /// Slides in from the top edge.
    Top,
    /// Slides in from the bottom edge.
    Bottom,
}

impl SheetSide {
    fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

/// The side panel + backdrop; mounted (and enter-animated) while open.
#[component]
pub fn SheetContent(
    #[prop(into, optional)] side: Signal<SheetSide>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(default = true)] show_close: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<SheetCtx>();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                data-slot="sheet-overlay"
                data-open="true"
                class="cn-sheet-overlay fixed inset-0 z-50 transition-opacity duration-150"
                on:click=move |_| ctx.open.set(false)
            ></div>
            <div
                data-slot="sheet-content"
                data-open="true"
                data-side=move || side.get().as_str()
                role="dialog"
                aria-modal="true"
                class=move || cn!("cn-sheet-content", class.get())
            >
                {children()}
                {show_close
                    .then(|| {
                        view! {
                            <button
                                type="button"
                                data-slot="sheet-close"
                                aria-label="Close"
                                class="cn-sheet-close cn-button cn-button-variant-ghost cn-button-size-icon-sm"
                                on:click=move |_| ctx.open.set(false)
                            >
                                <Icon icon=icondata::LuX />
                                <span class="sr-only">"Close"</span>
                            </button>
                        }
                    })}
            </div>
        </Show>
    }
}

/// A button that closes the sheet (use inside the footer/content).
#[component]
pub fn SheetClose(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<SheetCtx>();
    view! {
        <button
            type="button"
            data-slot="sheet-close"
            class=move || cn!("", class.get())
            on:click=move |_| ctx.open.set(false)
        >
            {children()}
        </button>
    }
}

slot! { SheetHeader, div, "sheet-header", "cn-sheet-header flex flex-col" }
slot! { SheetFooter, div, "sheet-footer", "cn-sheet-footer mt-auto flex flex-col" }
slot! { SheetTitle, h2, "sheet-title", "cn-sheet-title cn-font-heading" }
slot! { SheetDescription, p, "sheet-description", "cn-sheet-description" }
