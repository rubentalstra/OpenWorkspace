//! The scene model — pure serde, no Leptos. The serde-able representation of a
//! floor stored in `floor_plans.scene`, plus schema versioning/migration and
//! validation.

mod error;
mod kind;
mod migrate;
pub mod samples;
mod scene;

pub use error::SceneError;
pub use kind::CatalogKind;
pub use migrate::{CURRENT_SCENE_VERSION, load_scene};
pub use scene::{Geometry, Point2, Scene, SceneNode, SceneNodeId, Style, Transform, ViewBox};
