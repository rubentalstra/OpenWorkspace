/// Merges Tailwind classes, resolving conflicts so later utilities win and
/// duplicates collapse. First-party facade over `tw_merge` — component code uses
/// `cn!` rather than calling the vendor macro directly.
///
/// ```ignore
/// let class = cn!("px-2 py-1", active.then_some("bg-primary").unwrap_or_default());
/// ```
#[macro_export]
macro_rules! cn {
    ($($class:expr),* $(,)?) => {
        $crate::tw_merge::tw_merge!($($class),*)
    };
}
