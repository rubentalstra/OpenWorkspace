//! Structure & shell components: non-interactive shell geometry (walls, doors,
//! windows, columns, room enclosures). Styled by the `cn-floor-*` theming layer.

use leptos::prelude::*;

use crate::catalog::geometry::{anchor, points_attr};
use crate::catalog::registry::RenderCtx;
use crate::model::scene::Geometry;

/// `points` for a line/polygon geometry, empty for anything else.
fn line_points(geometry: &Geometry) -> String {
    match geometry {
        Geometry::Line { points } | Geometry::Polygon { points } => points_attr(points),
        Geometry::Point { .. } | Geometry::Path { .. } => String::new(),
    }
}

/// A wall run (open polyline).
pub(crate) fn wall(ctx: RenderCtx<'_>) -> AnyView {
    let transform = ctx.node.transform.to_attr();
    let points = line_points(&ctx.node.geometry);
    view! {
        <polyline
            points=points
            data-slot="floor-node"
            data-kind="wall"
            class="cn-floor-node cn-floor-wall"
            transform=transform
        />
    }
    .into_any()
}

/// A door opening (polyline; the swing arc lands with P20 polish).
pub(crate) fn door(ctx: RenderCtx<'_>) -> AnyView {
    let transform = ctx.node.transform.to_attr();
    let points = line_points(&ctx.node.geometry);
    view! {
        <polyline
            points=points
            data-slot="floor-node"
            data-kind="door"
            class="cn-floor-node cn-floor-door"
            transform=transform
        />
    }
    .into_any()
}

/// A window run (polyline).
pub(crate) fn window(ctx: RenderCtx<'_>) -> AnyView {
    let transform = ctx.node.transform.to_attr();
    let points = line_points(&ctx.node.geometry);
    view! {
        <polyline
            points=points
            data-slot="floor-node"
            data-kind="window"
            class="cn-floor-node cn-floor-window"
            transform=transform
        />
    }
    .into_any()
}

/// A structural column (small disc at the geometry anchor).
pub(crate) fn column(ctx: RenderCtx<'_>) -> AnyView {
    let transform = ctx.node.transform.to_attr();
    let (cx, cy) = anchor(&ctx.node.geometry).unwrap_or((0.0, 0.0));
    view! {
        <circle
            cx=cx
            cy=cy
            r="1"
            data-slot="floor-node"
            data-kind="column"
            class="cn-floor-node cn-floor-column"
            transform=transform
        />
    }
    .into_any()
}

/// A room enclosure outline (closed polygon).
pub(crate) fn room_enclosure(ctx: RenderCtx<'_>) -> AnyView {
    let transform = ctx.node.transform.to_attr();
    let points = line_points(&ctx.node.geometry);
    view! {
        <polygon
            points=points
            data-slot="floor-node"
            data-kind="room"
            class="cn-floor-node cn-floor-room"
            transform=transform
        />
    }
    .into_any()
}
