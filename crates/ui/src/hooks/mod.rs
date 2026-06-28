//! Reactive, SSR-safe hooks shared by the components and the app. Children are
//! declared here; the crate root re-exports their public items flat. Browser-only
//! work runs inside `Effect`s, so server render stays identical to first paint.

pub mod use_dismiss;
pub mod use_is_mobile;
pub mod use_media_query;
pub mod use_theme_mode;
