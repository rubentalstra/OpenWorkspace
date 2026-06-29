//! Annotation components: non-interactive text overlays.

use leptos::prelude::*;

use crate::catalog::geometry::anchor;
use crate::catalog::registry::RenderCtx;

/// A free text label anchored at its geometry.
pub(crate) fn label(ctx: &RenderCtx<'_>) -> AnyView {
    let transform = ctx.node.transform.to_attr();
    let (x, y) = anchor(&ctx.node.geometry).unwrap_or((0.0, 0.0));
    let text = ctx.node.label.clone().unwrap_or_default();
    view! {
        <text
            x=x
            y=y
            data-slot="floor-node"
            data-kind="label"
            class="cn-floor-node cn-floor-label"
            transform=transform
        >
            {text}
        </text>
    }
    .into_any()
}
