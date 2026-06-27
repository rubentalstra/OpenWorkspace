//! ui — the OpenWorkspace design-system kit on a stable Leptos stack: `tw_merge`
//! (via the first-party `cn!` facade and the `clx`/`variants`/`void` macros under
//! `tw/`) for class merging, and `leptos_icons` + `icondata` for Lucide glyphs.
//! None of rust-ui's own `icons`/`leptos_ui` crates (which force `leptos/nightly`)
//! are used, and all interactivity is pure Leptos — no JavaScript.
//!
//! Components are re-exported flat: `ui::Button`, `ui::Card`, `ui::Input`, … The
//! kit grows as each component is rewritten to the house standard (tasks #13–#23).

mod components;
mod tw;

// The vendored `variants!`/`cn!` macros expand `$crate::paste` / `$crate::tw_merge`,
// so those crates must be reachable at this crate's root.
#[doc(hidden)]
pub use paste;
#[doc(hidden)]
pub use tw_merge;

pub use components::hooks::*;
pub use components::ui::*;
