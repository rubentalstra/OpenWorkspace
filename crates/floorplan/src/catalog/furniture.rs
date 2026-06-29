//! Furniture components: non-interactive surfaces.

use leptos::prelude::*;

use crate::catalog::geometry::anchor;
use crate::catalog::registry::RenderCtx;

/// Side lengths of the desk-block surface, in scene units.
const BLOCK_W: f64 = 16.0;
const BLOCK_H: f64 = 10.0;

/// The shared desk surface a multi-seat desk is drawn on (non-bookable). Its seats
/// are separate bookable `Desk` nodes positioned over it.
pub(crate) fn desk_block(ctx: &RenderCtx<'_>) -> AnyView {
    let transform = ctx.node.transform.to_attr();
    let (cx, cy) = anchor(&ctx.node.geometry).unwrap_or((0.0, 0.0));
    view! {
        <rect
            x=cx - BLOCK_W / 2.0
            y=cy - BLOCK_H / 2.0
            width=BLOCK_W
            height=BLOCK_H
            rx="0.8"
            data-slot="floor-node"
            data-kind="desk_block"
            class="cn-floor-node cn-floor-desk-block"
            transform=transform
        />
    }
    .into_any()
}
