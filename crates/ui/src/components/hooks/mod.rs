//! Reusable Leptos hooks — pure Rust, SSR-safe (browser side effects run only in
//! `Effect`s). Three hooks are re-added with the components they pair with:
//! `use_cell_edit` and `use_card_carousel` with the data grid and card carousel,
//! `use_input_otp` with `InputOTP`.
mod use_can_scroll_vertical;
mod use_card_carousel;
mod use_cell_edit;
mod use_copy_clipboard;
mod use_data_scrolled;
mod use_form;
mod use_history;
mod use_horizontal_scroll;
mod use_input_otp;
mod use_is_mobile;
mod use_lock_body_scroll;
mod use_locks;
mod use_media_query;
mod use_pagination;
mod use_press_hold;
mod use_random;
mod use_theme_mode;
mod use_virtual_scroll;

pub use use_can_scroll_vertical::*;
pub use use_card_carousel::*;
pub use use_cell_edit::*;
pub use use_copy_clipboard::*;
pub use use_data_scrolled::*;
pub use use_form::*;
pub use use_history::*;
pub use use_horizontal_scroll::*;
pub use use_input_otp::*;
pub use use_is_mobile::*;
pub use use_lock_body_scroll::*;
pub use use_locks::*;
pub use use_media_query::*;
pub use use_pagination::*;
pub use use_press_hold::*;
pub use use_random::*;
pub use use_theme_mode::*;
pub use use_virtual_scroll::*;
