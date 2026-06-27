use leptos::prelude::*;
use std::time::Duration;

const DEFAULT_TIMEOUT_MS: u64 = 2000;

/// Writes text to the system clipboard and flags it as recently copied.
///
/// Returns `(copy, copied)`:
/// - `copy` accepts the text to place on the clipboard via the async Clipboard
///   API and sets `copied` to `true`.
/// - `copied` reports whether a copy happened within the last `timeout`; it
///   reverts to `false` once the timeout elapses. It stays `false` during
///   server rendering, where no clipboard exists.
///
/// `timeout` defaults to two seconds when `None`.
pub fn use_copy_clipboard(timeout: Option<Duration>) -> (impl Fn(&str) + Clone, ReadSignal<bool>) {
    let copied = RwSignal::new(false);
    let reset_after = timeout.unwrap_or(Duration::from_millis(DEFAULT_TIMEOUT_MS));

    let copy = move |text: &str| {
        // Runs only from a client event handler, so `window()` and the clipboard
        // are present; the returned Promise is left unawaited because the flag is
        // optimistic and a rejection cannot be surfaced through this signature.
        _ = window().navigator().clipboard().write_text(text);
        copied.set(true);

        // `try_set` because the reactive owner may be gone when the timer fires.
        set_timeout(move || _ = copied.try_set(false), reset_after);
    };

    (copy, copied.read_only())
}
