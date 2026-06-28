use crate::cn;
use leptos::prelude::*;
use web_sys::PointerEvent;

#[derive(Clone, Copy)]
struct ResizableCtx {
    /// Size of the first panel as a percentage of the group (0..=100).
    split: RwSignal<f64>,
    /// The group element, used by the handle to measure its rect.
    group: NodeRef<leptos::html::Div>,
    /// Whether a handle drag is in progress.
    dragging: RwSignal<bool>,
    /// Auto-incrementing index handed to each child panel as it mounts.
    next_index: RwSignal<usize>,
}

/// Minimum size, in percent, that either panel is allowed to shrink to.
const MIN_PCT: f64 = 5.0;

/// ResizablePanelGroup — shadcn Base UI `resizable` (horizontal). A flex row that
/// hosts two [`ResizablePanel`]s split by a draggable [`ResizableHandle`]. The split
/// is controlled via an external `value` signal (first-panel percent) or uncontrolled
/// via `default_value` (defaults to an even 50/50). Vertical orientation is not yet
/// implemented; this group is always horizontal.
#[component]
pub fn ResizablePanelGroup(
    #[prop(optional)] value: Option<RwSignal<f64>>,
    #[prop(default = 50.0)] default_value: f64,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let split =
        value.unwrap_or_else(|| RwSignal::new(default_value.clamp(MIN_PCT, 100.0 - MIN_PCT)));
    let group = NodeRef::<leptos::html::Div>::new();
    provide_context(ResizableCtx {
        split,
        group,
        dragging: RwSignal::new(false),
        next_index: RwSignal::new(0),
    });
    view! {
        <div
            node_ref=group
            data-slot="resizable-panel-group"
            data-orientation="horizontal"
            class=move || {
                cn!(
                    "cn-resizable-panel-group flex h-full w-full aria-[orientation=vertical]:flex-col",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// ResizablePanel — one side of a [`ResizablePanelGroup`]. The first panel mounted is
/// sized to the group's split percentage, the second to the remainder; both grow/shrink
/// from a `flex-basis` driven by that split. Further panels render at their natural size.
#[component]
pub fn ResizablePanel(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ResizableCtx>();
    let index = ctx.next_index.get_untracked();
    ctx.next_index.set(index + 1);
    let basis = move || match index {
        0 => format!("{}%", ctx.split.get()),
        1 => format!("{}%", 100.0 - ctx.split.get()),
        _ => "auto".to_owned(),
    };
    view! {
        <div
            data-slot="resizable-panel"
            class=move || cn!("min-w-0 overflow-hidden", class.get())
            style:flex-grow="0"
            style:flex-shrink="0"
            style:flex-basis=basis
        >
            {children()}
        </div>
    }
}

/// ResizableHandle — the draggable separator between two panels. Pointer-drag adjusts
/// the group's split using the pointer's `client_x` against the group's bounding rect.
/// Set `with_handle` to render the visible grip (`cn-resizable-handle-icon`).
#[component]
pub fn ResizableHandle(
    #[prop(default = false)] with_handle: bool,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let ctx = expect_context::<ResizableCtx>();
    let self_ref = NodeRef::<leptos::html::Div>::new();

    let split_at = move |client_x: f64| {
        let Some(el) = ctx.group.get_untracked() else {
            return;
        };
        let rect = el.get_bounding_client_rect();
        if rect.width() <= 0.0 {
            return;
        }
        let ratio = ((client_x - rect.left()) / rect.width()) * 100.0;
        ctx.split.set(ratio.clamp(MIN_PCT, 100.0 - MIN_PCT));
    };

    let on_pointer_down = move |ev: PointerEvent| {
        ctx.dragging.set(true);
        if let Some(el) = self_ref.get_untracked() {
            _ = el.set_pointer_capture(ev.pointer_id());
        }
        split_at(f64::from(ev.client_x()));
    };
    let on_pointer_move = move |ev: PointerEvent| {
        if ctx.dragging.get_untracked() {
            split_at(f64::from(ev.client_x()));
        }
    };
    let on_pointer_up = move |ev: PointerEvent| {
        ctx.dragging.set(false);
        if let Some(el) = self_ref.get_untracked() {
            _ = el.release_pointer_capture(ev.pointer_id());
        }
    };

    view! {
        <div
            node_ref=self_ref
            data-slot="resizable-handle"
            role="separator"
            aria-orientation="vertical"
            tabindex="0"
            class=move || {
                cn!(
                    "cn-resizable-handle relative flex w-px cursor-col-resize touch-none items-center justify-center bg-border ring-offset-background select-none after:absolute after:inset-y-0 after:left-1/2 after:w-1 after:-translate-x-1/2 focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden",
                    class.get(),
                )
            }
            on:pointerdown=on_pointer_down
            on:pointermove=on_pointer_move
            on:pointerup=on_pointer_up
        >
            <Show when=move || with_handle>
                <div class="cn-resizable-handle-icon z-10 flex shrink-0" />
            </Show>
        </div>
    }
}
