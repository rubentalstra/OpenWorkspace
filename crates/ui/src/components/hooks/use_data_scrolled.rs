use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast as _;
use leptos::wasm_bindgen::closure::Closure;

/// Element id of the optional scroll container that [`use_data_scrolled`]
/// watches in preference to the window. Set `id=DATA_SCROLL_TARGET` on the
/// scrollable element to track its `scrollTop` instead of the document scroll
/// position.
pub const DATA_SCROLL_TARGET: &str = "data-scroll-target";

/// Tracks whether the page (or the [`DATA_SCROLL_TARGET`] container, when
/// present) has scrolled past `threshold_px`. Returns a signal that is `false`
/// during SSR and on first client paint, then updates as the user scrolls.
///
/// All browser access lives inside an [`Effect`] so it never runs on the server.
pub fn use_data_scrolled(threshold_px: u32) -> RwSignal<bool> {
    let scrolled = RwSignal::new(false);

    Effect::new(move |_| {
        let threshold = f64::from(threshold_px);
        let container = document().get_element_by_id(DATA_SCROLL_TARGET);

        let read_position = {
            let container = container.clone();
            move || match container {
                Some(ref el) => f64::from(el.scroll_top()),
                None => scroll_position(),
            }
        };

        scrolled.set(read_position() > threshold);

        // `scroll` does not bubble, so a container needs a direct listener
        // (detached when the element leaves the DOM); otherwise watch the window.
        if let Some(el) = container {
            attach_scroll_listener(&el, move || scrolled.set(read_position() > threshold));
        } else {
            let handle = window_event_listener(leptos::ev::scroll, move |_| {
                scrolled.set(read_position() > threshold);
            });
            on_cleanup(move || handle.remove());
        }
    });

    scrolled
}

/// Registers a `scroll` listener on `element`; the browser closure is owned for
/// the element's lifetime and freed with it.
fn attach_scroll_listener(element: &web_sys::Element, mut on_scroll: impl FnMut() + 'static) {
    let callback = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| on_scroll());
    _ = element.add_event_listener_with_callback("scroll", callback.as_ref().unchecked_ref());
    callback.forget();
}

/// Mirrors the body's `padding-right` onto a fixed nav element so it does not
/// shift when a scroll lock compensates for the removed scrollbar by padding the
/// body.
fn sync_header_padding_with_body(padding: &str) {
    _ = (|| -> Option<()> {
        let element = document()
            .query_selector("[data-name='NavMenuFixed']")
            .ok()??;
        let header = element.dyn_ref::<web_sys::HtmlElement>()?;

        if padding.is_empty() || padding == "0px" {
            header.style().remove_property("padding-right").ok()?;
        } else {
            header.style().set_property("padding-right", padding).ok()?;
        }
        Some(())
    })();
}

/// Reads the current scroll position. When the body has been pinned with
/// `position: fixed` and a negative `top` (e.g. by a scroll lock), the body no
/// longer scrolls, so the offset is recovered from that `top` value instead of
/// `window.scrollY`.
fn scroll_position() -> f64 {
    let Some(body) = document().body() else {
        return window().scroll_y().unwrap_or_default();
    };

    let style = body.style();
    let is_fixed = style.get_property_value("position").ok().as_deref() == Some("fixed");
    let padding = style
        .get_property_value("padding-right")
        .unwrap_or_default();

    sync_header_padding_with_body(&padding);

    if is_fixed {
        style
            .get_property_value("top")
            .ok()
            .and_then(|top| top.strip_suffix("px")?.strip_prefix('-')?.parse().ok())
            .unwrap_or_default()
    } else {
        window().scroll_y().unwrap_or_default()
    }
}
