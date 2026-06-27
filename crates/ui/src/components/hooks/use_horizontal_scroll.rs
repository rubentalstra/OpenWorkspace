use std::time::Duration;

use leptos::html::Div;
use leptos::prelude::*;
use strum::Display;
use web_sys::Element;

const DEFAULT_SCROLL_PERCENTAGE: f64 = 0.5;
const DEFAULT_UPDATE_DELAY_MS: u64 = 300;

/// Position of a horizontally scrollable container relative to its content.
#[derive(Default, Clone, Copy, Display, PartialEq, Eq, Debug)]
#[strum(serialize_all = "PascalCase")]
pub enum HorizontalScrollState {
    /// Scrolled fully to the start (left edge).
    #[default]
    Start,
    /// Scrolled somewhere between the start and end.
    Middle,
    /// Scrolled fully to the end (right edge).
    End,
}

/// Handle returned by [`use_horizontal_scroll`]: the live scroll position plus
/// callbacks to drive and observe a horizontally scrollable container.
#[derive(Clone)]
pub struct HorizontalScrollContext {
    /// Tracks where the container is scrolled. Defaults to
    /// [`HorizontalScrollState::Start`] until the first client-side update.
    pub scroll_state: RwSignal<HorizontalScrollState>,
    /// Scrolls by a fraction of the container width; the sign of the argument
    /// chooses direction (negative left, positive right).
    pub scroll_by: Callback<i32>,
    /// Recomputes [`scroll_state`](Self::scroll_state); wire to the element's
    /// `on:scroll`. The scroll event is ignored — state is read from the DOM.
    pub on_scroll: Callback<leptos::ev::Event>,
}

/// Tracks and controls horizontal scrolling of the element behind `node_ref`.
///
/// `scroll_percentage` is the fraction of the container width that
/// [`HorizontalScrollContext::scroll_by`] moves (default `0.5`);
/// `update_delay_ms` is how long after a programmatic scroll the state is
/// recomputed, giving smooth scrolling time to settle (default `300`).
///
/// Returns a [`HorizontalScrollContext`]. The DOM is only touched inside the
/// returned callbacks, which run on client interaction, so the hook is safe to
/// call during SSR.
pub fn use_horizontal_scroll(
    node_ref: NodeRef<Div>,
    scroll_percentage: Option<f64>,
    update_delay_ms: Option<u64>,
) -> HorizontalScrollContext {
    let scroll_state = RwSignal::new(HorizontalScrollState::default());
    let scroll_pct = scroll_percentage.unwrap_or(DEFAULT_SCROLL_PERCENTAGE);
    let delay = Duration::from_millis(update_delay_ms.unwrap_or(DEFAULT_UPDATE_DELAY_MS));

    let update_scroll_state = move || {
        let Some(element) = current_element(node_ref) else {
            return;
        };
        let scroll_left = element.scroll_left();
        let scroll_width = element.scroll_width();
        let client_width = element.client_width();

        let state = if scroll_left <= 0 {
            HorizontalScrollState::Start
        } else if scroll_left >= scroll_width - client_width - 1 {
            HorizontalScrollState::End
        } else {
            HorizontalScrollState::Middle
        };

        // try_set so a scroll settling after the component unmounts is a no-op
        // rather than a write to a disposed signal.
        _ = scroll_state.try_set(state);
    };

    let scroll_by = Callback::new(move |direction: i32| {
        let Some(element) = current_element(node_ref) else {
            return;
        };
        let container_width = f64::from(element.client_width());
        let scroll_amount = scroll_amount_px(container_width, scroll_pct, direction);
        element.set_scroll_left(element.scroll_left() + scroll_amount);
        set_timeout(update_scroll_state, delay);
    });

    let on_scroll = Callback::new(move |_: leptos::ev::Event| update_scroll_state());

    HorizontalScrollContext {
        scroll_state,
        scroll_by,
        on_scroll,
    }
}

/// Pixels to scroll for one [`HorizontalScrollContext::scroll_by`] step,
/// rounded and saturated to `i32` so a huge container can never overflow.
#[expect(
    clippy::cast_possible_truncation,
    reason = "signed is clamped to the i32 range immediately above the cast"
)]
fn scroll_amount_px(container_width: f64, scroll_pct: f64, direction: i32) -> i32 {
    let magnitude = (container_width * scroll_pct).round();
    let signed = magnitude * f64::from(direction);
    if signed >= f64::from(i32::MAX) {
        i32::MAX
    } else if signed <= f64::from(i32::MIN) {
        i32::MIN
    } else {
        signed as i32
    }
}

/// Resolves `node_ref` to a [`web_sys::Element`] on the client; `None` during
/// SSR or before the node mounts.
fn current_element(node_ref: NodeRef<Div>) -> Option<Element> {
    node_ref
        .get()
        .map(|div| AsRef::<Element>::as_ref(&div).clone())
}
