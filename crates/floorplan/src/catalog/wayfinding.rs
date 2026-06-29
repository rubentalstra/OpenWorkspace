//! Wayfinding components: non-interactive markers (entrances, exits, amenities).
//! Each renders its Lucide icon and carries an accessible name via an SVG `<title>`.

use leptos::prelude::*;
use leptos_icons::Icon;

use crate::catalog::geometry::anchor;
use crate::catalog::registry::RenderCtx;

/// Side of the square marker-icon box, in scene units.
const MARKER_SIZE: f64 = 4.0;

/// A Lucide marker icon at the geometry anchor with an accessible title.
fn marker(
    ctx: &RenderCtx<'_>,
    kind: &'static str,
    class: &'static str,
    default_label: &'static str,
    icon: icondata::Icon,
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
            <g transform=format!(
                "translate({} {})",
                cx - MARKER_SIZE / 2.0,
                cy - MARKER_SIZE / 2.0,
            )>
                <Icon
                    icon=icon
                    width=MARKER_SIZE.to_string()
                    height=MARKER_SIZE.to_string()
                    attr:aria-hidden="true"
                />
            </g>
        </g>
    }
    .into_any()
}

/// An entrance marker.
pub(crate) fn entrance(ctx: &RenderCtx<'_>) -> AnyView {
    marker(
        ctx,
        "entrance",
        "cn-floor-entrance",
        "Entrance",
        icondata::LuLogIn,
    )
}

/// An exit marker.
pub(crate) fn exit(ctx: &RenderCtx<'_>) -> AnyView {
    marker(ctx, "exit", "cn-floor-exit", "Exit", icondata::LuLogOut)
}

/// An amenity marker (e.g. kitchen, restroom).
pub(crate) fn amenity(ctx: &RenderCtx<'_>) -> AnyView {
    marker(
        ctx,
        "amenity",
        "cn-floor-amenity",
        "Amenity",
        icondata::LuMapPin,
    )
}
