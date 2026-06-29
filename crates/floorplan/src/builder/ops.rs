//! Pure scene-editing operations + an undo/redo history. No Leptos — these are the
//! testable core the builder UI wires to pointer/keyboard events. Placeable point
//! nodes carry local `Point{0,0}` geometry and live position/rotation in their
//! `Transform`, so rotation pivots on the node's own centre and snapping applies to
//! the absolute position.

use crate::model::{
    CatalogKind, Geometry, Point2, Scene, SceneNode, SceneNodeId, Style, Transform,
};

/// Snaps a scalar to a grid (no-op when `grid <= 0`).
#[must_use]
pub fn snap(value: f64, grid: f64) -> f64 {
    if grid > 0.0 {
        (value / grid).round() * grid
    } else {
        value
    }
}

/// Snaps a point to the grid.
#[must_use]
pub fn snap_point(p: Point2, grid: f64) -> Point2 {
    Point2 {
        x: snap(p.x, grid),
        y: snap(p.y, grid),
    }
}

/// A fresh `{prefix}-{n}` id unique within the scene (smallest free `n >= 1`).
#[must_use]
pub fn fresh_id(scene: &Scene, prefix: &str) -> SceneNodeId {
    let mut n = 1u32;
    loop {
        let candidate = format!("{prefix}-{n}");
        if scene.nodes.iter().all(|node| node.id.as_str() != candidate) {
            return SceneNodeId::new(candidate);
        }
        n += 1;
    }
}

/// The id prefix a kind's placed nodes use (the snake-case `data-kind`, roughly).
fn prefix_for(kind: CatalogKind) -> &'static str {
    match kind {
        CatalogKind::Wall => "wall",
        CatalogKind::Door => "door",
        CatalogKind::Window => "window",
        CatalogKind::Column => "column",
        CatalogKind::RoomEnclosure => "room",
        CatalogKind::DeskBlock => "desk-block",
        CatalogKind::Desk => "seat",
        CatalogKind::MeetingRoom => "meeting-room",
        CatalogKind::ParkingSpace => "parking",
        CatalogKind::Zone => "zone",
        CatalogKind::Label => "label",
        CatalogKind::Entrance => "entrance",
        CatalogKind::Exit => "exit",
        CatalogKind::Amenity => "amenity",
        CatalogKind::Unknown => "node",
    }
}

/// Places a point-anchored node of `kind` at `at` (snapped): local `Point{0,0}`
/// geometry, position carried in `transform.translate`. Returns its id.
pub fn place_point(scene: &mut Scene, kind: CatalogKind, at: Point2, grid: f64) -> SceneNodeId {
    let id = fresh_id(scene, prefix_for(kind));
    let pos = snap_point(at, grid);
    scene.nodes.push(SceneNode {
        id: id.clone(),
        kind,
        geometry: Geometry::Point { x: 0.0, y: 0.0 },
        transform: Transform {
            translate: pos,
            ..Transform::default()
        },
        style: Style::default(),
        label: None,
    });
    id
}

/// Places a desk pod: a `DeskBlock` surface plus `seats` bookable `Desk` seats laid
/// out in two columns, centred on `at`. Returns `(block_id, seat_ids)`.
pub fn place_desk_pod(
    scene: &mut Scene,
    seats: u32,
    at: Point2,
    grid: f64,
) -> (SceneNodeId, Vec<SceneNodeId>) {
    let block = place_point(scene, CatalogKind::DeskBlock, at, grid);
    let cols = if seats <= 1 { 1 } else { 2 };
    let rows = seats.div_ceil(cols);
    let (sx, sy) = (8.0, 7.0);
    let mut seat_ids = Vec::with_capacity(seats as usize);
    for i in 0..seats {
        let col = f64::from(i % cols);
        let row = f64::from(i / cols);
        let dx = (col - f64::from(cols - 1) / 2.0) * sx;
        let dy = (row - f64::from(rows - 1) / 2.0) * sy;
        let pos = Point2 {
            x: at.x + dx,
            y: at.y + dy,
        };
        seat_ids.push(place_point(scene, CatalogKind::Desk, pos, grid));
    }
    (block, seat_ids)
}

/// Adds a drawn polyline/polygon node from absolute `points` (transform identity).
pub fn place_drawn(scene: &mut Scene, kind: CatalogKind, geometry: Geometry) -> SceneNodeId {
    let id = fresh_id(scene, prefix_for(kind));
    scene.nodes.push(SceneNode {
        id: id.clone(),
        kind,
        geometry,
        transform: Transform::default(),
        style: Style::default(),
        label: None,
    });
    id
}

fn node_mut<'a>(scene: &'a mut Scene, id: &SceneNodeId) -> Option<&'a mut SceneNode> {
    scene.nodes.iter_mut().find(|n| &n.id == id)
}

/// Moves a point-anchored node to an absolute snapped position. Returns `false` if
/// the node is missing.
pub fn move_to(scene: &mut Scene, id: &SceneNodeId, at: Point2, grid: f64) -> bool {
    match node_mut(scene, id) {
        Some(node) => {
            node.transform.translate = snap_point(at, grid);
            true
        }
        None => false,
    }
}

/// Nudges a point-anchored node by a delta (e.g. arrow keys), then snaps.
pub fn nudge(scene: &mut Scene, id: &SceneNodeId, dx: f64, dy: f64, grid: f64) -> bool {
    match node_mut(scene, id) {
        Some(node) => {
            let t = node.transform.translate;
            node.transform.translate = snap_point(
                Point2 {
                    x: t.x + dx,
                    y: t.y + dy,
                },
                grid,
            );
            true
        }
        None => false,
    }
}

/// Sets a node's rotation in degrees (snapped to `step`, e.g. 15°), normalised to
/// `[0, 360)`. Returns `false` if the node is missing.
pub fn rotate_to(scene: &mut Scene, id: &SceneNodeId, degrees: f64, step: f64) -> bool {
    match node_mut(scene, id) {
        Some(node) => {
            let snapped = snap(degrees, step).rem_euclid(360.0);
            node.transform.rotate_deg = snapped;
            true
        }
        None => false,
    }
}

/// Removes a node. Returns `false` if it was already absent.
pub fn delete(scene: &mut Scene, id: &SceneNodeId) -> bool {
    let before = scene.nodes.len();
    scene.nodes.retain(|n| &n.id != id);
    scene.nodes.len() != before
}

/// An undo/redo history of whole-scene snapshots (bounded).
#[derive(Debug)]
pub struct History {
    past: Vec<Scene>,
    future: Vec<Scene>,
    limit: usize,
}

impl History {
    /// A history bounded to `limit` undo steps.
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            limit: limit.max(1),
        }
    }

    /// Records the scene state *before* an edit (call prior to mutating). Clears the
    /// redo stack and drops the oldest entry past the limit.
    pub fn record(&mut self, snapshot: Scene) {
        self.future.clear();
        self.past.push(snapshot);
        if self.past.len() > self.limit {
            self.past.remove(0);
        }
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// Restores the previous snapshot, pushing `current` onto the redo stack.
    pub fn undo(&mut self, current: Scene) -> Option<Scene> {
        let prev = self.past.pop()?;
        self.future.push(current);
        Some(prev)
    }

    /// Re-applies the last undone snapshot, pushing `current` onto the undo stack.
    pub fn redo(&mut self, current: Scene) -> Option<Scene> {
        let next = self.future.pop()?;
        self.past.push(current);
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> Scene {
        Scene::default()
    }

    #[track_caller]
    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} !~= {b}");
    }

    #[test]
    fn snap_rounds_to_grid_and_passes_through_when_off() {
        approx(snap(13.0, 5.0), 15.0);
        approx(snap(12.0, 5.0), 10.0);
        approx(snap(13.3, 0.0), 13.3);
    }

    #[test]
    fn fresh_id_is_unique_and_sequential() {
        let mut scene = empty();
        let a = place_point(
            &mut scene,
            CatalogKind::Desk,
            Point2 { x: 0.0, y: 0.0 },
            0.0,
        );
        let b = place_point(
            &mut scene,
            CatalogKind::Desk,
            Point2 { x: 0.0, y: 0.0 },
            0.0,
        );
        assert_eq!(a.as_str(), "seat-1");
        assert_eq!(b.as_str(), "seat-2");
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn place_point_snaps_position_into_transform() {
        let mut scene = empty();
        let id = place_point(
            &mut scene,
            CatalogKind::Desk,
            Point2 { x: 13.0, y: 8.0 },
            5.0,
        );
        let node = scene.nodes.iter().find(|n| n.id == id).unwrap();
        assert_eq!(node.geometry, Geometry::Point { x: 0.0, y: 0.0 });
        assert_eq!(node.transform.translate, Point2 { x: 15.0, y: 10.0 });
    }

    #[test]
    fn desk_pod_makes_one_block_plus_n_bookable_seats() {
        let mut scene = empty();
        let (block, seats) = place_desk_pod(&mut scene, 4, Point2 { x: 50.0, y: 30.0 }, 0.0);
        assert_eq!(seats.len(), 4);
        // 1 block + 4 seats.
        assert_eq!(scene.nodes.len(), 5);
        let block_node = scene.nodes.iter().find(|n| n.id == block).unwrap();
        assert_eq!(block_node.kind, CatalogKind::DeskBlock);
        assert!(!block_node.kind.bookable());
        for seat in &seats {
            let s = scene.nodes.iter().find(|n| &n.id == seat).unwrap();
            assert_eq!(s.kind, CatalogKind::Desk);
            assert!(s.kind.bookable());
        }
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn move_nudge_rotate_delete() {
        let mut scene = empty();
        let id = place_point(
            &mut scene,
            CatalogKind::Desk,
            Point2 { x: 0.0, y: 0.0 },
            0.0,
        );

        assert!(move_to(&mut scene, &id, Point2 { x: 22.0, y: 8.0 }, 5.0));
        let t = scene.nodes[0].transform;
        assert_eq!(t.translate, Point2 { x: 20.0, y: 10.0 });

        assert!(nudge(&mut scene, &id, 5.0, 0.0, 5.0));
        approx(scene.nodes[0].transform.translate.x, 25.0);

        assert!(rotate_to(&mut scene, &id, 100.0, 15.0));
        approx(scene.nodes[0].transform.rotate_deg, 105.0);
        // Wraps into [0,360).
        assert!(rotate_to(&mut scene, &id, -30.0, 15.0));
        approx(scene.nodes[0].transform.rotate_deg, 330.0);

        assert!(delete(&mut scene, &id));
        assert!(scene.nodes.is_empty());
        assert!(!delete(&mut scene, &id));
    }

    #[test]
    fn history_undo_redo_round_trips() {
        let mut scene = empty();
        let mut hist = History::new(10);
        assert!(!hist.can_undo());

        hist.record(scene.clone());
        place_point(
            &mut scene,
            CatalogKind::Desk,
            Point2 { x: 0.0, y: 0.0 },
            0.0,
        );
        assert_eq!(scene.nodes.len(), 1);

        // Undo → back to empty; redo → desk again.
        scene = hist.undo(scene).unwrap();
        assert!(scene.nodes.is_empty());
        assert!(hist.can_redo());
        scene = hist.redo(scene).unwrap();
        assert_eq!(scene.nodes.len(), 1);
    }

    #[test]
    fn history_record_clears_redo_and_bounds() {
        let mut scene = empty();
        let mut hist = History::new(2);
        hist.record(scene.clone());
        place_point(
            &mut scene,
            CatalogKind::Desk,
            Point2 { x: 0.0, y: 0.0 },
            0.0,
        );
        scene = hist.undo(scene).unwrap();
        assert!(hist.can_redo());
        // A new edit clears the redo stack.
        hist.record(scene.clone());
        assert!(!hist.can_redo());
    }
}
