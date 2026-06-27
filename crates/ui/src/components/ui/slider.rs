use crate::cn;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

const SLIDER_ROOT: &str =
    "relative flex w-full touch-none items-center select-none data-[disabled]:opacity-50";
const SLIDER_TRACK: &str = "bg-muted relative h-1.5 w-full grow overflow-hidden rounded-full";
const SLIDER_RANGE: &str = "bg-primary absolute h-full";
const SLIDER_THUMB: &str = "border-primary bg-background ring-ring/50 absolute top-1/2 size-4 -translate-x-1/2 -translate-y-1/2 rounded-full border shadow-sm transition-[color,box-shadow] peer-focus-visible:ring-4 peer-disabled:pointer-events-none";
const SLIDER_INPUT: &str = "peer absolute inset-0 z-10 h-full w-full cursor-pointer appearance-none bg-transparent opacity-0 disabled:cursor-not-allowed";

/// Range slider built on a native `<input type="range">`, so dragging, arrow
/// keys, Home/End and Page Up/Down work with zero JavaScript. The native control
/// is visually hidden but covers the track, while the styled track, range and
/// thumb sit beneath it.
///
/// The current position lives in `value` (an [`RwSignal<f64>`]) so call sites can
/// read and drive it reactively; `min`, `max` and `step` bound the scale. Because
/// the native range element owns the slider role and its `aria-value*` state, the
/// browser reports the live position to assistive tech automatically. An
/// `on:input` handler mirrors the parsed value into the signal on the client only.
#[component]
pub fn Slider(
    #[prop(into, default = RwSignal::new(0.0))] value: RwSignal<f64>,
    #[prop(into, default = 0.0.into())] min: Signal<f64>,
    #[prop(into, default = 100.0.into())] max: Signal<f64>,
    #[prop(into, default = 1.0.into())] step: Signal<f64>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Input>,
) -> impl IntoView {
    let percent = move || {
        let (lo, hi) = (min.get(), max.get());
        let span = hi - lo;
        if span <= 0.0 {
            0.0
        } else {
            ((value.get() - lo) / span * 100.0).clamp(0.0, 100.0)
        }
    };
    let range_style = move || format!("width: {}%", percent());
    let thumb_style = move || format!("left: {}%", percent());

    let on_input = move |ev: ev::Event| {
        let Some(input) = ev
            .target()
            .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
        else {
            return;
        };
        value.set(input.value_as_number());
    };

    view! {
        <div
            data-name="Slider"
            data-disabled=move || disabled.get().then_some("")
            class=move || cn!(SLIDER_ROOT, class.get())
        >
            <input
                node_ref=node_ref
                data-name="SliderInput"
                type="range"
                class=SLIDER_INPUT
                prop:value=move || value.get().to_string()
                min=move || min.get().to_string()
                max=move || max.get().to_string()
                step=move || step.get().to_string()
                disabled=move || disabled.get()
                on:input=on_input
            />
            <div data-name="SliderTrack" class=SLIDER_TRACK>
                <div data-name="SliderRange" class=SLIDER_RANGE style=range_style />
            </div>
            <div data-name="SliderThumb" class=SLIDER_THUMB style=thumb_style />
        </div>
    }
}
