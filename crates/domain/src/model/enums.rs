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

/// The displayed availability state of a bookable space — the six UI states of
/// architecture plan §3.2 (free, partially free, not free, temporarily blocked,
/// permanent user, cannot be booked).
///
/// This is a **presentation read-model**, not a stored value and not a booking
/// lifecycle. It is *projected* (P13) for a viewed time window from a resource's
/// live occupancy ([`OccurrenceKind`] — bookings, permanent assignments, blackouts),
/// its capacity, and whether it is bookable at all; it is distinct from
/// [`BookingStatus`] (the lifecycle of one reservation). A desk with a checked-in
/// booking projects to [`SpaceState::NotFree`]; "checked in" is a property of the
/// booking, never of the space.
///
/// `floorplan` renders this as the `data-state` attribute. Per the plan the state
/// must be conveyed "by icon and label, never by colour alone" (WCAG): the theming
/// layer pairs each state with an icon and an accessible label.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum SpaceState {
    /// Fully available to book.
    #[default]
    Free,
    /// A multi-capacity (shared) resource with some — but not all — capacity taken.
    PartiallyFree,
    /// Fully taken for the window (includes an occupied, checked-in desk).
    NotFree,
    /// Out of service for a bounded period (blackout / maintenance).
    TemporarilyBlocked,
    /// Held by a standing / permanent assignment.
    PermanentUser,
    /// On the plan but never bookable (display-only).
    CannotBeBooked,
}

impl SpaceState {
    /// The stable key for this state: both the `data-state` attribute value and the
    /// i18n message id for its label. Lowercase `snake_case`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::PartiallyFree => "partially_free",
            Self::NotFree => "not_free",
            Self::TemporarilyBlocked => "temporarily_blocked",
            Self::PermanentUser => "permanent_user",
            Self::CannotBeBooked => "cannot_be_booked",
        }
    }

    /// Projects a display state from a resource's occupancy over a viewed window —
    /// the **single source of truth** for how [`OccurrenceKind`], capacity and
    /// bookability map onto a `SpaceState`. P13 supplies `covering`: the kinds of the
    /// occurrences that overlap the window.
    ///
    /// Precedence, highest first: not bookable → `CannotBeBooked`; any blackout →
    /// `TemporarilyBlocked` (maintenance makes the space unusable for everyone,
    /// including a permanent holder); any permanent assignment → `PermanentUser`;
    /// then booked seats versus `capacity` → `NotFree` / `PartiallyFree` / `Free`.
    /// Each booking occurrence consumes one seat; a `capacity` of 0 is treated as 1.
    #[must_use]
    pub fn project(bookable: bool, capacity: u32, covering: &[OccurrenceKind]) -> Self {
        if !bookable {
            return Self::CannotBeBooked;
        }
        if covering.contains(&OccurrenceKind::Blackout) {
            return Self::TemporarilyBlocked;
        }
        if covering.contains(&OccurrenceKind::PermanentAssignment) {
            return Self::PermanentUser;
        }
        let booked = u32::try_from(
            covering
                .iter()
                .filter(|kind| matches!(kind, OccurrenceKind::Booking))
                .count(),
        )
        .unwrap_or(u32::MAX);
        if booked == 0 {
            Self::Free
        } else if booked >= capacity.max(1) {
            Self::NotFree
        } else {
            Self::PartiallyFree
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_state_defaults_free_and_keys_are_snake_case() {
        assert_eq!(SpaceState::default(), SpaceState::Free);
        assert_eq!(SpaceState::Free.as_str(), "free");
        assert_eq!(SpaceState::PartiallyFree.as_str(), "partially_free");
        assert_eq!(SpaceState::NotFree.as_str(), "not_free");
        assert_eq!(
            SpaceState::TemporarilyBlocked.as_str(),
            "temporarily_blocked"
        );
        assert_eq!(SpaceState::PermanentUser.as_str(), "permanent_user");
        assert_eq!(SpaceState::CannotBeBooked.as_str(), "cannot_be_booked");
    }

    #[test]
    fn space_state_projection_follows_precedence_and_capacity() {
        use OccurrenceKind::{Blackout, Booking, PermanentAssignment};

        // Bookability gate wins over any occupancy.
        assert_eq!(
            SpaceState::project(false, 1, &[Booking]),
            SpaceState::CannotBeBooked
        );
        // Blackout outranks a permanent assignment and bookings.
        assert_eq!(
            SpaceState::project(true, 4, &[Booking, PermanentAssignment, Blackout]),
            SpaceState::TemporarilyBlocked
        );
        // Permanent assignment outranks bookings.
        assert_eq!(
            SpaceState::project(true, 4, &[Booking, PermanentAssignment]),
            SpaceState::PermanentUser
        );
        // Capacity-driven booking states.
        assert_eq!(SpaceState::project(true, 1, &[]), SpaceState::Free);
        assert_eq!(
            SpaceState::project(true, 1, &[Booking]),
            SpaceState::NotFree
        );
        assert_eq!(
            SpaceState::project(true, 4, &[Booking]),
            SpaceState::PartiallyFree
        );
        assert_eq!(
            SpaceState::project(true, 4, &[Booking, Booking, Booking, Booking]),
            SpaceState::NotFree
        );
        // A zero capacity is treated as a single seat.
        assert_eq!(
            SpaceState::project(true, 0, &[Booking]),
            SpaceState::NotFree
        );
    }
}
