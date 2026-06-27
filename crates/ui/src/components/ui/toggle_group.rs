use crate::{clx, cn};
use leptos::html;
use leptos::prelude::*;

const GROUP_BASE: &str =
    "group/toggle-group flex w-fit items-center rounded-md data-[orientation=vertical]:flex-col";

const ITEM_BASE: &str = "inline-flex h-9 min-w-0 flex-1 shrink-0 items-center justify-center gap-2 whitespace-nowrap bg-transparent px-2 text-sm font-medium shadow-none outline-none transition-[color,box-shadow] hover:bg-muted hover:text-muted-foreground focus:z-10 focus-visible:z-10 focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-pressed:bg-accent aria-pressed:text-accent-foreground aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";

clx! {
    /// Action affordance rendered alongside a [`ToggleGroup`], styled to match
    /// its items but semantically a link rather than a toggle.
    ToggleGroupAction, a,
    "inline-flex size-6 shrink-0 items-center justify-center gap-2 whitespace-nowrap rounded-sm p-0 text-sm font-medium outline-none transition-all hover:bg-accent hover:text-accent-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 dark:hover:bg-accent/50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"
}

/// Visual style of a [`ToggleGroup`].
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleGroupVariant {
    #[default]
    Default,
    Outline,
}

/// Layout axis of a [`ToggleGroup`].
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Selection cardinality of a [`ToggleGroup`].
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleGroupSelection {
    /// At most one item is pressed at a time; pressing it again clears the
    /// selection.
    #[default]
    Single,
    /// Any number of items may be pressed independently.
    Multiple,
}

/// Pressed-item state shared from a [`ToggleGroup`] to its descendant
/// [`ToggleGroupItem`]s, along with the styling axes each item reads back to
/// match the group chrome.
#[derive(Clone, Copy)]
struct ToggleGroupCtx {
    variant: ToggleGroupVariant,
    orientation: ToggleGroupOrientation,
    selection: ToggleGroupSelection,
    grouped: bool,
    pressed: RwSignal<Vec<String>>,
}

impl ToggleGroupCtx {
    fn is_pressed(&self, value: &str) -> bool {
        self.pressed.with(|set| set.iter().any(|v| v == value))
    }

    fn toggle(&self, value: &str) {
        let already = self.is_pressed(value);
        self.pressed.update(|set| match self.selection {
            ToggleGroupSelection::Single => {
                set.clear();
                if !already {
                    set.push(value.to_owned());
                }
            }
            ToggleGroupSelection::Multiple => {
                if already {
                    set.retain(|v| v != value);
                } else {
                    set.push(value.to_owned());
                }
            }
        });
    }
}

/// A set of related toggle buttons. Tracks pressed items in shared state and
/// exposes the selection through `value`, restyling each [`ToggleGroupItem`]
/// from its `aria-pressed` state. `spacing` of `0` fuses items into a single
/// segmented bar with shared borders and rounded outer edges. Native
/// attributes, events and bindings forward to the root element.
#[component]
pub fn ToggleGroup(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] variant: ToggleGroupVariant,
    #[prop(optional)] orientation: ToggleGroupOrientation,
    #[prop(optional)] selection: ToggleGroupSelection,
    #[prop(into, default = RwSignal::new(Vec::new()))] value: RwSignal<Vec<String>>,
    #[prop(optional, default = 1)] spacing: i32,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let grouped = spacing == 0;
    provide_context(ToggleGroupCtx {
        variant,
        orientation,
        selection,
        grouped,
        pressed: value,
    });

    let vertical = orientation == ToggleGroupOrientation::Vertical;
    let outline = variant == ToggleGroupVariant::Outline;
    let gap = move || {
        if grouped {
            "gap:0".to_owned()
        } else {
            format!("gap:{}rem", f64::from(spacing) * 0.25)
        }
    };
    let merged = move || cn!(GROUP_BASE, outline.then_some("shadow-xs"), class.get());

    view! {
        <div
            node_ref=node_ref
            data-name="ToggleGroup"
            role="group"
            data-variant=if outline { "outline" } else { "default" }
            data-orientation=if vertical { "vertical" } else { "horizontal" }
            data-spacing=spacing.to_string()
            style=gap
            class=merged
        >
            {children()}
        </div>
    }
}

/// A single button within a [`ToggleGroup`]. Clicking it toggles `value` in the
/// group's selection and flips `aria-pressed`; the enclosing group restyles the
/// pressed state. Native attributes, events and bindings forward to the root.
#[component]
pub fn ToggleGroupItem(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Button>,
    children: Children,
) -> impl IntoView {
    let ctx = use_context::<ToggleGroupCtx>();

    let select_value = value.clone();
    let pressed_value = value;
    let pressed = move || ctx.is_some_and(|c| c.is_pressed(&pressed_value));

    let vertical = ctx.is_some_and(|c| c.orientation == ToggleGroupOrientation::Vertical);
    let grouped = ctx.is_some_and(|c| c.grouped);
    let outline = ctx.is_some_and(|c| c.variant == ToggleGroupVariant::Outline);

    let rounded = match (grouped, vertical) {
        (true, true) => "rounded-none first:rounded-t-md last:rounded-b-md",
        (true, false) => "rounded-none first:rounded-l-md last:rounded-r-md",
        (false, _) => "rounded-md",
    };
    let border = match (outline, grouped, vertical) {
        (true, true, true) => "border border-t-0 first:border-t",
        (true, true, false) => "border border-l-0 first:border-l",
        (true, false, _) => "border",
        (false, _, _) => "",
    };
    let width = vertical.then_some("w-full");

    let merged = move || cn!(ITEM_BASE, rounded, border, width, class.get());

    view! {
        <button
            node_ref=node_ref
            data-name="ToggleGroupItem"
            type="button"
            class=merged
            aria-pressed=move || if pressed() { "true" } else { "false" }
            on:click=move |_| {
                if let Some(ctx) = ctx {
                    ctx.toggle(&select_value);
                }
            }
        >
            {children()}
        </button>
    }
}
