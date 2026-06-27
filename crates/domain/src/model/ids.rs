//! Strongly-typed newtype identifiers.
//!
//! Each wraps a [`Uuid`] so the type system prevents passing, say, a
//! [`ResourceId`] where a [`BookingId`] is expected. These are I/O-free: the
//! `db` crate maps row `uuid::Uuid` values to/from these at its boundary and
//! owns any `sqlx` derives.

use uuid::Uuid;

/// Declares an opaque `Uuid` newtype with the common accessor surface.
macro_rules! uuid_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub struct $name(Uuid);

        impl $name {
            /// Wraps a raw [`Uuid`] as this identifier.
            #[must_use]
            pub const fn new(id: Uuid) -> Self {
                Self(id)
            }

            /// Returns the underlying [`Uuid`].
            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(id: Uuid) -> Self {
                Self(id)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

uuid_newtype!(
    /// Identifies a `bookings` row (the recurring/single series header).
    BookingId
);
uuid_newtype!(
    /// Identifies a `booking_occurrences` row (a materialised instance).
    OccurrenceId
);
uuid_newtype!(
    /// Identifies a `resources` row.
    ResourceId
);
uuid_newtype!(
    /// Identifies a `users` row.
    UserId
);
uuid_newtype!(
    /// Identifies an `organizations` row.
    OrganizationId
);
uuid_newtype!(
    /// Identifies a `teams` row.
    TeamId
);
uuid_newtype!(
    /// Identifies a `locations` row.
    LocationId
);
uuid_newtype!(
    /// Identifies a `roles` row.
    RoleId
);
uuid_newtype!(
    /// Identifies a `floor_zones` row.
    FloorZoneId
);
