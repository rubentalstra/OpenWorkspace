//! The floor builder's server boundary: wasm-safe DTOs and the authz-gated
//! `#[server]` functions that load, save and supply the builder. The ssr-only
//! [`backend`] module authorizes (`FloorBuild`) and maps DTOs ↔ `db` types; the
//! pages live in [`page`].

pub mod page;

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use serde::{Deserialize, Serialize};

use crate::CsrfClient;

/// A floor shown in the picker.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FloorDto {
    pub id: String,
    pub name: String,
    pub building: Option<String>,
}

/// A catalog equipment item.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquipmentItemDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// One assigned equipment item + quantity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquipAssignDto {
    pub item_id: String,
    pub quantity: i32,
}

/// Booking-policy fields (optional limits = "no limit").
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct RulesDto {
    pub max_advance_days: Option<i32>,
    pub min_advance_minutes: Option<i32>,
    pub min_duration_minutes: Option<i32>,
    pub max_duration_minutes: Option<i32>,
    pub slot_granularity_minutes: Option<i32>,
    pub buffer_minutes: Option<i32>,
    pub max_per_user_per_day: Option<i32>,
    pub max_active_per_user: Option<i32>,
    pub cancellation_deadline_minutes: Option<i32>,
    pub allow_recurrence: bool,
    pub max_recurrence_count: Option<i32>,
    pub max_recurrence_horizon_days: Option<i32>,
    pub require_approval: bool,
}

/// A bookable node's resource configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceDto {
    /// `None` creates a new resource on save.
    pub resource_id: Option<String>,
    pub scene_node_id: String,
    /// `domain::ResourceKind` token: `desk` / `room` / `parking` / `equipment`.
    pub kind: String,
    pub name: String,
    pub code: Option<String>,
    pub category_id: Option<String>,
    pub capacity: Option<i32>,
    pub bookable: bool,
    pub requires_checkin: bool,
    pub is_accessible: bool,
    pub description: Option<String>,
    pub rules: RulesDto,
    pub equipment: Vec<EquipAssignDto>,
}

/// A floor zone bound to a scene polygon.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZoneDto {
    pub zone_id: Option<String>,
    pub scene_node_id: String,
    pub name: String,
    pub organization_id: Option<String>,
    pub team_id: Option<String>,
}

/// The whole editable document the builder saves atomically.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildDocDto {
    pub floor_id: String,
    pub scene: floorplan::Scene,
    pub viewbox: Option<String>,
    /// `None` on the first save; the optimistic-lock guard otherwise.
    pub expected_version: Option<i32>,
    /// `draft` or `published`.
    pub status: String,
    pub resources: Vec<ResourceDto>,
    pub zones: Vec<ZoneDto>,
}

/// The builder's initial load: the document, the floor name, and the catalog.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadedBuildDto {
    pub floor_name: String,
    pub doc: BuildDocDto,
    pub equipment_catalog: Vec<EquipmentItemDto>,
}

/// Floors the signed-in admin can open in the builder.
#[server(input = GetUrl)]
pub async fn list_buildable_floors() -> Result<Vec<FloorDto>, ServerFnError> {
    backend::list_floors().await
}

/// Loads a floor's builder document (authz: FloorBuild on the floor).
#[server(input = GetUrl)]
pub async fn load_build_doc(floor_id: String) -> Result<LoadedBuildDto, ServerFnError> {
    backend::load(floor_id).await
}

/// Persists the whole document atomically; returns the new version (authz:
/// FloorBuild on the floor).
#[server(client = CsrfClient)]
pub async fn save_build_doc(doc: BuildDocDto) -> Result<i32, ServerFnError> {
    backend::save(doc).await
}

/// Adds a catalog equipment item (authz: FloorBuild somewhere — any builder).
#[server(client = CsrfClient)]
pub async fn create_equipment_item(
    name: String,
    description: Option<String>,
) -> Result<EquipmentItemDto, ServerFnError> {
    backend::create_equipment(name, description).await
}

#[cfg(feature = "ssr")]
mod backend {
    use super::{
        BuildDocDto, EquipAssignDto, EquipmentItemDto, FloorDto, LoadedBuildDto, ResourceDto,
        RulesDto, ZoneDto,
    };
    use auth::{AuthSession, AuthzBackend, Target};
    use domain::authz::Action;
    use domain::{
        EquipmentItemId, FloorZoneId, LocationId, OrganizationId, ResourceId, TeamId, UserId,
    };
    use leptos::prelude::*;
    use uuid::Uuid;

    fn db() -> db::Db {
        expect_context::<db::Db>()
    }

    fn authz() -> AuthzBackend {
        expect_context::<AuthzBackend>()
    }

    fn parse_id(id: &str) -> Result<Uuid, ServerFnError> {
        Uuid::parse_str(id).map_err(|_| ServerFnError::new("invalid id"))
    }

    async fn current_user() -> Result<UserId, ServerFnError> {
        let session: AuthSession = leptos_axum::extract().await?;
        session
            .user
            .map(|u| u.id)
            .ok_or_else(|| ServerFnError::new("not authenticated"))
    }

    /// Authorizes `FloorBuild` on a floor and returns the actor.
    async fn ensure_floor_build(floor: LocationId) -> Result<UserId, ServerFnError> {
        let user = current_user().await?;
        authz()
            .authorize(user, Action::FloorBuild, Target::Location(floor), None)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(user)
    }

    pub(super) async fn list_floors() -> Result<Vec<FloorDto>, ServerFnError> {
        // The list is a convenience; FloorBuild is enforced per floor on load/save.
        let _ = current_user().await?;
        let floors = db::list_floors(&db())
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(floors
            .into_iter()
            .map(|f| FloorDto {
                id: f.id.to_string(),
                name: f.name,
                building: f.building,
            })
            .collect())
    }

    pub(super) async fn load(floor_id: String) -> Result<LoadedBuildDto, ServerFnError> {
        let floor = LocationId::new(parse_id(&floor_id)?);
        ensure_floor_build(floor).await?;
        let pool = db();

        let row = db::load_floor_plan(&pool, floor)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        let (scene, viewbox, expected_version, status) = match row {
            Some(r) => {
                let version = u32::try_from(r.scene_schema_version).unwrap_or(0);
                let scene = floorplan::load_scene(r.scene, version)
                    .map_err(|e| ServerFnError::new(e.to_string()))?;
                let status = match r.status {
                    db::FloorPlanStatusRow::Draft => "draft",
                    db::FloorPlanStatusRow::Published => "published",
                };
                (scene, r.viewbox, Some(r.version), status.to_owned())
            }
            None => (floorplan::Scene::default(), None, None, "draft".to_owned()),
        };

        let resources = db::list_resources(&pool, floor)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .into_iter()
            .map(resource_row_to_dto)
            .collect();
        let zones = db::list_floor_zones(&pool, floor)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .into_iter()
            .filter_map(zone_row_to_dto)
            .collect();
        let equipment_catalog = db::list_equipment_items(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .into_iter()
            .map(|i| EquipmentItemDto {
                id: i.id.to_string(),
                name: i.name,
                description: i.description,
            })
            .collect();

        let floor_name = db::list_floors(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .into_iter()
            .find(|f| f.id == floor.as_uuid())
            .map_or_else(|| "Floor".to_owned(), |f| f.name);

        Ok(LoadedBuildDto {
            floor_name,
            doc: BuildDocDto {
                floor_id,
                scene,
                viewbox,
                expected_version,
                status,
                resources,
                zones,
            },
            equipment_catalog,
        })
    }

    pub(super) async fn save(doc: BuildDocDto) -> Result<i32, ServerFnError> {
        let floor = LocationId::new(parse_id(&doc.floor_id)?);
        let user = ensure_floor_build(floor).await?;

        let scene_value =
            serde_json::to_value(&doc.scene).map_err(|e| ServerFnError::new(e.to_string()))?;
        let status = match doc.status.as_str() {
            "published" => db::FloorPlanStatusRow::Published,
            _ => db::FloorPlanStatusRow::Draft,
        };
        let resources = doc
            .resources
            .into_iter()
            .map(resource_dto_to_spec)
            .collect::<Result<Vec<_>, _>>()?;
        let zones = doc
            .zones
            .into_iter()
            .map(zone_dto_to_spec)
            .collect::<Result<Vec<_>, _>>()?;

        let builder_doc = db::FloorBuilderDoc {
            floor_id: floor,
            scene: scene_value,
            scene_schema_version: i32::try_from(doc.scene.schema_version).unwrap_or(1),
            viewbox: doc.viewbox,
            background_asset_id: None,
            status,
            expected_version: doc.expected_version,
            updated_by: Some(user),
            resources,
            zones,
        };
        db::save_floor_builder_doc(&db(), &builder_doc)
            .await
            .map_err(|e| match e {
                db::DbError::StaleState => {
                    ServerFnError::new("the plan changed since you opened it; reload and retry")
                }
                other => ServerFnError::new(other.to_string()),
            })
    }

    pub(super) async fn create_equipment(
        name: String,
        description: Option<String>,
    ) -> Result<EquipmentItemDto, ServerFnError> {
        // Any authenticated builder may extend the shared catalog.
        let _ = current_user().await?;
        let id = db::create_equipment_item(&db(), &name, description.as_deref())
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(EquipmentItemDto {
            id: id.as_uuid().to_string(),
            name,
            description,
        })
    }

    fn resource_kind_row(token: &str) -> Result<db::ResourceKindRow, ServerFnError> {
        Ok(match token {
            "desk" => db::ResourceKindRow::Desk,
            "room" => db::ResourceKindRow::Room,
            "parking" => db::ResourceKindRow::Parking,
            "equipment" => db::ResourceKindRow::Equipment,
            _ => return Err(ServerFnError::new("unknown resource kind")),
        })
    }

    fn resource_kind_token(kind: db::ResourceKindRow) -> &'static str {
        match kind {
            db::ResourceKindRow::Desk => "desk",
            db::ResourceKindRow::Room => "room",
            db::ResourceKindRow::Parking => "parking",
            db::ResourceKindRow::Equipment => "equipment",
        }
    }

    fn rules_dto_to_spec(r: RulesDto) -> db::ResourceRulesSpec {
        db::ResourceRulesSpec {
            max_advance_days: r.max_advance_days,
            min_advance_minutes: r.min_advance_minutes,
            min_duration_minutes: r.min_duration_minutes,
            max_duration_minutes: r.max_duration_minutes,
            slot_granularity_minutes: r.slot_granularity_minutes,
            buffer_minutes: r.buffer_minutes,
            max_per_user_per_day: r.max_per_user_per_day,
            max_active_per_user: r.max_active_per_user,
            cancellation_deadline_minutes: r.cancellation_deadline_minutes,
            allow_recurrence: r.allow_recurrence,
            max_recurrence_count: r.max_recurrence_count,
            max_recurrence_horizon_days: r.max_recurrence_horizon_days,
            require_approval: r.require_approval,
        }
    }

    fn resource_dto_to_spec(d: ResourceDto) -> Result<db::ResourceSpec, ServerFnError> {
        let resource_id = d
            .resource_id
            .as_deref()
            .map(parse_id)
            .transpose()?
            .map(ResourceId::new);
        let category_id = d.category_id.as_deref().map(parse_id).transpose()?;
        let equipment = d
            .equipment
            .into_iter()
            .map(|e| Ok((EquipmentItemId::new(parse_id(&e.item_id)?), e.quantity)))
            .collect::<Result<Vec<_>, ServerFnError>>()?;
        Ok(db::ResourceSpec {
            resource_id,
            scene_node_id: d.scene_node_id,
            kind: resource_kind_row(&d.kind)?,
            name: d.name,
            code: d.code,
            category_id,
            capacity: d.capacity,
            bookable: d.bookable,
            requires_checkin: d.requires_checkin,
            is_accessible: d.is_accessible,
            description: d.description,
            status: db::ResourceStatusRow::Active,
            rules: rules_dto_to_spec(d.rules),
            equipment,
        })
    }

    fn zone_dto_to_spec(d: ZoneDto) -> Result<db::ZoneSpec, ServerFnError> {
        Ok(db::ZoneSpec {
            zone_id: d
                .zone_id
                .as_deref()
                .map(parse_id)
                .transpose()?
                .map(FloorZoneId::new),
            scene_node_id: d.scene_node_id,
            name: d.name,
            organization_id: d
                .organization_id
                .as_deref()
                .map(parse_id)
                .transpose()?
                .map(OrganizationId::new),
            team_id: d
                .team_id
                .as_deref()
                .map(parse_id)
                .transpose()?
                .map(TeamId::new),
        })
    }

    fn resource_row_to_dto(r: db::ResourceRow) -> ResourceDto {
        ResourceDto {
            resource_id: Some(r.id.to_string()),
            scene_node_id: r.scene_node_id,
            kind: resource_kind_token(r.kind).to_owned(),
            name: r.name,
            code: r.code,
            category_id: r.category_id.map(|c| c.to_string()),
            capacity: r.capacity,
            bookable: r.bookable,
            requires_checkin: r.requires_checkin,
            is_accessible: r.is_accessible,
            description: r.description,
            rules: RulesDto {
                max_advance_days: r.rules.max_advance_days,
                min_advance_minutes: r.rules.min_advance_minutes,
                min_duration_minutes: r.rules.min_duration_minutes,
                max_duration_minutes: r.rules.max_duration_minutes,
                slot_granularity_minutes: r.rules.slot_granularity_minutes,
                buffer_minutes: r.rules.buffer_minutes,
                max_per_user_per_day: r.rules.max_per_user_per_day,
                max_active_per_user: r.rules.max_active_per_user,
                cancellation_deadline_minutes: r.rules.cancellation_deadline_minutes,
                allow_recurrence: r.rules.allow_recurrence,
                max_recurrence_count: r.rules.max_recurrence_count,
                max_recurrence_horizon_days: r.rules.max_recurrence_horizon_days,
                require_approval: r.rules.require_approval,
            },
            equipment: r
                .equipment
                .into_iter()
                .map(|(item, quantity)| EquipAssignDto {
                    item_id: item.to_string(),
                    quantity,
                })
                .collect(),
        }
    }

    fn zone_row_to_dto(r: db::ZoneRow) -> Option<ZoneDto> {
        r.scene_node_id.map(|node| ZoneDto {
            zone_id: Some(r.id.to_string()),
            scene_node_id: node,
            name: r.name,
            organization_id: r.organization_id.map(|o| o.to_string()),
            team_id: r.team_id.map(|t| t.to_string()),
        })
    }
}
