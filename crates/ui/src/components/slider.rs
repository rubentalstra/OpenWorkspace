use crate::cn;
use leptos::prelude::*;
use web_sys::{KeyboardEvent, PointerEvent};

/// Slider — shadcn Base UI `slider` (single-thumb, horizontal). Controlled: read
/// `value`, react to `on_change`. Supports pointer-drag (with pointer capture) and
/// keyboard (arrows / Home / End). `min`/`max`/`step` default to 0/100/1.
#[component]
pub fn Slider(
    #[prop(into, optional)] value: Signal<f64>,
    #[prop(optional)] on_change: Option<Callback<f64>>,
    #[prop(default = 0.0)] min: f64,
    #[prop(default = 100.0)] max: f64,
    #[prop(default = 1.0)] step: f64,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let control_ref = NodeRef::<leptos::html::Div>::new();
    let dragging = RwSignal::new(false);

    let pct = move || {
        let span = (max - min).max(f64::EPSILON);
        (((value.get() - min) / span) * 100.0).clamp(0.0, 100.0)
    };

    let emit = move |raw: f64| {
        let snapped = if step > 0.0 {
            min + ((raw - min) / step).round() * step
        } else {
            raw
        };
        if let Some(cb) = on_change {
            cb.run(snapped.clamp(min, max));
        }
    };

    let value_at = move |client_x: f64| {
        let Some(el) = control_ref.get_untracked() else {
            return;
        };
        let rect = el.get_bounding_client_rect();
        if rect.width() <= 0.0 {
            return;
        }
        let ratio = ((client_x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        emit(min + ratio * (max - min));
    };

    let on_pointer_down = move |ev: PointerEvent| {
        dragging.set(true);
        if let Some(el) = control_ref.get_untracked() {
            _ = el.set_pointer_capture(ev.pointer_id());
        }
        value_at(f64::from(ev.client_x()));
    };
    let on_pointer_move = move |ev: PointerEvent| {
        if dragging.get_untracked() {
            value_at(f64::from(ev.client_x()));
        }
    };
    let on_pointer_up = move |ev: PointerEvent| {
        dragging.set(false);
        if let Some(el) = control_ref.get_untracked() {
            _ = el.release_pointer_capture(ev.pointer_id());
        }
    };
    let on_key = move |ev: KeyboardEvent| {
        let current = value.get_untracked();
        let next = match ev.key().as_str() {
            "ArrowRight" | "ArrowUp" => current + step,
            "ArrowLeft" | "ArrowDown" => current - step,
            "Home" => min,
            "End" => max,
            _ => return,
        };
        ev.prevent_default();
        emit(next);
    };

    view! {
        <div
            data-slot="slider"
            data-orientation="horizontal"
            class=move || cn!("data-horizontal:w-full data-vertical:h-full", class.get())
        >
            <div
                node_ref=control_ref
                data-orientation="horizontal"
                class="cn-slider relative flex w-full touch-none items-center select-none data-disabled:opacity-50"
                on:pointerdown=on_pointer_down
                on:pointermove=on_pointer_move
                on:pointerup=on_pointer_up
            >
                <div
                    data-slot="slider-track"
                    data-orientation="horizontal"
                    class="cn-slider-track relative grow overflow-hidden select-none"
                >
                    <div
                        data-slot="slider-range"
                        data-orientation="horizontal"
                        class="cn-slider-range select-none data-horizontal:h-full data-vertical:w-full"
                        style:width=move || format!("{}%", pct())
                    />
                </div>
                <span
                    data-slot="slider-thumb"
                    data-orientation="horizontal"
                    role="slider"
                    tabindex="0"
                    aria-valuemin=min.to_string()
                    aria-valuemax=max.to_string()
                    aria-valuenow=move || value.get().to_string()
                    class="cn-slider-thumb block shrink-0 select-none disabled:pointer-events-none disabled:opacity-50"
                    style:position="absolute"
                    style:left=move || format!("{}%", pct())
                    style:transform="translateX(-50%)"
                    on:keydown=on_key
                />
            </div>
        </div>
    }
}
