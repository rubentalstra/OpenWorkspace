//! domain — the pure, I/O-free booking engine.
//!
//! This crate carries the headless booking logic: plain domain enums and
//! newtype IDs, the half-open [`TimeRange`], DST-correct recurrence expansion
//! ([`recurrence::expand_series`]), the occurrence state machine
//! ([`state_machine::transition`]), and request validation
//! ([`validation::validate_booking`]).
//!
//! It has **no** `sqlx` and performs **no** I/O: the clock is always injected.
//! The `db` crate owns the persistence-mapped row enums and converts to/from
//! the plain enums re-exported here.

pub mod authz;
mod enums;
mod ids;
pub mod recurrence;
pub mod segmentation;
pub mod state_machine;
mod time_range;
pub mod validation;

pub use enums::{
    BookingSource, BookingStatus, BookingVisibility, OccurrenceKind, ResourceKind, SegmentationMode,
};
pub use ids::{
    BookingId, FloorZoneId, LocationId, OccurrenceId, OrganizationId, ResourceId, RoleId, TeamId,
    UserId,
};
pub use time_range::TimeRange;
