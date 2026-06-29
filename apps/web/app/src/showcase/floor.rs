//! Floor-plan showcase: the read-only `floorplan::FloorPlan` renderer over an
//! in-Rust sample scene, with a state legend. Exercises pan/zoom, reactive
//! `data-state`, selection, and the accessibility surface (the `/ui/floor` axe gate).

use std::collections::HashMap;

use floorplan::{FloorPlan, SceneNodeId, SpaceState, samples};
use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{Button, ButtonSize, ButtonVariant, Card, CardContent, CardHeader, CardTitle};

/// The six floor states with the Lucide icon + label the legend shows (matching the
/// renderer's per-node glyph, so the floor reads by icon and label, never colour
/// alone).
const FLOOR_LEGEND: &[(SpaceState, icondata::Icon, &str)] = &[
    (SpaceState::Free, icondata::LuPlus, "Free"),
    (
        SpaceState::PartiallyFree,
        icondata::LuMinus,
        "Partially free",
    ),
    (SpaceState::NotFree, icondata::LuX, "Not free"),
    (
        SpaceState::TemporarilyBlocked,
        icondata::LuClock,
        "Temporarily blocked",
    ),
    (
        SpaceState::PermanentUser,
        icondata::LuUser,
        "Permanent user",
    ),
    (
        SpaceState::CannotBeBooked,
        icondata::LuBan,
        "Cannot be booked",
    ),
];

/// Initial demo availability — a few desks in distinct states.
fn demo_floor_states() -> HashMap<SceneNodeId, SpaceState> {
    HashMap::from([
        (SceneNodeId::new("desk-1"), SpaceState::Free),
        (SceneNodeId::new("desk-2"), SpaceState::NotFree),
        (SceneNodeId::new("desk-3"), SpaceState::PartiallyFree),
    ])
}

/// The next state in the demo cycle (drives the "cycle a desk" button, showing a
/// single-node reactive repaint).
fn next_floor_state(state: SpaceState) -> SpaceState {
    match state {
        SpaceState::Free => SpaceState::PartiallyFree,
        SpaceState::PartiallyFree => SpaceState::NotFree,
        SpaceState::NotFree => SpaceState::TemporarilyBlocked,
        SpaceState::TemporarilyBlocked => SpaceState::PermanentUser,
        SpaceState::PermanentUser => SpaceState::CannotBeBooked,
        SpaceState::CannotBeBooked => SpaceState::Free,
    }
}

/// The read-only floor renderer over a sample scene, with a state legend.
#[component]
pub fn FloorPage() -> impl IntoView {
    let scene = samples::office();
    let states = RwSignal::new(demo_floor_states());
    let selected = RwSignal::new(Option::<String>::None);
    let on_select =
        Callback::new(move |id: SceneNodeId| selected.set(Some(id.as_str().to_owned())));

    let cycle_desk = move |_| {
        states.update(|map| {
            let entry = map
                .entry(SceneNodeId::new("desk-1"))
                .or_insert(SpaceState::Free);
            *entry = next_floor_state(*entry);
        });
    };

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex flex-col gap-1">
                <h1 class="cn-font-heading text-2xl font-semibold tracking-tight">"Floor plan"</h1>
                <p class="text-muted-foreground text-sm">
                    "Read-only inline-SVG renderer. Drag to pan, scroll to zoom, Tab to a bookable desk, Enter to select."
                </p>
            </div>
            <Card>
                <CardHeader class="flex flex-row items-center justify-between gap-4">
                    <CardTitle class="text-base">"Engineering — Floor 2 (sample)"</CardTitle>
                    <div class="flex items-center gap-3">
                        <span class="text-muted-foreground text-sm tabular-nums">
                            {move || {
                                selected
                                    .get()
                                    .map_or_else(
                                        || "No selection".to_owned(),
                                        |id| format!("Selected: {id}"),
                                    )
                            }}
                        </span>
                        <Button
                            size=ButtonSize::Sm
                            variant=ButtonVariant::Outline
                            on:click=cycle_desk
                        >
                            "Cycle a desk"
                        </Button>
                    </div>
                </CardHeader>
                <CardContent class="flex flex-col gap-4">
                    <div class="bg-muted/20 aspect-[5/3] w-full overflow-hidden rounded-md border">
                        <FloorPlan scene=scene states=states on_select=on_select />
                    </div>
                    <FloorLegend />
                </CardContent>
            </Card>
        </div>
    }
}

/// The state legend: each of the six states by icon and label.
#[component]
fn FloorLegend() -> impl IntoView {
    view! {
        <ul class="flex flex-wrap gap-x-4 gap-y-2" aria-label="Floor state legend">
            {FLOOR_LEGEND
                .iter()
                .map(|&(state, icon, label)| {
                    view! {
                        <li
                            class="cn-floor-legend text-muted-foreground flex items-center gap-1.5 text-sm"
                            data-state=state.as_str()
                        >
                            <span class="cn-floor-legend-swatch inline-flex size-5 items-center justify-center rounded border">
                                <Icon icon=icon attr:class="size-3.5" />
                            </span>
                            {label}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}
