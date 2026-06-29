//! The `FloorBuilder` scene-canvas editor: a pan/zoom SVG canvas with a component
//! palette, click-to-place, click-to-select, drag-to-move, delete, and undo/redo.
//!
//! It edits a shared `RwSignal<Scene>` and reports the selected node via a shared
//! `RwSignal<Option<SceneNodeId>>`, so the app can render resource/equipment panels
//! beside it and persist the result — keeping this crate free of `db`/`auth`.
//! Resource binding, zones and multi-click wall/zone drawing are layered on later.

use domain::SpaceState;
use leptos::prelude::*;
use leptos::svg::Svg;
use ui::{Button, ButtonSize, ButtonVariant};

use super::ops::{self, History};
use crate::catalog::render_node;
use crate::catalog::{Category, by_category};
use crate::model::{CatalogKind, Point2, Scene, SceneNodeId, ViewBox};

/// Default snap grid (scene units).
const GRID: f64 = 4.0;

/// The active builder tool.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Tool {
    /// Select / move existing nodes.
    #[default]
    Select,
    /// Click the canvas to place a single node of this kind.
    Place(CatalogKind),
    /// Click the canvas to drop a desk pod of N bookable seats.
    PlacePod(u32),
}

/// An in-progress pointer gesture.
#[derive(Clone)]
enum Drag {
    /// Panning the canvas: pointer start + the viewBox at gesture start.
    Pan {
        start_x: f64,
        start_y: f64,
        origin: ViewBox,
    },
    /// Moving a specific node under the pointer.
    Node { id: SceneNodeId },
}

/// Palette groups, in display order.
const PALETTE_GROUPS: &[(Category, &str)] = &[
    (Category::Structure, "Structure"),
    (Category::Furniture, "Furniture"),
    (Category::Bookable, "Bookable"),
    (Category::Zoning, "Zoning"),
    (Category::Wayfinding, "Wayfinding"),
    (Category::Annotation, "Annotation"),
];

/// The interactive floor-builder canvas.
#[component]
#[expect(
    clippy::too_many_lines,
    reason = "the canvas wires many pointer/keyboard handlers and the palette in one place"
)]
pub fn FloorBuilder(
    /// The editable scene (shared with the app for persistence).
    scene: RwSignal<Scene>,
    /// The currently selected node (shared so the app shows its resource panel).
    #[prop(into)]
    selected: RwSignal<Option<SceneNodeId>>,
    /// An optional semi-transparent reference underlay (a presigned image URL).
    #[prop(optional, into)]
    reference_href: Signal<Option<String>>,
) -> impl IntoView {
    let tool = RwSignal::new(Tool::Select);
    let view_box = RwSignal::new(scene.with_untracked(|s| s.view_box));
    let svg_ref = NodeRef::<Svg>::new();
    let drag = RwSignal::new(Option::<Drag>::None);
    let history = RwSignal::new(History::new(64));
    // Builder nodes render in their neutral availability state.
    let free = Signal::derive(|| SpaceState::Free);

    let to_svg = move |client_x: f64, client_y: f64| -> Option<Point2> {
        let el = svg_ref.get_untracked()?;
        let rect = el.get_bounding_client_rect();
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return None;
        }
        let vb = view_box.get_untracked();
        let fx = (client_x - rect.left()) / rect.width();
        let fy = (client_y - rect.top()) / rect.height();
        Some(Point2 {
            x: vb.min_x + fx * vb.width,
            y: vb.min_y + fy * vb.height,
        })
    };

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
        let factor = if ev.delta_y() < 0.0 { 0.9 } else { 1.0 / 0.9 };
        let width = (vb.width * factor).clamp(vb.width * 0.02, vb.width * 50.0);
        let height = (vb.height * factor).clamp(vb.height * 0.02, vb.height * 50.0);
        view_box.set(ViewBox {
            min_x: anchor_x - fx * width,
            min_y: anchor_y - fy * height,
            width,
            height,
        });
    };

    let on_canvas_down = move |ev: web_sys::PointerEvent| {
        let Some(at) = to_svg(f64::from(ev.client_x()), f64::from(ev.client_y())) else {
            return;
        };
        match tool.get_untracked() {
            Tool::Select => {
                selected.set(None);
                if let Some(el) = svg_ref.get_untracked() {
                    el.set_pointer_capture(ev.pointer_id()).ok();
                }
                drag.set(Some(Drag::Pan {
                    start_x: f64::from(ev.client_x()),
                    start_y: f64::from(ev.client_y()),
                    origin: view_box.get_untracked(),
                }));
            }
            Tool::Place(kind) => {
                history.update(|h| h.record(scene.get_untracked()));
                let mut placed = None;
                scene.update(|s| placed = Some(ops::place_point(s, kind, at, GRID)));
                selected.set(placed);
            }
            Tool::PlacePod(seats) => {
                history.update(|h| h.record(scene.get_untracked()));
                let mut first = None;
                scene.update(|s| {
                    let (_, ids) = ops::place_desk_pod(s, seats, at, GRID);
                    first = ids.into_iter().next();
                });
                selected.set(first);
            }
        }
    };

    let on_pointer_move = move |ev: web_sys::PointerEvent| {
        let Some(state) = drag.get_untracked() else {
            return;
        };
        match state {
            Drag::Pan {
                start_x,
                start_y,
                origin,
            } => {
                let Some(el) = svg_ref.get_untracked() else {
                    return;
                };
                let rect = el.get_bounding_client_rect();
                if rect.width() <= 0.0 || rect.height() <= 0.0 {
                    return;
                }
                let dx = (f64::from(ev.client_x()) - start_x) / rect.width() * origin.width;
                let dy = (f64::from(ev.client_y()) - start_y) / rect.height() * origin.height;
                view_box.set(ViewBox {
                    min_x: origin.min_x - dx,
                    min_y: origin.min_y - dy,
                    ..origin
                });
            }
            Drag::Node { id } => {
                if let Some(at) = to_svg(f64::from(ev.client_x()), f64::from(ev.client_y())) {
                    scene.update(|s| {
                        ops::move_to(s, &id, at, GRID);
                    });
                }
            }
        }
    };

    let on_pointer_up = move |_: web_sys::PointerEvent| drag.set(None);

    let on_key = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        if key == "Escape" {
            tool.set(Tool::Select);
        } else if (key == "Delete" || key == "Backspace")
            && let Some(id) = selected.get_untracked()
        {
            ev.prevent_default();
            history.update(|h| h.record(scene.get_untracked()));
            scene.update(|s| {
                ops::delete(s, &id);
            });
            selected.set(None);
        }
    };

    let undo = move |_: web_sys::MouseEvent| {
        if let Some(prev) = history
            .try_update(|h| h.undo(scene.get_untracked()))
            .flatten()
        {
            scene.set(prev);
            selected.set(None);
        }
    };
    let redo = move |_: web_sys::MouseEvent| {
        if let Some(next) = history
            .try_update(|h| h.redo(scene.get_untracked()))
            .flatten()
        {
            scene.set(next);
            selected.set(None);
        }
    };

    let nodes_view = move || {
        scene
            .get()
            .nodes
            .into_iter()
            .map(|node| {
                let id_selected = node.id.clone();
                let id_down = node.id.clone();
                let id_click = node.id.clone();
                let is_selected = Memo::new(move |_| selected.get().as_ref() == Some(&id_selected));
                let on_node_down = move |ev: web_sys::PointerEvent| {
                    ev.stop_propagation();
                    if tool.get_untracked() == Tool::Select {
                        selected.set(Some(id_down.clone()));
                        drag.set(Some(Drag::Node {
                            id: id_down.clone(),
                        }));
                    }
                };
                let on_node_click = move |ev: web_sys::MouseEvent| {
                    ev.stop_propagation();
                    selected.set(Some(id_click.clone()));
                };
                let body = render_node(&node, free, None);
                view! {
                    <g
                        class="cn-floor-builder-node"
                        data-selected=move || if is_selected.get() { "true" } else { "false" }
                        on:pointerdown=on_node_down
                        on:click=on_node_click
                    >
                        {body}
                    </g>
                }
            })
            .collect_view()
    };

    view! {
        <div class="cn-floor-builder" data-slot="floor-builder">
            <BuilderToolbar tool=tool on_undo=undo on_redo=redo />
            <div class="cn-floor-builder-canvas">
                <svg
                    node_ref=svg_ref
                    viewBox=move || view_box.get().to_attr()
                    preserveAspectRatio="xMidYMid meet"
                    tabindex="0"
                    role="application"
                    aria-label="Floor builder canvas"
                    class="cn-floor cn-floor-builder-svg"
                    on:wheel=on_wheel
                    on:pointerdown=on_canvas_down
                    on:pointermove=on_pointer_move
                    on:pointerup=on_pointer_up
                    on:pointerleave=on_pointer_up
                    on:keydown=on_key
                >
                    {move || {
                        reference_href
                            .get()
                            .map(|href| {
                                let vb = view_box.get_untracked();
                                view! {
                                    <image
                                        href=href
                                        x=vb.min_x
                                        y=vb.min_y
                                        width=vb.width
                                        height=vb.height
                                        class="cn-floor-builder-underlay"
                                        preserveAspectRatio="none"
                                    />
                                }
                            })
                    }}
                    {nodes_view}
                </svg>
            </div>
        </div>
    }
}

/// The palette + tool/undo toolbar. Tools are `ui::Button`s; the active tool shows
/// the filled (Default) variant, the rest Outline.
#[component]
fn BuilderToolbar(
    tool: RwSignal<Tool>,
    on_undo: impl Fn(web_sys::MouseEvent) + 'static,
    on_redo: impl Fn(web_sys::MouseEvent) + 'static,
) -> impl IntoView {
    let variant_for = move |t: Tool| {
        Signal::derive(move || {
            if tool.get() == t {
                ButtonVariant::Default
            } else {
                ButtonVariant::Outline
            }
        })
    };
    view! {
        <div class="cn-floor-builder-toolbar" role="toolbar" aria-label="Builder tools">
            <Button
                size=ButtonSize::Sm
                variant=variant_for(Tool::Select)
                on:click=move |_| tool.set(Tool::Select)
            >
                "Select"
            </Button>
            <Button
                size=ButtonSize::Sm
                variant=variant_for(Tool::PlacePod(4))
                on:click=move |_| tool.set(Tool::PlacePod(4))
            >
                "Desk pod (4)"
            </Button>
            {PALETTE_GROUPS
                .iter()
                .map(|&(category, label)| {
                    view! {
                        <div class="cn-floor-builder-group" role="group" aria-label=label>
                            {by_category(category)
                                .map(|entry| {
                                    let kind = entry.meta.kind;
                                    let name = entry.meta.label;
                                    view! {
                                        <Button
                                            size=ButtonSize::Sm
                                            variant=variant_for(Tool::Place(kind))
                                            on:click=move |_| tool.set(Tool::Place(kind))
                                        >
                                            {name}
                                        </Button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })
                .collect_view()}
            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline on:click=on_undo>
                "Undo"
            </Button>
            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline on:click=on_redo>
                "Redo"
            </Button>
        </div>
    }
}

// Snapshot tests need `to_html`, which is `ssr`-only.
#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use leptos::prelude::*;

    use super::FloorBuilder;
    use crate::model::SceneNodeId;
    use crate::samples;

    #[test]
    fn builder_renders_toolbar_palette_and_scene() {
        let html = Owner::new().with(|| {
            let scene = RwSignal::new(samples::office());
            let selected = RwSignal::new(Option::<SceneNodeId>::None);
            view! { <FloorBuilder scene=scene selected=selected /> }.to_html()
        });
        insta::assert_snapshot!("builder_office", html);
    }
}
