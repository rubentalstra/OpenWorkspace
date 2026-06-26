//! Browser hydration entry point.

// The #[wasm_bindgen] entry point expands to unsafe code; this is the one crate
// that opts out of the workspace-wide `unsafe_code = "deny"`.
#![allow(unsafe_code)]

use app::App;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
