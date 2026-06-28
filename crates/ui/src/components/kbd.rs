use crate::slot;
use leptos::prelude::*;

slot! {
    /// Kbd — shadcn Base UI `kbd`. A keyboard-key hint.
    Kbd, kbd, "kbd",
    "cn-kbd pointer-events-none inline-flex items-center justify-center select-none"
}
slot! {
    /// Groups several `Kbd`s into a single chord (e.g. ⌘ + K).
    KbdGroup, kbd, "kbd-group", "cn-kbd-group inline-flex items-center"
}
