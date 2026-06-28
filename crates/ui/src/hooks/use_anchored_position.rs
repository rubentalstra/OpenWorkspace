use leptos::prelude::*;

/// Which edge of the trigger a popup is placed against (Base UI `side`).
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Side {
    /// Above the trigger.
    Top,
    /// To the right of the trigger.
    Right,
    /// Below the trigger (the default).
    #[default]
    Bottom,
    /// To the left of the trigger.
    Left,
}

impl Side {
    /// The `data-side` value, for the nova slide-in animation.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
        }
    }
}

/// How a popup aligns along the trigger's cross axis (Base UI `align`).
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Align {
    /// Aligned to the trigger's start edge (the default).
    #[default]
    Start,
    /// Centred on the trigger.
    Center,
    /// Aligned to the trigger's end edge.
    End,
}

/// Inline style that parks an as-yet-unmeasured popup off-screen. The measuring
/// effect runs after the content mounts, so the first frame has no real rect; we
/// place the popup off-screen (and its `fade-in` keeps it invisible) instead of at
/// the viewport's top-left corner, which otherwise reads as a "fly-in" once the
/// real position lands.
const OFFSCREEN: &str = "position:fixed;top:-9999px;left:-9999px;";

/// The trigger's viewport-relative rect, refreshed whenever `open` becomes true.
///
/// Anchored popups (popover, dropdown, select, tooltip, …) render their content as a
/// normal DOM descendant of the trigger so outside-click dismissal keeps working,
/// but position it with `position: fixed`. A fixed box escapes any clipping
/// ancestor (e.g. a card's `overflow-hidden`) and stacks above page content, while
/// the measured rect keeps it pinned to the trigger. Until the first measurement
/// the popup is parked off-screen (see [`OFFSCREEN`]).
#[derive(Clone, Copy)]
pub struct AnchorRect {
    left: RwSignal<f64>,
    top: RwSignal<f64>,
    right: RwSignal<f64>,
    bottom: RwSignal<f64>,
    width: RwSignal<f64>,
    height: RwSignal<f64>,
    measured: RwSignal<bool>,
}

impl AnchorRect {
    /// Inline `style` placing the popup directly below the trigger, left-aligned and
    /// at least as wide as it — the default for menus, selects, and popovers.
    pub fn below(self) -> Signal<String> {
        Signal::derive(move || {
            if !self.measured.get() {
                return OFFSCREEN.to_owned();
            }
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
            if !self.measured.get() {
                return OFFSCREEN.to_owned();
            }
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
            if !self.measured.get() {
                return OFFSCREEN.to_owned();
            }
            format!(
                "position:fixed;top:{}px;left:{}px;",
                self.top.get(),
                self.right.get() + 4.0,
            )
        })
    }

    /// Inline `style` placing the popup against `side` of the trigger, aligned per
    /// `align` along the cross axis — the general Base-UI positioner. Alignment uses a
    /// CSS `transform` (so it needs the positioner/popup two-element split to avoid
    /// fighting the enter animation). Parked off-screen until measured.
    #[must_use]
    pub fn place_style(self, side: Side, align: Align) -> String {
        const GAP: f64 = 4.0;
        if !self.measured.get() {
            return OFFSCREEN.to_owned();
        }
        let (anchor_x, tx) = match side {
            Side::Right => (self.right.get() + GAP, 0.0),
            Side::Left => (self.left.get() - GAP, -100.0),
            Side::Top | Side::Bottom => match align {
                Align::Start => (self.left.get(), 0.0),
                Align::Center => (self.center_x(), -50.0),
                Align::End => (self.right.get(), -100.0),
            },
        };
        let (anchor_y, ty) = match side {
            Side::Bottom => (self.bottom.get() + GAP, 0.0),
            Side::Top => (self.top.get() - GAP, -100.0),
            Side::Left | Side::Right => match align {
                Align::Start => (self.top.get(), 0.0),
                Align::Center => (self.center_y(), -50.0),
                Align::End => (self.bottom.get(), -100.0),
            },
        };
        format!(
            "position:fixed;top:{anchor_y}px;left:{anchor_x}px;transform:translate({tx}%,{ty}%);--anchor-width:{}px;",
            self.width.get(),
        )
    }

    /// Whether the trigger has been measured yet; popups using a bespoke style (the
    /// tooltip's side-aware placement) park themselves off-screen until this is true.
    pub fn measured(self) -> bool {
        self.measured.get()
    }

    /// The off-screen parking style for not-yet-measured bespoke popups.
    pub fn offscreen() -> &'static str {
        OFFSCREEN
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
        measured: RwSignal::new(false),
    };
    Effect::new(move |_| {
        if !open.get() {
            rect.measured.set(false);
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
            rect.measured.set(true);
        }
    });
    rect
}
