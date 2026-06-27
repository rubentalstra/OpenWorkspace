use crate::{Button, ButtonSize, ButtonVariant, clx, cn, use_lock_body_scroll, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::HtmlElement;

clx! {
    /// Centered, width-capped column for a drawer's main content.
    DrawerBody, div, "flex flex-col gap-4 mx-auto w-full max-w-[500px]"
}
clx! {
    /// Heading block for a drawer — stacks an optional [`DrawerTitle`] and
    /// [`DrawerDescription`].
    DrawerHeader, div, "flex flex-col gap-2"
}
clx! {
    /// Supporting copy beneath a [`DrawerTitle`].
    DrawerDescription, p, "text-sm text-muted-foreground"
}
clx! {
    /// Action row pinned to the bottom of a drawer panel. Stacks reversed on
    /// mobile so the primary action sits last, aligning trailing from `sm` up.
    DrawerFooter, footer, "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end"
}

/// Edge the drawer anchors to and slides in from.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawerPosition {
    #[default]
    Bottom,
    Left,
    Right,
}

impl DrawerPosition {
    /// Anchoring, sizing and corner-rounding classes that pin the panel to its
    /// edge.
    fn anchor(self) -> &'static str {
        match self {
            Self::Bottom => "inset-x-0 bottom-0 max-h-[96vh] rounded-t-[10px]",
            Self::Left => "inset-y-0 left-0 w-[400px] max-w-[90vw] rounded-r-[10px]",
            Self::Right => "inset-y-0 right-0 w-[400px] max-w-[90vw] rounded-l-[10px]",
        }
    }

    /// Off-screen transform applied while closed; replaced by `translate-*-0`
    /// when open so the panel slides into view.
    fn offscreen(self) -> &'static str {
        match self {
            Self::Bottom => "translate-y-full",
            Self::Left => "-translate-x-full",
            Self::Right => "translate-x-full",
        }
    }

    fn as_data(self) -> &'static str {
        match self {
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// Surface treatment of a [`DrawerContent`] panel: flush to its edge (`Inset`)
/// or detached with a margin and full rounding (`Floating`).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawerVariant {
    #[default]
    Inset,
    Floating,
}

impl DrawerVariant {
    fn surface(self) -> &'static str {
        match self {
            Self::Inset => "",
            Self::Floating => "m-3 rounded-[10px]",
        }
    }

    fn as_data(self) -> &'static str {
        match self {
            Self::Inset => "inset",
            Self::Floating => "floating",
        }
    }
}

/// Open state plus the wiring shared from [`Drawer`] to its trigger, content,
/// handle and close buttons. The title id labels the panel for assistive
/// technology and the trigger id lets the panel return focus to the opener on
/// close.
#[derive(Clone, Copy)]
struct DrawerContext {
    open: RwSignal<bool>,
    title_id: StoredValue<String>,
    trigger_id: StoredValue<String>,
}

impl DrawerContext {
    fn close(self) {
        self.open.set(false);
    }
}

/// Root of an edge-anchored modal drawer. Owns the open state, shares it with
/// the nested [`DrawerTrigger`], [`DrawerContent`], [`DrawerHandle`] and
/// [`DrawerClose`] through context, and locks body scrolling while open.
///
/// Pass `open` to drive the state externally (controlled); otherwise an
/// internal signal seeded from `default_open` is used.
#[component]
pub fn Drawer(
    /// External signal driving open/closed; when omitted the drawer manages its
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
    let ctx = DrawerContext {
        open,
        title_id: StoredValue::new(use_random_id_for("drawer-title")),
        trigger_id: StoredValue::new(use_random_id_for("drawer-trigger")),
    };

    let locked = use_lock_body_scroll(default_open);
    Effect::new(move |_| locked.set(ctx.open.get()));

    view! {
        <Provider value=ctx>
            <div
                data-name="Drawer"
                data-state=move || if open.get() { "open" } else { "closed" }
                class=move || cn!(class.get())
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Prominent heading for a drawer's content. Adopts the shared title id so the
/// [`DrawerContent`] panel can label itself with it for assistive technology.
#[component]
pub fn DrawerTitle(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DrawerContext>();

    view! {
        <h3
            data-name="DrawerTitle"
            id=move || ctx.title_id.get_value()
            class=move || cn!("text-lg leading-none font-semibold", class.get())
        >
            {children()}
        </h3>
    }
}

/// Button that opens the enclosing [`Drawer`]. Defaults to the outline variant;
/// reflects open state via `aria-expanded` and `aria-haspopup="dialog"`, and
/// carries the id the panel focuses on close.
#[component]
pub fn DrawerTrigger(
    #[prop(into, optional, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DrawerContext>();

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

/// Button that closes the enclosing [`Drawer`]. Defaults to the outline variant.
#[component]
pub fn DrawerClose(
    #[prop(into, optional, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<ButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DrawerContext>();

    view! {
        <Button
            variant=variant
            size=size
            class=class
            attr:aria-label="Close drawer"
            on:click=move |_| ctx.close()
        >
            {children()}
        </Button>
    }
}

/// Drag affordance shown at the top of a bottom-anchored [`DrawerContent`].
///
/// Pointer-drag dismissal from the vendored implementation is intentionally
/// simplified: the handle is a button that closes the drawer on click or
/// Enter/Space, pairing with the panel's slide transition. The visible bar is
/// decorative; the surrounding button carries the accessible label.
#[component]
pub fn DrawerHandle() -> impl IntoView {
    let ctx = expect_context::<DrawerContext>();

    view! {
        <button
            type="button"
            data-name="DrawerHandle"
            aria-label="Close drawer"
            class="block relative mx-auto mb-8 h-[5px] w-12 shrink-0 rounded-full bg-muted-foreground/30 opacity-70 transition-opacity hover:opacity-100 active:opacity-100 outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-ring"
            on:click=move |_| ctx.close()
        />
    }
}

/// Modal panel and dimming backdrop for a [`Drawer`]. Carries `role="dialog"`
/// and `aria-modal`, labels itself with the shared title id, and slides in from
/// `position` when the drawer opens. Focuses the panel on open and returns focus
/// to the trigger on close; Escape and (unless disabled) a backdrop click close
/// the drawer.
///
/// The panel stays mounted and animates via a reactive transform toggle — the
/// pure-Leptos analogue of the accordion's grid-reveal trick — so the slide
/// transition runs without JavaScript. While closed it is `aria-hidden` and
/// non-interactive (`pointer-events: none`), keeping it out of the tab order.
#[component]
pub fn DrawerContent(
    #[prop(into, optional)] position: Signal<DrawerPosition>,
    #[prop(into, optional)] variant: Signal<DrawerVariant>,
    /// Whether clicking the backdrop closes the drawer.
    #[prop(default = true)]
    close_on_backdrop_click: bool,
    /// Whether the dimming backdrop is rendered.
    #[prop(default = true)]
    show_overlay: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DrawerContext>();
    let panel_ref = NodeRef::<html::Div>::new();

    Effect::new(move |prev: Option<bool>| {
        let open = ctx.open.get();
        if open {
            if let Some(panel) = panel_ref.get() {
                _ = panel.focus();
            }
        } else if prev == Some(true)
            && let Some(trigger) = document()
                .get_element_by_id(&ctx.trigger_id.get_value())
                .and_then(|el| el.dyn_into::<HtmlElement>().ok())
        {
            _ = trigger.focus();
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
            "fixed inset-0 z-200 bg-black/50 transition-opacity duration-200",
            if show_overlay { "" } else { "hidden" },
            if ctx.open.get() {
                "opacity-100 pointer-events-auto"
            } else {
                "opacity-0 pointer-events-none"
            },
        )
    };

    let panel = move || {
        let pos = position.get();
        cn!(
            "fixed z-210 flex flex-col bg-background shadow-lg pt-3 pb-6 px-6 transition-transform duration-300 overflow-y-auto overscroll-y-contain outline-none",
            pos.anchor(),
            variant.get().surface(),
            if ctx.open.get() {
                "translate-x-0 translate-y-0 pointer-events-auto"
            } else {
                pos.offscreen()
            },
            if ctx.open.get() {
                ""
            } else {
                "pointer-events-none"
            },
            class.get(),
        )
    };

    view! {
        <div
            data-name="DrawerOverlay"
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
            data-name="DrawerContent"
            role="dialog"
            aria-modal="true"
            aria-hidden=move || (!ctx.open.get()).to_string()
            aria-labelledby=move || ctx.title_id.get_value()
            data-position=move || position.get().as_data()
            data-variant=move || variant.get().as_data()
            data-state=data_state
            tabindex="-1"
            class=panel
        >
            {children()}
        </div>
    }
}
