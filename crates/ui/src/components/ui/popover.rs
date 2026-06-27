use crate::{clx, cn, use_random_id_for};
use leptos::context::Provider;
use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::Node;

clx! {
    /// Heading for a popover panel; pair with [`PopoverDescription`] inside
    /// [`PopoverContent`].
    PopoverTitle, h3, "leading-none font-medium mb-3"
}
clx! {
    /// Muted supporting copy for a popover panel.
    PopoverDescription, p, "text-muted-foreground text-sm"
}

/// Placement of a [`PopoverContent`] panel relative to its trigger. The panel is
/// absolutely positioned within the [`Popover`] root, so the alignment maps to a
/// fixed corner rather than the CSS anchor-positioning fallbacks the vendored
/// version relied on.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverAlign {
    /// Below the trigger, left edges aligned.
    Start,
    /// To the left of the trigger, top edges aligned.
    StartOuter,
    /// Below the trigger, right edges aligned.
    End,
    /// To the right of the trigger, top edges aligned.
    EndOuter,
    /// Below the trigger, horizontally centered.
    #[default]
    Center,
}

impl PopoverAlign {
    fn placement(self) -> &'static str {
        match self {
            Self::Start => "top-full left-0 mt-2 origin-top-left",
            Self::StartOuter => "right-full top-0 mr-2 origin-top-right",
            Self::End => "top-full right-0 mt-2 origin-top-right",
            Self::EndOuter => "left-full top-0 ml-2 origin-top-left",
            Self::Center => "top-full left-1/2 -translate-x-1/2 mt-2 origin-top",
        }
    }
}

/// Shared open state plus the wiring that lets the parts cooperate: node refs for
/// outside-click detection and focus return, and the title id that labels the
/// panel for assistive technology. Provided by [`Popover`] to its descendants.
#[derive(Clone, Copy)]
struct PopoverContext {
    open: RwSignal<bool>,
    align: PopoverAlign,
    title_id: StoredValue<String>,
    trigger_ref: NodeRef<html::Button>,
    panel_ref: NodeRef<html::Div>,
}

/// Anchored popover. Owns the open state and shares it with the nested
/// [`PopoverTrigger`] and [`PopoverContent`] through context; the trigger toggles
/// it and the panel closes on Escape or an outside click.
#[component]
pub fn Popover(
    /// Initial open state.
    #[prop(default = false)]
    default_open: bool,
    #[prop(default = PopoverAlign::default())] align: PopoverAlign,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = PopoverContext {
        open: RwSignal::new(default_open),
        align,
        title_id: StoredValue::new(use_random_id_for("popover")),
        trigger_ref: NodeRef::new(),
        panel_ref: NodeRef::new(),
    };

    view! {
        <Provider value=ctx>
            <div
                data-name="Popover"
                data-state=move || if ctx.open.get() { "open" } else { "closed" }
                class=move || cn!("relative inline-block w-fit", class.get())
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Button that toggles the enclosing [`Popover`]. Reflects the open state via
/// `aria-expanded` and advertises the popup through `aria-haspopup`.
#[component]
pub fn PopoverTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<PopoverContext>();
    let merged = move || {
        cn!(
            "px-4 py-2 h-9 inline-flex justify-center items-center text-sm font-medium whitespace-nowrap rounded-md transition-colors w-fit border bg-background border-input hover:bg-accent hover:text-accent-foreground outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&_svg:not([class*='size-'])]:size-4",
            class.get(),
        )
    };

    view! {
        <button
            node_ref=ctx.trigger_ref
            type="button"
            data-name="PopoverTrigger"
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            aria-haspopup="dialog"
            aria-expanded=move || ctx.open.get().to_string()
            class=merged
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
        </button>
    }
}

/// Panel revealed while the enclosing [`Popover`] is open. Renders inside a
/// [`Show`], carries `role="dialog"` labelled by a [`PopoverTitle`], focuses
/// itself on open, returns focus to the trigger on close, and dismisses on
/// Escape or a pointer press outside the panel and trigger.
#[component]
pub fn PopoverContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<PopoverContext>();
    let children = StoredValue::new(children);

    Effect::new(move |_| {
        if !ctx.open.get() {
            return;
        }
        let keydown = window_event_listener(leptos::ev::keydown, move |ev: KeyboardEvent| {
            if ev.key() == "Escape" {
                ev.prevent_default();
                ctx.open.set(false);
            }
        });
        let pointerdown = window_event_listener(leptos::ev::mousedown, move |ev: MouseEvent| {
            if is_outside(&ev, ctx.panel_ref, ctx.trigger_ref) {
                ctx.open.set(false);
            }
        });
        on_cleanup(move || {
            keydown.remove();
            pointerdown.remove();
        });
    });

    Effect::new(move |was_open: Option<bool>| {
        let open = ctx.open.get();
        if open {
            if let Some(panel) = ctx.panel_ref.get() {
                _ = panel.focus();
            }
        } else if was_open == Some(true) {
            if let Some(trigger) = ctx.trigger_ref.get_untracked() {
                _ = trigger.focus();
            }
        }
        open
    });

    let merged = move || {
        cn!(
            "absolute z-50 p-4 rounded-md border bg-card shadow-md w-[250px] outline-none",
            ctx.align.placement(),
            class.get(),
        )
    };

    view! {
        <Show when=move || ctx.open.get()>
            <div
                node_ref=ctx.panel_ref
                data-name="PopoverContent"
                role="dialog"
                tabindex="-1"
                aria-labelledby=move || ctx.title_id.get_value()
                class=merged
            >
                {children.get_value()()}
            </div>
        </Show>
    }
}

/// Reports whether a pointer event landed outside both the popover panel and its
/// trigger. Runs only inside the pointer handler, so the `web_sys` DOM access
/// never executes during server rendering.
fn is_outside(
    ev: &MouseEvent,
    panel_ref: NodeRef<html::Div>,
    trigger_ref: NodeRef<html::Button>,
) -> bool {
    let Some(target) = ev.target().and_then(|t| t.dyn_into::<Node>().ok()) else {
        return false;
    };
    let in_panel = panel_ref
        .get_untracked()
        .is_some_and(|panel| panel.contains(Some(&target)));
    let in_trigger = trigger_ref
        .get_untracked()
        .is_some_and(|trigger| trigger.contains(Some(&target)));
    !in_panel && !in_trigger
}
