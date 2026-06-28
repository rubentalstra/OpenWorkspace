//! First-party helpers backing the data components: date math, URL query state,
//! the country list and phone-number formatting. `Country`, `PhoneNumber` and
//! `PhoneFormat` are re-exported at the crate root; the rest are crate-internal.
pub(crate) mod country;
pub(crate) mod date;
pub(crate) mod phone_number;
pub(crate) mod query;
