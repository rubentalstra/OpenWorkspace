//! shadcn Base UI components. Each module is transcribed 1:1 from the archived
//! Base UI source in `crates/ui/reference/shadcn/ui`. Children are declared here;
//! the crate root (`lib.rs`) re-exports their public items flat (`ui::Button`, …).
//! Grows wave by wave.

pub mod aspect_ratio;
pub mod avatar;
pub mod badge;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod input;
pub mod kbd;
pub mod label;
pub mod progress;
pub mod separator;
pub mod skeleton;
pub mod spinner;
pub mod switch;
pub mod textarea;
pub mod toggle;
