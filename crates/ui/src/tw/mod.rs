//! Styling primitives. `cn!` is the first-party class-merge facade over `tw_merge`;
//! the `clx`/`variants`/`void` macros (vendored from `leptos_ui`, MIT) emit
//! `#[component]` fns with reactive, merged classes. They avoid `leptos_ui`'s
//! mandatory `leptos/nightly` pin; `$crate::paste`/`$crate::tw_merge` resolve to
//! the re-exports in lib.rs.
pub(crate) mod clx;
mod cn;
pub(crate) mod variants;
