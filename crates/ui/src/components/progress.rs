use crate::cn;
use leptos::prelude::*;

/// Progress — shadcn Base UI `progress`. A determinate bar; `value` is a percent
/// (0–100). The indicator width tracks `value`.
#[component]
pub fn Progress(
    #[prop(into, optional)] value: Signal<f64>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let pct = move || value.get().clamp(0.0, 100.0);
    view! {
        <div
            data-slot="progress"
            role="progressbar"
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow=move || pct().to_string()
            class=move || {
                cn!("cn-progress relative h-1 w-full overflow-hidden rounded-full", class.get())
            }
        >
            <div
                data-slot="progress-indicator"
                class="cn-progress-indicator h-full"
                style:width=move || format!("{}%", pct())
            />
        </div>
    }
}
