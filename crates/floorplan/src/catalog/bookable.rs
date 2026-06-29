//! Bookable resources: focusable, stateful nodes. Each renders a
//! `role="button"` group with a reactive `data-state` (the theming layer styles
//! availability) and click / Enter / Space selection via `on_select`.

use domain::SpaceState;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos_icons::Icon;

use crate::catalog::geometry::{anchor, points_attr};
use crate::catalog::registry::RenderCtx;
use crate::model::Geometry;

/// Side of the square state-icon box, in scene units.
const SYMBOL_SIZE: f64 = 3.0;

/// The Lucide icon for a state, so availability is conveyed by symbol — not by
/// colour alone (WCAG 1.4.1). Drawn `aria-hidden` (the accessible name is the node
/// label; the legend carries the text labels). `CannotBeBooked` uses a distinct
/// "ban" glyph rather than another cross, so it is not colour-only against
/// `NotFree`.
fn state_icon(state: SpaceState) -> icondata::Icon {
    match state {
        SpaceState::Free => icondata::LuPlus,
        SpaceState::PartiallyFree => icondata::LuMinus,
        SpaceState::NotFree => icondata::LuX,
        SpaceState::TemporarilyBlocked => icondata::LuClock,
        SpaceState::PermanentUser => icondata::LuUser,
        SpaceState::CannotBeBooked => icondata::LuBan,
    }
}

/// Wraps a component `shape` in a focusable, selectable, stateful `<g>` and overlays
/// a non-colour state glyph at the geometry anchor.
fn bookable_group(
    ctx: &RenderCtx<'_>,
    kind: &'static str,
    class: &'static str,
    default_label: &'static str,
    shape: AnyView,
) -> AnyView {
    let node = ctx.node;
    let state = ctx.state;
    let on_select = ctx.on_select;
    let aria = node
        .label
        .clone()
        .unwrap_or_else(|| default_label.to_owned());
    let transform = node.transform.to_attr();
    let (gx, gy) = anchor(&node.geometry).unwrap_or((0.0, 0.0));
    let id_click = node.id.clone();
    let id_key = node.id.clone();

    let on_click = move |_| {
        if let Some(cb) = on_select {
            cb.run(id_click.clone());
        }
    };
    let on_keydown = move |ev: KeyboardEvent| {
        let key = ev.key();
        if key == "Enter" || key == " " {
            ev.prevent_default();
            if let Some(cb) = on_select {
                cb.run(id_key.clone());
            }
        }
    };

    view! {
        <g
            role="button"
            tabindex="0"
            aria-label=aria
            data-slot="floor-node"
            data-kind=kind
            data-state=move || state.get().as_str()
            class=format!("cn-floor-node cn-floor-bookable {class}")
            transform=transform
            on:click=on_click
            on:keydown=on_keydown
        >
            {shape}
            <g
                transform=format!(
                    "translate({} {})",
                    gx - SYMBOL_SIZE / 2.0,
                    gy - SYMBOL_SIZE / 2.0,
                )
                aria-hidden="true"
                class="cn-floor-state-symbol"
            >
                {move || {
                    view! {
                        <Icon
                            icon=state_icon(state.get())
                            width=SYMBOL_SIZE.to_string()
                            height=SYMBOL_SIZE.to_string()
                        />
                    }
                }}
            </g>
        </g>
    }
    .into_any()
}

/// A rectangular footprint centred on the geometry anchor.
fn footprint(ctx: &RenderCtx<'_>, w: f64, h: f64) -> AnyView {
    let (cx, cy) = anchor(&ctx.node.geometry).unwrap_or((0.0, 0.0));
    view! { <rect x=cx - w / 2.0 y=cy - h / 2.0 width=w height=h rx="0.5" /> }.into_any()
}

/// A single desk.
pub(crate) fn desk(ctx: &RenderCtx<'_>) -> AnyView {
    let shape = footprint(ctx, 8.0, 6.0);
    bookable_group(ctx, "desk", "cn-floor-desk", "Desk", shape)
}

/// A bench desk (wider shared footprint).
pub(crate) fn desk_bench(ctx: &RenderCtx<'_>) -> AnyView {
    let shape = footprint(ctx, 16.0, 6.0);
    bookable_group(
        ctx,
        "desk_bench",
        "cn-floor-desk-bench",
        "Bench desk",
        shape,
    )
}

/// A parking space.
pub(crate) fn parking_space(ctx: &RenderCtx<'_>) -> AnyView {
    let shape = footprint(ctx, 5.0, 10.0);
    bookable_group(
        ctx,
        "parking_space",
        "cn-floor-parking-space",
        "Parking space",
        shape,
    )
}

/// A meeting room (bookable polygon enclosure).
pub(crate) fn meeting_room(ctx: &RenderCtx<'_>) -> AnyView {
    let points = match &ctx.node.geometry {
        Geometry::Polygon { points } | Geometry::Line { points } => points_attr(points),
        Geometry::Point { .. } | Geometry::Path { .. } => String::new(),
    };
    let shape = view! { <polygon points=points /> }.into_any();
    bookable_group(
        ctx,
        "meeting_room",
        "cn-floor-meeting-room",
        "Meeting room",
        shape,
    )
}
