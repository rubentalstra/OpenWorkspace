//! The read-only inline-SVG floor renderer.
//!
//! [`FloorPlan`] renders a [`Scene`] to inline SVG via the catalog. The scene's
//! node list is fixed for the component's lifetime (structure is static); only the
//! reactive `viewBox` (pan/zoom) and per-node `data-state` change. Each bookable
//! node reads its availability through a [`Memo`], so a single-node update (P15 SSE)
//! repaints only the node whose [`SpaceState`] actually changed — not the whole
//! floor. Pan/zoom are native pointer/wheel handlers over the `viewBox`, so the
//! server- and client-rendered markup are byte-identical (handlers wire on hydrate).

use std::collections::HashMap;

use domain::SpaceState;
use leptos::prelude::*;
use leptos::svg::Svg;

use crate::catalog::render_node;
use crate::model::{Scene, SceneNodeId, ViewBox};

/// Zoom bounds as a multiple of the scene's initial `viewBox` extent (smaller =
/// more zoomed in).
const MIN_EXTENT: f64 = 0.1;
const MAX_EXTENT: f64 = 6.0;
/// Multiplicative zoom step per wheel notch.
const ZOOM_STEP: f64 = 0.9;

/// An in-progress drag-pan: the pointer's start position and the `viewBox` at the
/// moment the drag began.
#[derive(Clone, Copy)]
struct Drag {
    start_x: f64,
    start_y: f64,
    origin: ViewBox,
}

/// A read-only, pan/zoom-able SVG render of a floor scene.
#[expect(
    clippy::implicit_hasher,
    reason = "a component prop cannot be generic over the map's BuildHasher; callers pass a std HashMap"
)]
#[component]
pub fn FloorPlan(
    /// The scene to render. Consumed; the node list is fixed for the component's
    /// lifetime.
    scene: Scene,
    /// Per-node availability; nodes absent from the map are [`SpaceState::Free`].
    /// P15 mutates this to repaint individual nodes over SSE.
    #[prop(optional, into)]
    states: Signal<HashMap<SceneNodeId, SpaceState>>,
    /// Invoked when a bookable node is activated (click / Enter / Space).
    #[prop(optional, into)]
    on_select: Option<Callback<SceneNodeId>>,
) -> impl IntoView {
    let initial = scene.view_box;
    let view_box = RwSignal::new(initial);
    let svg_ref = NodeRef::<Svg>::new();
    let drag = RwSignal::new(Option::<Drag>::None);

    let rendered: Vec<AnyView> = scene
        .nodes
        .into_iter()
        .map(|node| {
            let id = node.id.clone();
            let state =
                Memo::new(move |_| states.with(|map| map.get(&id).copied().unwrap_or_default()));
            render_node(&node, state.into(), on_select)
        })
        .collect();

    let on_wheel = move |ev: web_sys::WheelEvent| {
        ev.prevent_default();
        let Some(el) = svg_ref.get_untracked() else {
            return;
        };
        let rect = el.get_bounding_client_rect();
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        let vb = view_box.get_untracked();
        let fx = (f64::from(ev.client_x()) - rect.left()) / rect.width();
        let fy = (f64::from(ev.client_y()) - rect.top()) / rect.height();
        let anchor_x = vb.min_x + fx * vb.width;
        let anchor_y = vb.min_y + fy * vb.height;
        let factor = if ev.delta_y() < 0.0 {
            ZOOM_STEP
        } else {
            1.0 / ZOOM_STEP
        };
        let width =
            (vb.width * factor).clamp(initial.width * MIN_EXTENT, initial.width * MAX_EXTENT);
        let height =
            (vb.height * factor).clamp(initial.height * MIN_EXTENT, initial.height * MAX_EXTENT);
        view_box.set(ViewBox {
            min_x: anchor_x - fx * width,
            min_y: anchor_y - fy * height,
            width,
            height,
        });
    };

    let on_pointer_down = move |ev: web_sys::PointerEvent| {
        if let Some(el) = svg_ref.get_untracked() {
            el.set_pointer_capture(ev.pointer_id()).ok();
        }
        drag.set(Some(Drag {
            start_x: f64::from(ev.client_x()),
            start_y: f64::from(ev.client_y()),
            origin: view_box.get_untracked(),
        }));
    };

    let on_pointer_move = move |ev: web_sys::PointerEvent| {
        let Some(d) = drag.get_untracked() else {
            return;
        };
        let Some(el) = svg_ref.get_untracked() else {
            return;
        };
        let rect = el.get_bounding_client_rect();
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        let dx = (f64::from(ev.client_x()) - d.start_x) / rect.width() * d.origin.width;
        let dy = (f64::from(ev.client_y()) - d.start_y) / rect.height() * d.origin.height;
        view_box.set(ViewBox {
            min_x: d.origin.min_x - dx,
            min_y: d.origin.min_y - dy,
            ..d.origin
        });
    };

    let on_pointer_up = move |_: web_sys::PointerEvent| drag.set(None);
    let on_pointer_leave = move |_: web_sys::PointerEvent| drag.set(None);

    view! {
        <svg
            node_ref=svg_ref
            viewBox=move || view_box.get().to_attr()
            preserveAspectRatio="xMidYMid meet"
            role="group"
            aria-label="Floor plan"
            data-slot="floor-plan"
            class="cn-floor"
            on:wheel=on_wheel
            on:pointerdown=on_pointer_down
            on:pointermove=on_pointer_move
            on:pointerup=on_pointer_up
            on:pointerleave=on_pointer_leave
        >
            {rendered}
        </svg>
    }
}

// Snapshot tests need `to_html`, which is `ssr`-only.
#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use std::collections::HashMap;

    use leptos::prelude::*;

    use super::FloorPlan;
    use crate::model::SceneNodeId;
    use crate::{SpaceState, samples};

    /// Renders the office sample to a static HTML string (SSR), so the snapshot
    /// captures the structure + initial `viewBox` + per-node `data-state`.
    fn render(states: HashMap<SceneNodeId, SpaceState>) -> String {
        Owner::new().with(|| {
            let scene = samples::office();
            let states = RwSignal::new(states);
            view! { <FloorPlan scene=scene states=states /> }.to_html()
        })
    }

    #[test]
    fn office_default_states_snapshot() {
        insta::assert_snapshot!("office_default_states", render(HashMap::new()));
    }

    #[test]
    fn office_mixed_states_snapshot() {
        let states = HashMap::from([
            (SceneNodeId::new("desk-1"), SpaceState::Free),
            (SceneNodeId::new("desk-2"), SpaceState::NotFree),
            (SceneNodeId::new("desk-3"), SpaceState::PartiallyFree),
        ]);
        insta::assert_snapshot!("office_mixed_states", render(states));
    }
}
