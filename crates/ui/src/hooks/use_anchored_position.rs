use leptos::prelude::*;

/// The trigger's viewport-relative rect, refreshed whenever `open` becomes true.
///
/// Anchored popups (popover, dropdown, select, tooltip, …) render their content as a
/// normal DOM descendant of the trigger so outside-click dismissal keeps working,
/// but position it with `position: fixed`. A fixed box escapes any clipping
/// ancestor (e.g. a card's `overflow-hidden`) and stacks above page content, while
/// the measured rect keeps it pinned to the trigger. Values are zero on the server
/// and before the first client open.
#[derive(Clone, Copy)]
pub struct AnchorRect {
    left: RwSignal<f64>,
    top: RwSignal<f64>,
    right: RwSignal<f64>,
    bottom: RwSignal<f64>,
    width: RwSignal<f64>,
    height: RwSignal<f64>,
}

impl AnchorRect {
    /// Inline `style` placing the popup directly below the trigger, left-aligned and
    /// at least as wide as it — the default for menus, selects, and popovers.
    pub fn below(self) -> Signal<String> {
        Signal::derive(move || {
            format!(
                "position:fixed;top:{}px;left:{}px;min-width:{}px;",
                self.bottom.get() + 4.0,
                self.left.get(),
                self.width.get(),
            )
        })
    }

    /// Inline `style` placing a fixed-width popup below the trigger, horizontally
    /// centred on it — the popover default (`align: center`).
    pub fn below_center(self) -> Signal<String> {
        Signal::derive(move || {
            format!(
                "position:fixed;top:{}px;left:{}px;transform:translateX(-50%);",
                self.bottom.get() + 4.0,
                self.center_x(),
            )
        })
    }

    /// Inline `style` placing a submenu to the right of the trigger, top-aligned —
    /// the submenu default (`side: right`).
    pub fn right_of(self) -> Signal<String> {
        Signal::derive(move || {
            format!(
                "position:fixed;top:{}px;left:{}px;",
                self.top.get(),
                self.right.get() + 4.0,
            )
        })
    }

    /// Distance of the trigger's top edge from the viewport top (px).
    pub fn top(self) -> f64 {
        self.top.get()
    }

    /// Distance of the trigger's bottom edge from the viewport top (px).
    pub fn bottom(self) -> f64 {
        self.bottom.get()
    }

    /// Distance of the trigger's left edge from the viewport left (px).
    pub fn left(self) -> f64 {
        self.left.get()
    }

    /// Distance of the trigger's right edge from the viewport left (px).
    pub fn right(self) -> f64 {
        self.right.get()
    }

    /// Horizontal centre of the trigger (px).
    pub fn center_x(self) -> f64 {
        self.left.get() + self.width.get() / 2.0
    }

    /// Vertical centre of the trigger (px).
    pub fn center_y(self) -> f64 {
        self.top.get() + self.height.get() / 2.0
    }
}

/// Measure `anchor`'s bounding rect each time `open` flips true, returning the
/// reactive [`AnchorRect`] an overlay's content uses to position itself `fixed`.
pub fn use_anchor_rect(open: RwSignal<bool>, anchor: NodeRef<leptos::html::Div>) -> AnchorRect {
    let rect = AnchorRect {
        left: RwSignal::new(0.0),
        top: RwSignal::new(0.0),
        right: RwSignal::new(0.0),
        bottom: RwSignal::new(0.0),
        width: RwSignal::new(0.0),
        height: RwSignal::new(0.0),
    };
    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        if let Some(el) = anchor.get_untracked() {
            let r = el.get_bounding_client_rect();
            rect.left.set(r.left());
            rect.top.set(r.top());
            rect.right.set(r.right());
            rect.bottom.set(r.bottom());
            rect.width.set(r.width());
            rect.height.set(r.height());
        }
    });
    rect
}
