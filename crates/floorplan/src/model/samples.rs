//! Sample scenes for the showcase, snapshot tests and the db round-trip test —
//! pure model data, no Leptos. Demo data, not production content.

use crate::model::kind::CatalogKind;
use crate::model::scene::{Geometry, Point2, Scene, SceneNode, Style, Transform, ViewBox};

fn node(id: &str, kind: CatalogKind, geometry: Geometry, label: Option<&str>) -> SceneNode {
    SceneNode {
        id: id.into(),
        kind,
        geometry,
        transform: Transform::default(),
        style: Style::default(),
        label: label.map(str::to_owned),
    }
}

fn poly(points: &[(f64, f64)]) -> Geometry {
    Geometry::Polygon {
        points: points.iter().map(|&(x, y)| Point2 { x, y }).collect(),
    }
}

/// A small office floor: a room outline, a team zone, three desks (a bookable
/// resource each), a door and a label. Stable shape — snapshot tests rely on it.
#[must_use]
pub fn office() -> Scene {
    Scene {
        schema_version: crate::model::migrate::CURRENT_SCENE_VERSION,
        view_box: ViewBox {
            min_x: 0.0,
            min_y: 0.0,
            width: 100.0,
            height: 60.0,
        },
        nodes: vec![
            node(
                "room-1",
                CatalogKind::RoomEnclosure,
                poly(&[(0.0, 0.0), (100.0, 0.0), (100.0, 60.0), (0.0, 60.0)]),
                None,
            ),
            node(
                "zone-eng",
                CatalogKind::Zone,
                poly(&[(4.0, 4.0), (48.0, 4.0), (48.0, 56.0), (4.0, 56.0)]),
                Some("Engineering"),
            ),
            node(
                "door-1",
                CatalogKind::Door,
                Geometry::Line {
                    points: vec![Point2 { x: 0.0, y: 26.0 }, Point2 { x: 0.0, y: 34.0 }],
                },
                None,
            ),
            node(
                "desk-1",
                CatalogKind::Desk,
                Geometry::Point { x: 14.0, y: 14.0 },
                Some("Desk A1"),
            ),
            node(
                "desk-2",
                CatalogKind::Desk,
                Geometry::Point { x: 14.0, y: 34.0 },
                Some("Desk A2"),
            ),
            node(
                "desk-3",
                CatalogKind::Desk,
                Geometry::Point { x: 34.0, y: 14.0 },
                Some("Desk A3"),
            ),
            node(
                "label-eng",
                CatalogKind::Label,
                Geometry::Point { x: 26.0, y: 2.0 },
                Some("Engineering"),
            ),
        ],
    }
}
