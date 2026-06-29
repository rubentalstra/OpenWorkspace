//! Zoning components: non-interactive area overlays (departments, neighbourhoods).

use leptos::prelude::*;

use crate::catalog::geometry::{anchor, points_attr};
use crate::catalog::registry::RenderCtx;
use crate::model::scene::Geometry;

/// A zone: a filled polygon with an optional centred label.
pub(crate) fn zone(ctx: RenderCtx<'_>) -> AnyView {
    let transform = ctx.node.transform.to_attr();
    let points = match &ctx.node.geometry {
        Geometry::Polygon { points } | Geometry::Line { points } => points_attr(points),
        Geometry::Point { .. } | Geometry::Path { .. } => String::new(),
    };
    let (lx, ly) = anchor(&ctx.node.geometry).unwrap_or((0.0, 0.0));
    let label = ctx.node.label.clone();
    view! {
        <g
            data-slot="floor-node"
            data-kind="zone"
            class="cn-floor-node cn-floor-zone"
            transform=transform
        >
            <polygon points=points class="cn-floor-zone-area" />
            {label
                .map(|text| {
                    view! {
                        <text x=lx y=ly class="cn-floor-zone-label">
                            {text}
                        </text>
                    }
                })}
        </g>
    }
    .into_any()
}
