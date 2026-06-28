use std::cell::RefCell;

use leptos::ev;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use web_sys::{Element, Event, EventTarget, Node, NodeList};

const CAROUSEL_ROOT: &str = r#"[data-name="CardCarousel"]"#;
const CAROUSEL_TRACK: &str = r#"[data-name="CardCarouselTrack"]"#;
const CAROUSEL_NAV_BUTTON: &str = r#"[data-name="CardCarouselNavButton"]"#;
const CAROUSEL_INDICATOR: &str = r#"[data-name="CardCarouselIndicator"]"#;

thread_local! {
    static REGISTERED: RefCell<Option<Registration>> = const { RefCell::new(None) };
}

/// Live listeners for the page-wide carousel controller, parked in a
/// `thread_local` so the click handle and captured scroll closure outlive every
/// individual carousel. Dropping them would unbind the listeners while
/// carousels are still mounted, so the controller is never torn down for the
/// life of the page.
struct Registration {
    _click: WindowListenerHandle,
    _scroll: Closure<dyn FnMut(Event)>,
}

/// Wire up the single delegated controller that drives every `CardCarousel` on
/// the page: nav-button clicks scroll the track, and track scrolling updates the
/// indicator dots and the nav-button disabled state.
///
/// Idempotent — the first call on the client registers the listeners and later
/// calls are no-ops, so every `CardCarouselTrack` may call it freely. The work
/// runs inside an [`Effect`], which never executes during server rendering, so
/// no browser API is touched at hook-invocation time.
pub fn use_card_carousel() {
    Effect::new(move |_| {
        REGISTERED.with(|cell| {
            if cell.borrow().is_some() {
                return;
            }
            if let Some(registration) = register() {
                *cell.borrow_mut() = Some(registration);
            }
        });
    });
}

fn register() -> Option<Registration> {
    let target: EventTarget = document().dyn_into().ok()?;

    let click = window_event_listener(ev::click, handle_click);

    let scroll = Closure::new(handle_scroll);
    // Capture phase: scroll on an overflow-scroll track does not bubble, so a
    // bubble-phase document listener would never observe it.
    target
        .add_event_listener_with_callback_and_bool("scroll", scroll.as_ref().unchecked_ref(), true)
        .ok()?;

    Some(Registration {
        _click: click,
        _scroll: scroll,
    })
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "window event-listener callback takes the event by value"
)]
fn handle_click(event: ev::MouseEvent) {
    let Some(target) = event.target() else { return };
    let Ok(el) = target.dyn_into::<Element>() else {
        return;
    };
    let Some(button) = el.closest(CAROUSEL_NAV_BUTTON).ok().flatten() else {
        return;
    };

    event.stop_propagation();
    event.prevent_default();

    let Some(root) = button.closest(CAROUSEL_ROOT).ok().flatten() else {
        return;
    };
    let Some(track) = root.query_selector(CAROUSEL_TRACK).ok().flatten() else {
        return;
    };
    let Ok(buttons) = root.query_selector_all(CAROUSEL_NAV_BUTTON) else {
        return;
    };

    let is_prev = buttons
        .item(0)
        .and_then(|node| node.dyn_into::<Element>().ok())
        .is_some_and(|first| first == button);

    let delta = f64::from(track.client_width()) * if is_prev { -1.0 } else { 1.0 };
    // No explicit behavior argument: CSS `scroll-smooth` on the track animates
    // the move and dodges a WebKit bug where `behavior: 'smooth'` breaks
    // `scroll-snap-type: mandatory`.
    track.scroll_by_with_x_and_y(delta, 0.0);
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "scroll event-listener callback takes the event by value"
)]
fn handle_scroll(event: Event) {
    let Some(target) = event.target() else { return };
    let Ok(el) = target.dyn_into::<Element>() else {
        return;
    };
    let Some(track) = el.closest(CAROUSEL_TRACK).ok().flatten() else {
        return;
    };
    let Some(root) = track.closest(CAROUSEL_ROOT).ok().flatten() else {
        return;
    };

    let Ok(indicators) = root.query_selector_all(CAROUSEL_INDICATOR) else {
        return;
    };
    let Ok(buttons) = root.query_selector_all(CAROUSEL_NAV_BUTTON) else {
        return;
    };

    let index = active_index(track.scroll_left(), track.client_width());
    let count = indicators.length();

    sync_indicators(&indicators, index);

    set_aria_disabled(buttons.item(0), index == 0);
    set_aria_disabled(buttons.item(1), count > 0 && index >= count - 1);
}

/// Page index nearest the current scroll offset, clamped to a non-negative
/// whole number of pages. Returns `0` when the track has no measurable width.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "pages is positive and clamped to u32::MAX before the cast"
)]
fn active_index(scroll_left: i32, client_width: i32) -> u32 {
    if client_width <= 0 {
        return 0;
    }
    let pages = (f64::from(scroll_left) / f64::from(client_width)).round();
    if pages.is_sign_positive() {
        pages.min(f64::from(u32::MAX)) as u32
    } else {
        0
    }
}

fn sync_indicators(indicators: &NodeList, active: u32) {
    for i in 0..indicators.length() {
        let Some(node) = indicators.item(i) else {
            continue;
        };
        let Ok(dot) = node.dyn_into::<Element>() else {
            continue;
        };
        if i == active {
            _ = dot.set_attribute("aria-current", "true");
        } else {
            _ = dot.remove_attribute("aria-current");
        }
    }
}

fn set_aria_disabled(node: Option<Node>, disabled: bool) {
    let Some(node) = node else { return };
    let Ok(el) = node.dyn_into::<Element>() else {
        return;
    };
    if disabled {
        _ = el.set_attribute("aria-disabled", "true");
    } else {
        _ = el.remove_attribute("aria-disabled");
    }
}
