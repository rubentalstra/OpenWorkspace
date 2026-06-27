use leptos::prelude::*;

use super::use_media_query::use_media_query;

/// Viewport width, in pixels, at and above which the layout is treated as
/// non-mobile. Matches Tailwind's `md` breakpoint.
pub const MOBILE_BREAKPOINT: u32 = 768;

/// Reactive hook that reports whether the viewport is narrower than the mobile
/// breakpoint.
///
/// Backed by [`use_media_query`], so the returned [`Signal`] updates as the
/// viewport is resized. During server rendering — where no viewport exists — it
/// reports `false` and is populated on the client once the media query is
/// evaluated.
///
/// # Example
/// ```ignore
/// let is_mobile = use_is_mobile();
///
/// view! {
///     {move || if is_mobile.get() {
///         view! { <Drawer>...</Drawer> }.into_any()
///     } else {
///         view! { <Dialog>...</Dialog> }.into_any()
///     }}
/// }
/// ```
pub fn use_is_mobile() -> Signal<bool> {
    use_media_query(&format!("(max-width: {}px)", MOBILE_BREAKPOINT - 1))
}
