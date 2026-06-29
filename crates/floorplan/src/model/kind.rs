//! The catalog vocabulary (`CatalogKind`) — the set of component kinds a scene node
//! may be.
//!
//! `CatalogKind` is a closed serde enum with a `#[serde(other)] Unknown`
//! catch-all, so a scene written by a newer build (with a kind this build doesn't
//! know) deserializes to `Unknown` and renders as a neutral placeholder rather than
//! failing to load — forward compatibility for the long-lived `floor_plans.scene`.
//!
//! The runtime *availability* a bookable node shows (`data-state`) is not a scene
//! concept and lives in the domain as [`domain::SpaceState`]; the renderer maps it.

use serde::{Deserialize, Serialize};

/// The kind of a placed scene component. Seeded with a representative subset; the
/// registry (`crate::catalog`) grows one entry at a time toward the full catalog.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Hash, strum::EnumIter)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_kind_falls_back_not_errors() {
        let kind: CatalogKind = serde_json::from_str("\"hologram_emitter\"").unwrap();
        assert_eq!(kind, CatalogKind::Unknown);
    }

    #[test]
    fn known_kind_round_trips() {
        let kind: CatalogKind = serde_json::from_str("\"meeting_room\"").unwrap();
        assert_eq!(kind, CatalogKind::MeetingRoom);
        assert_eq!(serde_json::to_string(&kind).unwrap(), "\"meeting_room\"");
    }

    #[test]
    fn bookable_classification() {
        assert!(CatalogKind::Desk.bookable());
        assert!(CatalogKind::MeetingRoom.bookable());
        assert!(!CatalogKind::Wall.bookable());
        assert!(!CatalogKind::Zone.bookable());
    }
}
