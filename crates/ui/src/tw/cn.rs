//! `cn!` — the first-party class-merge facade over `tw_merge`, mirroring shadcn's
//! `cn()` (`twMerge(clsx(...))`). Later classes win on Tailwind conflicts, so a
//! caller's `class` can always override a component's defaults.

/// Merge Tailwind class fragments, de-duplicating conflicts (last wins).
///
/// Accepts string literals, `String`, and `Option<_>` fragments (a `None` is
/// skipped), e.g. `cn!("cn-button", active.then_some("cn-button-active"), class)`.
#[macro_export]
macro_rules! cn {
    ($($class:expr),* $(,)?) => {
        $crate::tw_merge::tw_merge!($($class),*)
    };
}
