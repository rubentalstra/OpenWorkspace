use std::sync::atomic::{AtomicU64, Ordering};

/// Embedded in every generated id so collisions with hand-written ids are
/// unlikely. Must not contain `/` or `-` so the value stays valid in a CSS
/// `view-transition-name` and as an HTML id selector.
const PREFIX: &str = "rust_ui";

/// Returns a process-unique id suitable for an element `id` attribute,
/// formatted as `_rust_ui_<n>`.
pub fn use_random_id() -> String {
    format!("_{PREFIX}_{}", next_id())
}

/// Returns a process-unique id prefixed with `element`, formatted as
/// `<element>_rust_ui_<n>`. Use to tie generated ids to a known element name.
pub fn use_random_id_for(element: &str) -> String {
    format!("{element}_{PREFIX}_{}", next_id())
}

/// Returns a CSS declaration assigning a process-unique
/// `view-transition-name`, e.g. `view-transition-name: _rust_ui_<n>`.
pub fn use_random_transition_name() -> String {
    format!("view-transition-name: {}", use_random_id())
}

/// Monotonic source of uniqueness. A plain incrementing counter is enough: ids
/// only need to be distinct within the process, not unpredictable.
fn next_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
