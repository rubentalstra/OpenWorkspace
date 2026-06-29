//! Shared geometry → SVG helpers used by every catalog component.

use crate::model::{Geometry, Point2};

/// `"x,y x,y …"` for an SVG `<polyline>`/`<polygon>` `points` attribute.
pub(crate) fn points_attr(points: &[Point2]) -> String {
    points
        .iter()
        .map(|p| format!("{},{}", p.x, p.y))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The anchor `(x, y)` of a geometry: a `Point` directly, else the centroid of a
/// polyline/polygon (so point-style components placed on any geometry still anchor
/// sensibly). `None` for an empty point set or a `Path` (no cheap anchor).
pub(crate) fn anchor(geometry: &Geometry) -> Option<(f64, f64)> {
    match geometry {
        Geometry::Point { x, y } => Some((*x, *y)),
        Geometry::Line { points } | Geometry::Polygon { points } => {
            if points.is_empty() {
                return None;
            }
            let n = points.len() as f64;
            let (sx, sy) = points
                .iter()
                .fold((0.0, 0.0), |(sx, sy), p| (sx + p.x, sy + p.y));
            Some((sx / n, sy / n))
        }
        Geometry::Path { .. } => None,
    }
}
