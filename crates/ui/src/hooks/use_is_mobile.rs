use leptos::prelude::*;

use super::use_media_query::use_media_query;

/// Viewport width, in pixels, at and above which the layout is treated as
/// non-mobile. Matches shadcn's `use-mobile` breakpoint and Tailwind's `md`.
pub const MOBILE_BREAKPOINT: u32 = 768;

/// Reactive hook reporting whether the viewport is narrower than the mobile
/// breakpoint.
///
/// Backed by [`use_media_query`], so the returned [`Signal`] updates as the
/// viewport is resized. During server rendering it reports `false` and is
/// populated on the client once the media query is evaluated.
pub fn use_is_mobile() -> Signal<bool> {
    use_media_query(&format!("(max-width: {}px)", MOBILE_BREAKPOINT - 1))
}
