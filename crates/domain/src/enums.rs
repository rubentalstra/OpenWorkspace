//! Plain domain enums (no `sqlx`, no I/O).
//!
//! These mirror the native PostgreSQL `ENUM` types declared in the `enums`
//! migration, but carry no persistence derives. The `db` crate owns the
//! `#[derive(sqlx::Type)]` row enums and provides `From` conversions in both
//! directions.

/// Lifecycle status of a booking / occurrence (`booking_status`).
///
/// Terminal states each require their matching timestamp at the DB layer
/// (status↔timestamp `CHECK`s); see [`crate::state_machine`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BookingStatus {
    /// Reserved but not yet checked in.
    Booked,
    /// The user has checked in; the resource is actively occupied.
    CheckedIn,
    /// Checked out early or on time; the tail of the period is freed.
    CheckedOut,
    /// Auto-released after the check-in grace window elapsed without check-in.
    Released,
    /// Marked as a no-show.
    NoShow,
    /// Cancelled by a user before it became active.
    Cancelled,
}

impl BookingStatus {
    /// Returns `true` if this is a terminal status that admits no further
    /// transitions.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::CheckedOut | Self::Released | Self::NoShow | Self::Cancelled
        )
    }
}

/// Which kind of block an occurrence row represents (`occurrence_kind`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum OccurrenceKind {
    /// A user booking (has a parent `bookings` row).
    Booking,
    /// A materialised permanent assignment block.
    PermanentAssignment,
    /// A materialised blackout block.
    Blackout,
}

/// Per-series read-path visibility (`booking_visibility`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BookingVisibility {
    /// Visible to everyone.
    Public,
    /// Visible within the owning organization.
    OrgVisible,
    /// Visible only to the booker / bookee.
    Private,
}

/// Category of a bookable resource (`resource_kind`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceKind {
    /// A desk / workstation.
    Desk,
    /// A meeting room.
    Room,
    /// A parking space.
    Parking,
    /// A vehicle.
    Vehicle,
    /// A piece of equipment.
    Equipment,
}

/// How a resource's organization/team binding restricts its visibility
/// (`segmentation_mode`).
///
/// Drives [`crate::segmentation::visible`]. `db` owns the `sqlx` mapping.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SegmentationMode {
    /// No restriction: every viewer sees the resource.
    Open,
    /// Visible only to viewers in the resource's effective organization.
    ByOrganization,
    /// Visible only to viewers in the effective organization, and (when the
    /// resource carries an effective team) only to viewers in that team.
    ByOrganizationAndTeam,
}

/// Channel a booking was created through (`booking_source`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BookingSource {
    /// Created via the web UI.
    Web,
    /// Created via the public API.
    Api,
    /// Created by a bulk import.
    Import,
    /// Created by a delegate on behalf of a principal.
    Delegate,
}
