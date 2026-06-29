//! Wayfinding components: non-interactive markers (entrances, exits, amenities).
//! Each carries an accessible name via an SVG `<title>`.

use leptos::prelude::*;

use crate::catalog::geometry::anchor;
use crate::catalog::registry::RenderCtx;

/// A marker disc at the geometry anchor with an accessible title.
fn marker(
    ctx: RenderCtx<'_>,
    kind: &'static str,
    class: &'static str,
    default_label: &'static str,
) -> AnyView {
    let transform = ctx.node.transform.to_attr();
    let (cx, cy) = anchor(&ctx.node.geometry).unwrap_or((0.0, 0.0));
    let title = ctx
        .node
        .label
        .clone()
        .unwrap_or_else(|| default_label.to_owned());
    view! {
        <g
            data-slot="floor-node"
            data-kind=kind
            class=format!("cn-floor-node {class}")
            transform=transform
        >
            <title>{title}</title>
            <circle cx=cx cy=cy r="2" />
        </g>
    }
    .into_any()
}

/// An entrance marker.
pub(crate) fn entrance(ctx: RenderCtx<'_>) -> AnyView {
    marker(ctx, "entrance", "cn-floor-entrance", "Entrance")
}

/// An exit marker.
pub(crate) fn exit(ctx: RenderCtx<'_>) -> AnyView {
    marker(ctx, "exit", "cn-floor-exit", "Exit")
}

/// An amenity marker (e.g. kitchen, restroom).
pub(crate) fn amenity(ctx: RenderCtx<'_>) -> AnyView {
    marker(ctx, "amenity", "cn-floor-amenity", "Amenity")
}
