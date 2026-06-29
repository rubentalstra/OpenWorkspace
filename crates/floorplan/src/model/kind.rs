//! The catalog vocabulary (`CatalogKind`) and the runtime node state (`NodeState`).
//!
//! `CatalogKind` is a closed serde enum with a `#[serde(other)] Unknown`
//! catch-all, so a scene written by a newer build (with a kind this build doesn't
//! know) deserializes to `Unknown` and renders as a neutral placeholder rather than
//! failing to load — forward compatibility for the long-lived `floor_plans.scene`.

use serde::{Deserialize, Serialize};

/// The kind of a placed scene component. Seeded with a representative subset; the
/// registry (`crate::catalog`) grows one entry at a time toward the full catalog.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CatalogKind {
    // Structure and shell.
    Wall,
    Door,
    Window,
    Column,
    RoomEnclosure,
    // Bookable resources.
    Desk,
    DeskBench,
    MeetingRoom,
    ParkingSpace,
    // Zoning.
    Zone,
    // Annotation.
    Label,
    // Wayfinding.
    Entrance,
    Exit,
    Amenity,
    /// A kind this build does not recognise (forward-compat fallback). Rendered as a
    /// neutral placeholder; never produced by the builder.
    #[serde(other)]
    Unknown,
}

impl CatalogKind {
    /// Whether this kind is a bookable resource (focusable, carries `data-state`).
    #[must_use]
    pub const fn bookable(self) -> bool {
        matches!(
            self,
            Self::Desk | Self::DeskBench | Self::MeetingRoom | Self::ParkingSpace
        )
    }
}

/// The runtime availability a bookable node reflects via `data-state`. Provided to
/// the renderer per node (default [`NodeState::Free`]); P13 wires real availability
/// and P15 mutates a single node over SSE.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    /// Available to book.
    #[default]
    Free,
    /// Reserved by a booking (not yet checked in).
    Booked,
    /// Occupied now (checked in).
    CheckedIn,
    /// Out of service (blackout / maintenance).
    Unavailable,
    /// Held (e.g. a permanent assignment).
    Reserved,
    /// Highlighted by the current UI selection.
    Selected,
}

impl NodeState {
    /// The `data-state` attribute value the theming layer styles on.
    #[must_use]
    pub const fn data_state(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Booked => "booked",
            Self::CheckedIn => "checked_in",
            Self::Unavailable => "unavailable",
            Self::Reserved => "reserved",
            Self::Selected => "selected",
        }
    }
}
