use crate::{
    Button, ButtonSize, ButtonVariant, clx, cn, use_lock_body_scroll, use_random_id_for,
};
use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;
use web_sys::HtmlElement;

clx! {
    /// Heading block for a sheet — stacks an optional [`SheetTitle`] and
    /// [`SheetDescription`].
    SheetHeader, div, "flex flex-col gap-0.5 p-4"
}
clx! {
    /// Supporting copy beneath a [`SheetTitle`].
    SheetDescription, p, "text-muted-foreground"
}
clx! {
    /// Scrollable body region between a sheet's header and footer.
    SheetBody, div, "flex flex-col gap-4"
}
clx! {
    /// Action row pinned to the bottom of a sheet panel.
    SheetFooter, footer, "mt-auto flex flex-col gap-2 p-4"
}

/// Trigger and close buttons default to the same variants/sizes as [`Button`].
pub type SheetVariant = ButtonVariant;
/// See [`SheetVariant`].
pub type SheetSize = ButtonSize;

/// Edge the panel anchors to and slides in from.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetDirection {
    #[default]
    Right,
    Left,
    Top,
    Bottom,
}

impl SheetDirection {
    /// Anchoring and sizing classes that pin the panel to its edge.
    fn anchor(self) -> &'static str {
        match self {
            Self::Right => "top-0 right-0 h-full w-[400px]",
            Self::Left => "top-0 left-0 h-full w-[400px]",
            Self::Top => "top-0 left-0 w-full h-[400px]",
            Self::Bottom => "bottom-0 left-0 w-full h-[400px]",
        }
    }

    /// Off-screen transform applied while the panel is closed; replaced by
    /// `translate-*-0` when open so the panel slides into view.
    fn offscreen(self) -> &'static str {
        match self {
            Self::Right => "translate-x-full",
            Self::Left => "-translate-x-full",
            Self::Top => "-translate-y-full",
            Self::Bottom => "translate-y-full",
        }
    }

    fn as_data(self) -> &'static str {
        match self {
            Self::Right => "right",
            Self::Left => "left",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

/// Open state plus the wiring shared from [`Sheet`] to its trigger, content and
/// close buttons. The title id labels the panel for assistive technology and the
/// trigger id lets the panel return focus to the opener on close.
#[derive(Clone, Copy)]
struct SheetContext {
    open: RwSignal<bool>,
    title_id: StoredValue<String>,
    trigger_id: StoredValue<String>,
}

impl SheetContext {
    fn close(self) {
        self.open.set(false);
    }
}

/// Root of an edge-anchored modal sheet. Owns the open state, shares it with the
/// nested [`SheetTrigger`], [`SheetContent`] and [`SheetClose`] through context,
/// and locks body scrolling while the sheet is open.
#[component]
pub fn Sheet(
    /// Initial open state.
    #[prop(default = false)]
    default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = SheetContext {
        open: RwSignal::new(default_open),
        title_id: StoredValue::new(use_random_id_for("sheet-title")),
        trigger_id: StoredValue::new(use_random_id_for("sheet-trigger")),
    };

    let locked = use_lock_body_scroll(default_open);
    Effect::new(move |_| locked.set(ctx.open.get()));

    view! {
        <Provider value=ctx>
            <div data-name="Sheet" class=move || cn!(class.get())>
                {children()}
            </div>
        </Provider>
    }
}

/// Prominent heading for a sheet's content. Adopts the shared title id so the
/// [`SheetContent`] panel can label itself with it for assistive technology.
#[component]
pub fn SheetTitle(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<SheetContext>();

    view! {
        <h2
            data-name="SheetTitle"
            id=move || ctx.title_id.get_value()
            class=move || cn!("font-bold text-2xl", class.get())
        >
            {children()}
        </h2>
    }
}

/// Button that opens the enclosing [`Sheet`]. Defaults to the outline variant;
/// reflects open state via `aria-expanded` and `aria-haspopup="dialog"`.
#[component]
pub fn SheetTrigger(
    #[prop(into, optional, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<SheetContext>();

    view! {
        <Button
            variant=variant
            size=size
            class=class
            attr:id=move || ctx.trigger_id.get_value()
            attr:aria-haspopup="dialog"
            attr:aria-expanded=move || ctx.open.get().to_string()
            on:click=move |_| ctx.open.set(true)
        >
            {children()}
        </Button>
    }
}

/// Button that closes the enclosing [`Sheet`]. Defaults to the outline variant.
#[component]
pub fn SheetClose(
    #[prop(into, optional, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<SheetContext>();

    view! {
        <Button
            variant=variant
            size=size
            class=class
            attr:aria-label="Close sheet"
            on:click=move |_| ctx.close()
        >
            {children()}
        </Button>
    }
}

/// Modal panel for a [`Sheet`]. Carries `role="dialog"` and `aria-modal`, labels
/// itself with the shared title id, and slides in from `direction` when the sheet
/// opens. Focuses the panel on open and returns focus to the trigger on close;
/// Escape and (unless disabled) a backdrop click close the sheet.
///
/// The panel stays mounted and animates via a reactive transform toggle — the
/// pure-Leptos analogue of the accordion's grid-reveal trick — so the slide
/// transition runs without JavaScript. While closed it is `aria-hidden` and
/// non-interactive (`pointer-events: none`), keeping it out of the tab order.
#[component]
pub fn SheetContent(
    #[prop(into, optional)] direction: Signal<SheetDirection>,
    /// Whether the corner close button is shown.
    #[prop(default = true)]
    show_close_button: bool,
    /// Whether clicking the backdrop closes the sheet.
    #[prop(default = true)]
    close_on_backdrop_click: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<SheetContext>();
    let panel_ref = NodeRef::<html::Div>::new();

    Effect::new(move |prev: Option<bool>| {
        let open = ctx.open.get();
        if open {
            if let Some(panel) = panel_ref.get() {
                _ = panel.focus();
            }
        } else if prev == Some(true) {
            if let Some(trigger) = document()
                .get_element_by_id(&ctx.trigger_id.get_value())
                .and_then(|el| el.dyn_into::<HtmlElement>().ok())
            {
                _ = trigger.focus();
            }
        }
        open
    });

    Effect::new(move |_| {
        let handle = window_event_listener(leptos::ev::keydown, move |ev: KeyboardEvent| {
            if ev.key() == "Escape" && ctx.open.get_untracked() {
                ev.prevent_default();
                ctx.close();
            }
        });
        on_cleanup(move || handle.remove());
    });

    let data_state = move || if ctx.open.get() { "open" } else { "closed" };

    let backdrop = move || {
        cn!(
            "fixed inset-0 z-60 bg-black/50 transition-opacity duration-200",
            if ctx.open.get() {
                "opacity-100 pointer-events-auto"
            } else {
                "opacity-0 pointer-events-none"
            },
        )
    };

    let panel = move || {
        let dir = direction.get();
        cn!(
            "fixed z-100 flex flex-col bg-card shadow-lg p-6 transition-transform duration-300 overflow-y-auto overscroll-y-contain outline-none",
            dir.anchor(),
            if ctx.open.get() {
                "translate-x-0 translate-y-0 pointer-events-auto"
            } else {
                dir.offscreen()
            },
            if ctx.open.get() { "" } else { "pointer-events-none" },
            class.get(),
        )
    };

    view! {
        <div
            data-name="SheetBackdrop"
            aria-hidden="true"
            class=backdrop
            data-state=data_state
            on:click=move |_| {
                if close_on_backdrop_click {
                    ctx.close();
                }
            }
        />

        <div
            node_ref=panel_ref
            data-name="SheetContent"
            role="dialog"
            aria-modal="true"
            aria-hidden=move || (!ctx.open.get()).to_string()
            aria-labelledby=move || ctx.title_id.get_value()
            data-direction=move || direction.get().as_data()
            data-state=data_state
            tabindex="-1"
            class=panel
        >
            <Show when=move || show_close_button>
                <button
                    type="button"
                    class="absolute top-4 right-4 p-1 rounded-sm focus:ring-2 focus:ring-offset-2 focus:outline-none [&_svg:not([class*='size-'])]:size-4 focus:ring-ring"
                    aria-label="Close sheet"
                    on:click=move |_| ctx.close()
                >
                    <span class="sr-only">"Close sheet"</span>
                    <Icon icon=icondata::LuX />
                </button>
            </Show>

            {children()}
        </div>
    }
}
