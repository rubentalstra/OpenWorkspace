//! The floor-builder pages: `/build` (picker) and `/build/:floor_id` (the editor +
//! resource/equipment panel + Save/Publish). The interactive canvas is
//! `floorplan::FloorBuilder`; this page wires it to the server fns.

use std::collections::HashMap;

use floorplan::builder::FloorBuilder;
use floorplan::{CatalogKind, Scene, SceneNodeId};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use ui::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, CardHeader, CardTitle, Input,
    NativeSelect, NativeSelectOption,
};

use super::{
    BuildDocDto, CampusDto, EquipAssignDto, EquipmentItemDto, ResourceDto, RulesDto,
    list_buildable_floors, load_build_doc, load_campus, move_building_marker, save_build_doc,
};

/// `/build` — the floors the admin can open in the builder.
#[component]
pub fn BuildIndexPage() -> impl IntoView {
    let floors = Resource::new(|| (), |()| list_buildable_floors());
    view! {
        <div class="container flex flex-col gap-6 py-8">
            <h1 class="cn-font-heading text-2xl font-semibold tracking-tight">"Floor builder"</h1>
            <Suspense fallback=move || {
                view! { <p class="text-muted-foreground">"Loading…"</p> }
            }>
                {move || Suspend::new(async move {
                    match floors.await {
                        Ok(list) if list.is_empty() => {
                            view! { <p class="text-muted-foreground">"No floors yet."</p> }
                                .into_any()
                        }
                        Ok(list) => {
                            view! {
                                <ul class="flex flex-col gap-2">
                                    {list
                                        .into_iter()
                                        .map(|f| {
                                            let href = format!("/build/{}", f.id);
                                            let sub = f.building.unwrap_or_default();
                                            view! {
                                                <li>
                                                    <Button
                                                        href=href
                                                        variant=ButtonVariant::Outline
                                                        class="w-full justify-start gap-2"
                                                    >
                                                        <span class="font-medium">{f.name}</span>
                                                        <span class="text-muted-foreground text-sm">{sub}</span>
                                                    </Button>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            }
                                .into_any()
                        }
                        Err(e) => sign_in_or_error(&e.to_string()),
                    }
                })}
            </Suspense>
        </div>
    }
}

fn sign_in_or_error(message: &str) -> AnyView {
    let msg = message.to_owned();
    view! {
        <div class="flex flex-col items-start gap-2">
            <p class="text-destructive text-sm">{msg}</p>
            <Button href="/login".to_owned() variant=ButtonVariant::Outline>
                "Sign in"
            </Button>
        </div>
    }
    .into_any()
}

/// `/build/:floor_id` — the editor.
#[component]
pub fn BuildPage() -> impl IntoView {
    let params = use_params_map();
    let floor_id = move || params.read().get("floor_id").unwrap_or_default();
    let loaded = Resource::new(floor_id, load_build_doc);

    view! {
        <Suspense fallback=move || {
            view! { <p class="text-muted-foreground p-8">"Loading floor…"</p> }
        }>
            {move || Suspend::new(async move {
                match loaded.await {
                    Ok(data) => editor_view(data.floor_name, data.doc, data.equipment_catalog),
                    Err(e) => sign_in_or_error(&e.to_string()),
                }
            })}
        </Suspense>
    }
}

fn editor_view(
    floor_name: String,
    doc: BuildDocDto,
    equipment_catalog: Vec<EquipmentItemDto>,
) -> AnyView {
    let floor_id = doc.floor_id.clone();
    let scene = RwSignal::new(doc.scene);
    let selected = RwSignal::new(Option::<SceneNodeId>::None);
    let version = RwSignal::new(doc.expected_version);
    let equipment = RwSignal::new(equipment_catalog);
    let clipboard = RwSignal::new(Option::<ResourceDto>::None);
    let status = RwSignal::new(String::new());
    // Resource configs keyed by scene_node_id.
    let resources: RwSignal<HashMap<String, ResourceDto>> = RwSignal::new(
        doc.resources
            .into_iter()
            .map(|r| (r.scene_node_id.clone(), r))
            .collect(),
    );

    let save = StoredValue::new(move |publish: bool| {
        let floor_id = floor_id.clone();
        spawn_local(async move {
            let scene_now = scene.get_untracked();
            // Only keep resource configs whose bookable node still exists.
            let live: Vec<ResourceDto> = resources
                .get_untracked()
                .into_values()
                .filter(|r| {
                    scene_now
                        .nodes
                        .iter()
                        .any(|n| n.id.as_str() == r.scene_node_id && n.kind.bookable())
                })
                .collect();
            let dto = BuildDocDto {
                floor_id,
                scene: scene_now,
                viewbox: None,
                expected_version: version.get_untracked(),
                status: if publish { "published" } else { "draft" }.to_owned(),
                resources: live,
                zones: Vec::new(),
            };
            match save_build_doc(dto).await {
                Ok(v) => {
                    version.set(Some(v));
                    status.set(if publish {
                        "Published".into()
                    } else {
                        "Saved".into()
                    });
                }
                Err(e) => status.set(e.to_string()),
            }
        });
    });

    view! {
        <div class="cn-floor-builder-page flex h-svh flex-col">
            <header class="flex items-center justify-between gap-4 border-b px-4 py-2">
                <a href="/build" class="text-muted-foreground text-sm">
                    "← Floors"
                </a>
                <h1 class="cn-font-heading font-semibold">{floor_name}</h1>
                <div class="flex items-center gap-2">
                    <span class="text-muted-foreground text-sm">{move || status.get()}</span>
                    <Button
                        size=ButtonSize::Sm
                        variant=ButtonVariant::Outline
                        on:click=move |_| save.with_value(|f| f(false))
                    >
                        "Save draft"
                    </Button>
                    <Button size=ButtonSize::Sm on:click=move |_| save.with_value(|f| f(true))>
                        "Publish"
                    </Button>
                </div>
            </header>
            <div class="flex min-h-0 flex-1">
                <div class="min-w-0 flex-1">
                    <FloorBuilder scene=scene selected=selected />
                </div>
                <aside class="w-80 shrink-0 overflow-y-auto border-l p-3">
                    <ResourcePanel
                        scene=scene
                        selected=selected
                        resources=resources
                        equipment=equipment
                        clipboard=clipboard
                    />
                </aside>
            </div>
        </div>
    }
    .into_any()
}

/// The inspector for the selected bookable node: its resource config, rules,
/// equipment, and copy/paste.
#[component]
#[expect(
    clippy::implicit_hasher,
    reason = "a component prop cannot be generic over the map's BuildHasher"
)]
fn ResourcePanel(
    scene: RwSignal<Scene>,
    selected: RwSignal<Option<SceneNodeId>>,
    resources: RwSignal<HashMap<String, ResourceDto>>,
    equipment: RwSignal<Vec<EquipmentItemDto>>,
    clipboard: RwSignal<Option<ResourceDto>>,
) -> impl IntoView {
    // The selected bookable node's id + kind token, if any.
    let selected_bookable = Memo::new(move |_| {
        let id = selected.get()?;
        scene.with(|s| {
            s.nodes
                .iter()
                .find(|n| n.id == id)
                .filter(|n| n.kind.bookable())
                .and_then(|n| {
                    n.kind
                        .resource_kind()
                        .map(|_| (id.as_str().to_owned(), n.kind))
                })
        })
    });

    move || {
        let Some((node_id, kind)) = selected_bookable.get() else {
            return view! {
                <p class="text-muted-foreground text-sm">
                    "Select a bookable node (desk, room, parking) to configure it."
                </p>
            }
            .into_any();
        };
        let kind_token = catalog_kind_token(kind);
        // Ensure a config exists for this node.
        resources.update(|map| {
            map.entry(node_id.clone()).or_insert_with(|| ResourceDto {
                resource_id: None,
                scene_node_id: node_id.clone(),
                kind: kind_token.to_owned(),
                name: default_name(&node_id),
                code: Some(default_name(&node_id)),
                category_id: None,
                capacity: None,
                bookable: true,
                requires_checkin: true,
                is_accessible: false,
                description: None,
                rules: RulesDto {
                    allow_recurrence: true,
                    ..RulesDto::default()
                },
                equipment: Vec::new(),
            });
        });

        let id_for_field = node_id.clone();
        let name = move || {
            resources.with(|m| {
                m.get(&id_for_field)
                    .map(|r| r.name.clone())
                    .unwrap_or_default()
            })
        };
        let id_set_name = node_id.clone();
        let set_name = move |v: String| {
            resources.update(|m| {
                if let Some(r) = m.get_mut(&id_set_name) {
                    r.name = v;
                }
            });
        };

        let id_copy = node_id.clone();
        let copy = move |_| {
            let cfg = resources.with(|m| m.get(&id_copy).cloned());
            clipboard.set(cfg);
        };
        let paste = move |_| {
            if let Some(src) = clipboard.get_untracked() {
                let targets: Vec<(String, CatalogKind)> = scene.with_untracked(|s| {
                    s.nodes
                        .iter()
                        .filter(|n| n.kind.bookable())
                        .map(|n| (n.id.as_str().to_owned(), n.kind))
                        .collect()
                });
                resources.update(|m| {
                    for (tid, tkind) in targets {
                        // Copy the shared config but keep each node's own identity.
                        let (name, code, resource_id) = m.get(&tid).map_or_else(
                            || (default_name(&tid), Some(default_name(&tid)), None),
                            |e| (e.name.clone(), e.code.clone(), e.resource_id.clone()),
                        );
                        let mut cfg = src.clone();
                        cfg.scene_node_id.clone_from(&tid);
                        cfg.name = name;
                        cfg.code = code;
                        cfg.resource_id = resource_id;
                        cfg.kind = catalog_kind_token(tkind).into();
                        m.insert(tid, cfg);
                    }
                });
            }
        };

        view! {
            <Card>
                <CardHeader class="flex flex-row items-center justify-between">
                    <CardTitle class="text-base">{format!("Resource · {kind_token}")}</CardTitle>
                    <div class="flex gap-1">
                        <Button size=ButtonSize::Xs variant=ButtonVariant::Outline on:click=copy>
                            "Copy"
                        </Button>
                        <Button
                            size=ButtonSize::Xs
                            variant=ButtonVariant::Outline
                            on:click=paste
                            attr:disabled=move || clipboard.get().is_none()
                        >
                            "Paste to all"
                        </Button>
                    </div>
                </CardHeader>
                <CardContent class="flex flex-col gap-3">
                    <label class="flex flex-col gap-1 text-sm">
                        "Name"
                        <Input
                            prop:value=name
                            on:input=move |ev| set_name(event_target_value(&ev))
                        />
                    </label>
                    <EquipmentEditor node_id=node_id resources=resources equipment=equipment />
                </CardContent>
            </Card>
        }
        .into_any()
    }
}

/// Assign catalog equipment (with quantity) to the selected resource.
#[component]
#[expect(
    clippy::implicit_hasher,
    reason = "a component prop cannot be generic over the map's BuildHasher"
)]
fn EquipmentEditor(
    node_id: String,
    resources: RwSignal<HashMap<String, ResourceDto>>,
    equipment: RwSignal<Vec<EquipmentItemDto>>,
) -> impl IntoView {
    let id_list = node_id.clone();
    let assigned = move || {
        resources.with(|m| {
            m.get(&id_list)
                .map(|r| r.equipment.clone())
                .unwrap_or_default()
        })
    };
    let id_add = node_id;
    let add = move |item_id: String| {
        resources.update(|m| {
            if let Some(r) = m.get_mut(&id_add)
                && !r.equipment.iter().any(|e| e.item_id == item_id)
            {
                r.equipment.push(EquipAssignDto {
                    item_id,
                    quantity: 1,
                });
            }
        });
    };
    let name_of = move |item_id: &str| {
        equipment.with(|cat| {
            cat.iter()
                .find(|i| i.id == item_id)
                .map_or_else(|| item_id.to_owned(), |i| i.name.clone())
        })
    };

    view! {
        <div class="flex flex-col gap-2 text-sm">
            <span class="text-muted-foreground">"Equipment"</span>
            <ul class="flex flex-col gap-1">
                {move || {
                    assigned()
                        .into_iter()
                        .map(|e| {
                            let label = name_of(&e.item_id);
                            view! {
                                <li class="flex items-center justify-between gap-2">
                                    <span>{label}</span>
                                    <span class="text-muted-foreground tabular-nums">
                                        {format!("×{}", e.quantity)}
                                    </span>
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
            <NativeSelect on:change=move |ev| {
                let v = event_target_value(&ev);
                if !v.is_empty() {
                    add(v);
                }
            }>
                <NativeSelectOption attr:value="">"Add equipment…"</NativeSelectOption>
                {move || {
                    equipment
                        .get()
                        .into_iter()
                        .map(|i| {
                            view! {
                                <NativeSelectOption attr:value=i.id>{i.name}</NativeSelectOption>
                            }
                        })
                        .collect_view()
                }}
            </NativeSelect>
        </div>
    }
}

fn default_name(node_id: &str) -> String {
    node_id.replace(['-', '_'], " ")
}

fn catalog_kind_token(kind: CatalogKind) -> &'static str {
    match kind {
        CatalogKind::MeetingRoom => "room",
        CatalogKind::ParkingSpace => "parking",
        _ => "desk",
    }
}

/// `/build/campus/:campus_id` — the campus map + draggable building markers.
#[component]
pub fn CampusPage() -> impl IntoView {
    let params = use_params_map();
    let campus_id = move || params.read().get("campus_id").unwrap_or_default();
    let loaded = Resource::new(campus_id, load_campus);

    view! {
        <Suspense fallback=move || {
            view! { <p class="text-muted-foreground p-8">"Loading campus…"</p> }
        }>
            {move || Suspend::new(async move {
                match loaded.await {
                    Ok(campus) => campus_editor_view(campus),
                    Err(e) => sign_in_or_error(&e.to_string()),
                }
            })}
        </Suspense>
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the campus canvas wires the map, markers and drag handlers in one view"
)]
fn campus_editor_view(campus: CampusDto) -> AnyView {
    let container = NodeRef::<leptos::html::Div>::new();
    let positions: RwSignal<HashMap<String, (f64, f64)>> = RwSignal::new(
        campus
            .buildings
            .iter()
            .map(|b| {
                (
                    b.id.clone(),
                    (b.marker_x.unwrap_or(0.5), b.marker_y.unwrap_or(0.5)),
                )
            })
            .collect(),
    );
    let dragging = RwSignal::new(Option::<String>::None);
    let map_src = campus
        .map_image_asset_id
        .as_ref()
        .map(|a| format!("/api/assets/{a}"));

    let to_fraction = move |cx: f64, cy: f64| -> Option<(f64, f64)> {
        let el = container.get_untracked()?;
        let rect = el.get_bounding_client_rect();
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return None;
        }
        Some((
            ((cx - rect.left()) / rect.width()).clamp(0.0, 1.0),
            ((cy - rect.top()) / rect.height()).clamp(0.0, 1.0),
        ))
    };

    let on_move = move |ev: web_sys::PointerEvent| {
        if let Some(id) = dragging.get_untracked()
            && let Some(frac) = to_fraction(f64::from(ev.client_x()), f64::from(ev.client_y()))
        {
            positions.update(|m| {
                m.insert(id, frac);
            });
        }
    };
    let on_up = move |_: web_sys::PointerEvent| {
        let Some(id) = dragging.get_untracked() else {
            return;
        };
        dragging.set(None);
        if let Some((x, y)) = positions.with_untracked(|m| m.get(&id).copied()) {
            spawn_local(async move {
                if let Err(err) = move_building_marker(id, Some(x), Some(y)).await {
                    leptos::logging::error!("marker save failed: {err}");
                }
            });
        }
    };

    view! {
        <div class="cn-campus-editor flex h-svh flex-col">
            <header class="flex items-center justify-between gap-4 border-b px-4 py-2">
                <a href="/build" class="text-muted-foreground text-sm">
                    "← Floors"
                </a>
                <h1 class="cn-font-heading font-semibold">
                    {format!("Campus · {}", campus.name)}
                </h1>
                <span class="text-muted-foreground text-sm">
                    "Drag a marker to place a building"
                </span>
            </header>
            <div
                node_ref=container
                class="cn-campus-canvas relative min-h-0 flex-1"
                on:pointermove=on_move
                on:pointerup=on_up
                on:pointerleave=on_up
            >
                {map_src
                    .map(|src| {
                        view! {
                            <img
                                src=src
                                alt="Campus map"
                                class="pointer-events-none absolute inset-0 h-full w-full object-contain"
                            />
                        }
                    })}
                {campus
                    .buildings
                    .into_iter()
                    .map(|b| {
                        let id_pos = b.id.clone();
                        let id_down = b.id.clone();
                        let label = b.name.clone();
                        let aria = b.name;
                        let pos = move || {
                            positions.with(|m| m.get(&id_pos).copied().unwrap_or((0.5, 0.5)))
                        };
                        let on_down = move |ev: web_sys::PointerEvent| {
                            ev.stop_propagation();
                            dragging.set(Some(id_down.clone()));
                        };
                        view! {
                            <Button
                                class="cn-campus-marker"
                                variant=ButtonVariant::Default
                                attr:style=move || {
                                    let (x, y) = pos();
                                    format!("left:{}%;top:{}%", x * 100.0, y * 100.0)
                                }
                                on:pointerdown=on_down
                                attr:aria-label=aria
                            >
                                {label}
                            </Button>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
    .into_any()
}
