use crate::cn;
use leptos::html;
use leptos::prelude::*;

const PROGRESS_BASE: &str = "relative h-2 w-full overflow-hidden rounded-full bg-secondary";

/// Determinate progress bar. `value` is the current amount filled relative to
/// `max`; the inner indicator translates to reflect the percentage complete.
#[component]
pub fn Progress(
    #[prop(into, optional)] value: Signal<f64>,
    #[prop(into, default = 100.0.into())] max: Signal<f64>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
) -> impl IntoView {
    let pct = move || {
        let max = max.get();
        if max <= 0.0 {
            0.0
        } else {
            (value.get() / max * 100.0).clamp(0.0, 100.0)
        }
    };
    let indicator_style = move || format!("transform: translateX(-{}%)", 100.0 - pct());

    view! {
        <div
            node_ref=node_ref
            data-name="Progress"
            role="progressbar"
            aria-valuemin="0"
            aria-valuemax=move || max.get().to_string()
            aria-valuenow=move || value.get().to_string()
            class=move || cn!(PROGRESS_BASE, class.get())
        >
            <div
                data-name="ProgressIndicator"
                class="h-full w-full flex-1 bg-primary transition-all duration-300 ease-in-out"
                style=indicator_style
            />
        </div>
    }
}
