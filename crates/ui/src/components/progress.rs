use crate::cn;
use leptos::prelude::*;

/// Progress — shadcn Base UI `progress`. A determinate bar; `value` is a percent
/// (0–100). Mirrors the reference structure (root → track → indicator) so the nova
/// layer styles the muted track and primary indicator; the indicator width tracks
/// `value`.
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
            class=move || cn!("cn-progress-root flex flex-wrap gap-3", class.get())
        >
            <div
                data-slot="progress-track"
                class="cn-progress-track relative flex w-full items-center overflow-x-hidden"
            >
                <div
                    data-slot="progress-indicator"
                    class="cn-progress-indicator h-full transition-all"
                    style:width=move || format!("{}%", pct())
                />
            </div>
        </div>
    }
}
