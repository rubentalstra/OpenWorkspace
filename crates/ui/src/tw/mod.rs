//! Class tooling: the `cn!` merge facade and the `slot!` / `variants!`
//! component-generating macros. All are `#[macro_export]`ed to the crate root, so
//! call sites use `crate::cn!`, `crate::slot!`, `crate::variants!`.

mod cn;
mod slot;
mod variants;
