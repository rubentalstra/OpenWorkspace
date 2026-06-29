//! floorplan — the floor **scene model**, the **SVG component catalog** (a growable
//! registry), and the read-only **inline-SVG renderer**.
//!
//! The [`model`] submodule is pure serde (no Leptos), so `db`/server code can
//! (de)serialize `floor_plans.scene` and run [`model::load_scene`] without pulling a
//! UI framework. The `catalog` and `render` submodules are Leptos and gated behind
//! the `ssr`/`hydrate` features.

pub mod model;
pub use model::*;

/// The six floor UI states the renderer maps to `data-state`. Re-exported from
/// `domain` so consumers get the renderer's input type from this crate's surface.
pub use domain::SpaceState;

#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub mod builder;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub mod catalog;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub mod render;

#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub use builder::FloorBuilder;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub use render::FloorPlan;
