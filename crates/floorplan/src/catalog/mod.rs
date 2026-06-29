//! The SVG component catalog: a growable registry mapping each [`CatalogKind`] to
//! an inline-SVG render fn plus palette metadata.
//!
//! The single extension point is [`registry`] — see its docs. Per-category modules
//! hold the render fns; the registry table wires them to kinds and metadata.
//!
//! [`CatalogKind`]: crate::model::CatalogKind

mod annotation;
mod bookable;
mod geometry;
mod registry;
mod structure;
mod wayfinding;
mod zoning;

pub use registry::{
    CatalogEntry, CatalogMeta, Category, RenderCtx, by_category, entries, lookup, render_node,
};
