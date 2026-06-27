use crate::cn;
use leptos::html;
use leptos::prelude::*;

const SHIMMER_BASE: &str = "relative isolate overflow-hidden \
[--shimmer-color:rgba(255,255,255,0.15)] [--shimmer-bg:rgba(255,255,255,0.08)] \
[--shimmer-duration:1.5s]";

const SHIMMER_WAVE: &str = "pointer-events-none absolute inset-0 -translate-x-full \
bg-[var(--shimmer-bg)] bg-[linear-gradient(90deg,transparent,var(--shimmer-color),transparent)] \
[animation:ow_shimmer_var(--shimmer-duration)_infinite] motion-reduce:animate-none";

/// CSS keyframes powering the shimmer sweep. Emitted once per [`Shimmer`];
/// identical blocks collapse harmlessly. Driving the animation from CSS keeps
/// the component script-free and SSR-safe.
const SHIMMER_KEYFRAMES: &str = "@keyframes ow_shimmer { to { transform: translateX(100%); } }";

/// Loading wrapper that sweeps a highlight across its children while `loading`
/// is true and reveals them unchanged once it is false. The sweep is pure CSS:
/// an absolutely-positioned overlay animates `translateX`.
///
/// Colour and timing are CSS custom properties on the root, so callers tune them
/// through `class` or inline `style` (`[--shimmer-duration:2s]`,
/// `[--shimmer-color:theme(colors.white/20%)]`). Respects
/// `prefers-reduced-motion`. Native attributes, events and bindings forward to
/// the root.
#[component]
pub fn Shimmer(
    /// Whether the sweep is active. When false the children render without an
    /// overlay.
    #[prop(into)]
    loading: Signal<bool>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            data-name="Shimmer"
            data-loading=move || loading.get().then_some("true")
            class=move || cn!(SHIMMER_BASE, class.get())
        >
            <style>{SHIMMER_KEYFRAMES}</style>
            {children()}
            <Show when=move || loading.get()>
                <span aria-hidden="true" data-name="ShimmerWave" class=SHIMMER_WAVE />
            </Show>
        </div>
    }
}
