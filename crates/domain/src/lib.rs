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

mod access;
mod booking;
mod model;

// Module re-exports preserve the public paths (`domain::authz::…`,
// `domain::recurrence::…`, etc.) and the internal `crate::ids`/`crate::enums`/
// `crate::time_range` paths, independent of where each file now lives.
pub use access::{authz, segmentation};
pub use booking::{recurrence, state_machine, validation};
pub use model::{enums, ids, time_range};

pub use enums::{
    BookingSource, BookingStatus, BookingVisibility, OccurrenceKind, ResourceKind, ResourceStatus,
    SegmentationMode, SpaceState,
};
pub use ids::{
    AssetId, BookingId, EquipmentItemId, FloorZoneId, LocationId, OccurrenceId, OrganizationId,
    ResourceId, RoleId, TeamId, UserId,
};
pub use time_range::TimeRange;
