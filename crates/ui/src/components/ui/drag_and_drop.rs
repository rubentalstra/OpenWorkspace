use crate::{clx, cn};
use leptos::prelude::*;

/// Shared reorder state for a [`Draggable`] list. Tracks the item currently
/// being dragged and the item the pointer is hovering over, and forwards
/// completed drops to the caller-supplied `on_reorder` callback.
#[derive(Clone, Copy)]
pub struct DragDropContext {
    dragging: RwSignal<Option<usize>>,
    over: RwSignal<Option<usize>>,
    on_reorder: StoredValue<Callback<(usize, usize)>>,
}

impl DragDropContext {
    fn start(self, id: usize) {
        self.dragging.set(Some(id));
    }

    fn enter(self, id: usize) {
        if self.dragging.get_untracked().is_some() {
            self.over.set(Some(id));
        }
    }

    fn end(self) {
        self.dragging.set(None);
        self.over.set(None);
    }

    fn drop_on(self, target: usize) {
        if let Some(source) = self.dragging.get_untracked()
            && source != target
        {
            self.on_reorder.get_value().run((source, target));
        }
        self.end();
    }
}

clx! {
    /// Drop region inside a [`Draggable`] list — a styled surface that holds the
    /// reorderable items.
    DraggableZone, div, "flex flex-col gap-2 rounded-md border border-input bg-muted/40 p-4"
}

/// Reorderable list root. Provides [`DragDropContext`] to descendant
/// [`DraggableItem`]s and emits `on_reorder` with `(source_index, target_index)`
/// when an item is dropped onto another. Reordering is the caller's data
/// concern — keep the rendered order driven by your own signal so the DOM stays
/// owned by the reactive graph rather than mutated out of band.
#[component]
pub fn Draggable(
    /// Invoked with `(source_index, target_index)` on a completed drop.
    #[prop(into)]
    on_reorder: Callback<(usize, usize)>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    provide_context(DragDropContext {
        dragging: RwSignal::new(None),
        over: RwSignal::new(None),
        on_reorder: StoredValue::new(on_reorder),
    });

    view! {
        <div
            data-name="Draggable"
            class=move || cn!("flex w-full max-w-4xl flex-col gap-4", class.get())
        >
            {children()}
        </div>
    }
}

/// A draggable entry within a [`Draggable`] list. `index` is its position in the
/// caller's ordered data and is reported back through `on_reorder`. Drag, hover
/// and drop are handled in Rust via the native HTML drag events; visual state is
/// driven by reactive classes, so no out-of-band DOM mutation occurs.
#[component]
pub fn DraggableItem(
    /// Position of this item in the caller's ordered data.
    index: usize,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DragDropContext>();

    let is_dragging = move || ctx.dragging.get() == Some(index);
    let is_over = move || ctx.over.get() == Some(index) && ctx.dragging.get() != Some(index);

    let merged = move || {
        cn!(
            "flex cursor-move touch-none items-center rounded-md border border-input bg-card p-4 transition-[opacity,box-shadow] [&_svg:not([class*='size-'])]:size-4 [&_svg]:shrink-0",
            is_dragging().then_some("opacity-50"),
            is_over().then_some("ring-2 ring-ring ring-offset-2"),
            class.get(),
        )
    };

    view! {
        <div
            data-name="DraggableItem"
            class=merged
            draggable="true"
            tabindex="0"
            aria-grabbed=move || if is_dragging() { "true" } else { "false" }
            on:dragstart=move |_| ctx.start(index)
            on:dragend=move |_| ctx.end()
            on:dragenter=move |_| ctx.enter(index)
            on:dragover=move |ev| {
                if ctx.dragging.get_untracked().is_some() {
                    ev.prevent_default();
                }
            }
            on:drop=move |ev| {
                ev.prevent_default();
                ctx.drop_on(index);
            }
        >
            {children()}
        </div>
    }
}
