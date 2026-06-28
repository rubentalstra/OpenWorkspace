use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;

use crate::constants::Pagination;

/// Extra rows rendered above and below the viewport for smooth scrolling.
const BUFFER_ROWS: usize = 5;

/// Visible row range and total height for a virtualized list.
#[derive(Clone, Copy)]
pub struct VirtualScrollState {
    /// First row index to render.
    pub start_index: Memo<usize>,
    /// One past the last row index to render.
    pub end_index: Memo<usize>,
    /// Total height of the virtual container, in pixels.
    pub total_height: Signal<usize>,
}

/// Reads the virtual-scroll state provided by a parent virtualized grid, or
/// `None` outside one.
pub fn use_virtual_scroll_context() -> Option<VirtualScrollState> {
    use_context::<VirtualScrollState>()
}

/// Virtual scrolling for large lists: renders only the rows in the viewport plus
/// a small buffer. `container_ref` is the scroll container; `total_rows` is the
/// reactive row count. All DOM access runs inside an effect, so it is inert on
/// the server.
pub fn use_virtual_scroll(
    container_ref: NodeRef<html::Div>,
    total_rows: Signal<usize>,
) -> VirtualScrollState {
    let scroll_top = RwSignal::new(0usize);
    let container_height = RwSignal::new(600usize);

    // Guards the closures below: they outlive this scope (handed to the browser),
    // so they must not touch disposed signals after the component unmounts.
    let mounted = Arc::new(AtomicBool::new(true));
    let mounted_cleanup = Arc::clone(&mounted);
    on_cleanup(move || mounted_cleanup.store(false, Ordering::SeqCst));

    let mounted_effect = Arc::clone(&mounted);
    Effect::new(move || {
        let Some(el) = container_ref.get() else {
            return;
        };

        // Measure after layout: a `requestAnimationFrame` callback runs post-paint,
        // when `client_height` is correct (it reads 0 before first layout).
        let mounted_raf = Arc::clone(&mounted_effect);
        let measure = Closure::<dyn Fn()>::new(move || {
            if !mounted_raf.load(Ordering::SeqCst) {
                return;
            }
            if let Some(el) = container_ref.get_untracked() {
                container_height.set(usize::try_from(el.client_height()).unwrap_or(0));
            }
        });
        _ = window().request_animation_frame(measure.as_ref().unchecked_ref());
        measure.forget();

        let mounted_scroll = Arc::clone(&mounted_effect);
        let on_scroll = Closure::<dyn Fn()>::new(move || {
            if !mounted_scroll.load(Ordering::SeqCst) {
                return;
            }
            if let Some(el) = container_ref.get_untracked() {
                scroll_top.set(usize::try_from(el.scroll_top()).unwrap_or(0));
                container_height.set(usize::try_from(el.client_height()).unwrap_or(0));
            }
        });
        _ = el.add_event_listener_with_callback("scroll", on_scroll.as_ref().unchecked_ref());
        on_scroll.forget();
    });

    let start_index =
        Memo::new(move |_| (scroll_top.get() / Pagination::ROW_HEIGHT).saturating_sub(BUFFER_ROWS));

    let end_index = Memo::new(move |_| {
        let visible_rows = (container_height.get() / Pagination::ROW_HEIGHT) + 1;
        let start = scroll_top.get() / Pagination::ROW_HEIGHT;
        (start + visible_rows + BUFFER_ROWS * 2).min(total_rows.get())
    });

    let total_height = Signal::derive(move || total_rows.get() * Pagination::ROW_HEIGHT);

    VirtualScrollState {
        start_index,
        end_index,
        total_height,
    }
}
