//! Reusable Leptos hooks — pure Rust, SSR-safe (browser side effects run only in
//! `Effect`s). Five hooks are re-added with the components they pair with:
//! `use_pagination`, `use_virtual_scroll`, `use_cell_edit` with the date & data
//! cluster, `use_input_otp` with `InputOTP`, `use_card_carousel` with the
//! card carousel.
mod use_can_scroll_vertical;
mod use_copy_clipboard;
mod use_data_scrolled;
mod use_form;
mod use_history;
mod use_horizontal_scroll;
mod use_is_mobile;
mod use_lock_body_scroll;
mod use_locks;
mod use_media_query;
mod use_press_hold;
mod use_random;
mod use_theme_mode;

pub use use_can_scroll_vertical::*;
pub use use_copy_clipboard::*;
pub use use_data_scrolled::*;
pub use use_form::*;
pub use use_history::*;
pub use use_horizontal_scroll::*;
pub use use_is_mobile::*;
pub use use_lock_body_scroll::*;
pub use use_locks::*;
pub use use_media_query::*;
pub use use_press_hold::*;
pub use use_random::*;
pub use use_theme_mode::*;
