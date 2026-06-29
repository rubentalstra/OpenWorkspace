//! Storage-key generation. Keys are opaque and independent of the database
//! `assets.id` (generated before the row is inserted), so the bytes can be stored
//! and only then recorded. Time-ordered (`uuidv7`) for friendly object listing.

/// A fresh object key for an uploaded original, e.g. `assets/0190f3c2-…`.
#[must_use]
pub fn new_storage_key() -> String {
    format!("assets/{}", uuid::Uuid::now_v7())
}

/// The derived key for a variant (e.g. the thumbnail) of an original key.
#[must_use]
pub fn thumbnail_key(original: &str) -> String {
    format!("{original}/thumb")
}
