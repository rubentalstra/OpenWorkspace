use crate::cn;
use leptos::html;
use leptos::prelude::*;

const IMAGE_BASE: &str = "block max-w-full h-auto object-cover data-[error=true]:hidden";

/// Styled `<img>` with optional fixed aspect ratio and load-error tracking.
///
/// Set `attr:src`, `attr:alt`, `attr:srcset`, `attr:sizes`, `attr:loading`,
/// `attr:decoding` and any other native attribute at the call site — they all
/// forward to the underlying element. Pass `aspect` to lock a width-to-height
/// ratio (the box reserves space before the bitmap loads, avoiding layout
/// shift). When the source fails to load the element is hidden and exposes
/// `data-error="true"` as a styling slot, letting a sibling fallback take over;
/// `on_error` fires the same moment for call sites that need to react in Rust.
#[component]
pub fn Image(
    /// Width-to-height ratio enforced via `aspect-ratio`; when omitted the
    /// intrinsic ratio of the loaded bitmap applies.
    #[prop(into, optional)]
    aspect: Option<Signal<f64>>,
    /// Invoked once when the source fails to load.
    #[prop(optional)]
    on_error: Option<Callback<()>>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Img>,
) -> impl IntoView {
    let errored = RwSignal::new(false);
    let style = move || aspect.map(|aspect| format!("aspect-ratio: {}", aspect.get()));

    view! {
        <img
            node_ref=node_ref
            data-name="Image"
            data-error=move || errored.get().then_some("true")
            class=move || cn!(IMAGE_BASE, class.get())
            style=style
            on:error=move |_| {
                if !errored.get() {
                    errored.set(true);
                    if let Some(on_error) = on_error {
                        on_error.run(());
                    }
                }
            }
        />
    }
}
