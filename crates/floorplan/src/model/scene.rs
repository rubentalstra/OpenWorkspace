//! The scene document: the structured, serde-able representation of a floor that
//! is stored in `floor_plans.scene` (jsonb) and rendered to inline SVG.

use serde::{Deserialize, Serialize};

use crate::model::error::SceneError;
use crate::model::kind::CatalogKind;
use crate::model::migrate::CURRENT_SCENE_VERSION;

/// A 2D point in scene coordinates.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

/// The SVG `viewBox` of a scene (`min_x min_y width height`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct ViewBox {
    pub min_x: f64,
    pub min_y: f64,
    pub width: f64,
    pub height: f64,
}

impl ViewBox {
    /// Renders the SVG `viewBox` attribute string.
    #[must_use]
    pub fn to_attr(&self) -> String {
        format!(
            "{} {} {} {}",
            self.min_x, self.min_y, self.width, self.height
        )
    }

    fn is_valid(&self) -> bool {
        self.min_x.is_finite()
            && self.min_y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }
}

impl Default for ViewBox {
    fn default() -> Self {
        Self {
            min_x: 0.0,
            min_y: 0.0,
            width: 100.0,
            height: 100.0,
        }
    }
}

/// A stable identifier for a placed node — the join key to `resource_positions`
/// and `floor_zones.scene_node_id`. Never reassigned once placed.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SceneNodeId(String);

impl SceneNodeId {
    /// Wraps a raw id.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SceneNodeId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// A node's geometry. Internally tagged (`{"type":"polygon", …}`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Geometry {
    /// A single anchor point.
    Point { x: f64, y: f64 },
    /// An open polyline (≥ 2 points): walls, paths.
    Line { points: Vec<Point2> },
    /// A closed polygon (≥ 3 points): rooms, zones.
    Polygon { points: Vec<Point2> },
    /// A raw SVG path data string.
    Path { d: String },
}

impl Geometry {
    fn is_valid(&self) -> bool {
        let finite = |p: &Point2| p.x.is_finite() && p.y.is_finite();
        match self {
            Self::Point { x, y } => x.is_finite() && y.is_finite(),
            Self::Line { points } => points.len() >= 2 && points.iter().all(finite),
            Self::Polygon { points } => points.len() >= 3 && points.iter().all(finite),
            Self::Path { d } => !d.trim().is_empty(),
        }
    }
}

/// An affine placement applied to a node's geometry. Defaults to identity.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translate: Point2,
    pub rotate_deg: f64,
    pub scale: Point2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate: Point2 { x: 0.0, y: 0.0 },
            rotate_deg: 0.0,
            scale: Point2 { x: 1.0, y: 1.0 },
        }
    }
}

impl Transform {
    /// The SVG `transform` attribute string, or `None` for identity.
    #[must_use]
    pub fn to_attr(&self) -> Option<String> {
        let identity = self.translate.x == 0.0
            && self.translate.y == 0.0
            && self.rotate_deg == 0.0
            && self.scale.x == 1.0
            && self.scale.y == 1.0;
        if identity {
            return None;
        }
        Some(format!(
            "translate({} {}) rotate({}) scale({} {})",
            self.translate.x, self.translate.y, self.rotate_deg, self.scale.x, self.scale.y
        ))
    }
}

/// Optional per-node style overrides. Default styling is the catalog's themeable
/// `cn-floor-*` classes; these override for one-off cases.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Style {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
}

/// A placed component instance.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SceneNode {
    pub id: SceneNodeId,
    pub kind: CatalogKind,
    pub geometry: Geometry,
    #[serde(default)]
    pub transform: Transform,
    #[serde(default)]
    pub style: Style,
    /// Human-readable text: the rendered text for a `Label`, and the accessible
    /// name (`aria-label`) for a bookable node (e.g. "Desk A12").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

fn default_schema_version() -> u32 {
    CURRENT_SCENE_VERSION
}

/// A floor's scene: a versioned document of placed nodes. Fully defaultable so the
/// schema's empty `'{}'::jsonb` default loads as an empty canvas.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Scene {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub view_box: ViewBox,
    #[serde(default)]
    pub nodes: Vec<SceneNode>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCENE_VERSION,
            view_box: ViewBox::default(),
            nodes: Vec::new(),
        }
    }
}

impl Scene {
    /// Validates structural invariants: a sane view box, unique node ids, and
    /// well-formed geometry. The load path calls this so corruption surfaces early
    /// (the renderer itself is fail-soft per node).
    ///
    /// # Errors
    ///
    /// [`SceneError::InvalidViewBox`], [`SceneError::DuplicateNodeId`] or
    /// [`SceneError::InvalidGeometry`].
    pub fn validate(&self) -> Result<(), SceneError> {
        if !self.view_box.is_valid() {
            return Err(SceneError::InvalidViewBox);
        }
        let mut seen = std::collections::HashSet::with_capacity(self.nodes.len());
        for node in &self.nodes {
            if !seen.insert(node.id.as_str()) {
                return Err(SceneError::DuplicateNodeId(node.id.as_str().to_owned()));
            }
            if !node.geometry.is_valid() {
                return Err(SceneError::InvalidGeometry(node.id.as_str().to_owned()));
            }
        }
        Ok(())
    }
}
