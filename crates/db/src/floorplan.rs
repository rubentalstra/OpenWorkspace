//! Floor-plan persistence: the `floor_plans` table (one row per floor location).
//!
//! The scene is stored and read as opaque jsonb (`serde_json::Value`); this crate
//! carries no `floorplan` dependency in production. The consumer feeds
//! `(row.scene, row.scene_schema_version)` into `floorplan::load_scene` so the scene
//! migration chain runs on load and any older persisted scene upgrades to current.

use domain::{AssetId, LocationId, UserId};
use serde_json::Value;
use uuid::Uuid;

use crate::locations::ZoneSpec;
use crate::locations::{delete_stale_zones, upsert_zone};
use crate::resources::{ResourceSpec, delete_stale_positions, upsert_position, upsert_resource};
use crate::{Db, DbError, classify, set_system_context};

/// Persistence-mapped mirror of the `floor_plan_status` enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "floor_plan_status", rename_all = "snake_case")]
pub enum FloorPlanStatusRow {
    /// Work in progress; not shown to end users.
    Draft,
    /// Live and visible.
    Published,
}

/// A row from `floor_plans`. `scene` stays opaque jsonb — pair it with
/// `scene_schema_version` for `floorplan::load_scene`.
#[derive(Clone, Debug)]
pub struct FloorPlanRow {
    pub floor_id: Uuid,
    pub scene: Value,
    pub scene_schema_version: i32,
    pub viewbox: Option<String>,
    pub background_asset_id: Option<Uuid>,
    pub status: FloorPlanStatusRow,
    pub version: i32,
}

/// Loads the floor plan for a floor location, or `None` if it has no plan row yet.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_floor_plan(
    pool: &Db,
    floor: LocationId,
) -> Result<Option<FloorPlanRow>, DbError> {
    let row = sqlx::query_as!(
        FloorPlanRow,
        r#"
        SELECT floor_id,
               scene AS "scene: Value",
               scene_schema_version,
               viewbox,
               background_asset_id,
               status AS "status: FloorPlanStatusRow",
               version
        FROM floor_plans
        WHERE floor_id = $1
        "#,
        floor.as_uuid(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// The whole floor-builder document committed atomically by [`save_floor_builder_doc`]:
/// the scene jsonb plus the resources/bindings and zones it references.
#[derive(Clone, Debug)]
pub struct FloorBuilderDoc {
    pub floor_id: LocationId,
    pub scene: Value,
    pub scene_schema_version: i32,
    pub viewbox: Option<String>,
    pub background_asset_id: Option<AssetId>,
    pub status: FloorPlanStatusRow,
    /// `None` on the first save (no row yet); `Some` is the optimistic-lock guard.
    pub expected_version: Option<i32>,
    pub updated_by: Option<UserId>,
    pub resources: Vec<ResourceSpec>,
    pub zones: Vec<ZoneSpec>,
}

/// Persists a whole floor-builder document in one transaction and returns the new
/// `floor_plans.version`: upserts the plan (optimistic-locked), reconciles
/// `resource_positions` (unbinding removed seats; resource entities are kept),
/// upserts the referenced resources + rules + equipment, and reconciles zones.
///
/// # Errors
///
/// [`DbError::StaleState`] if `expected_version` no longer matches (a concurrent
/// edit); [`DbError::Sqlx`] on any other database error.
pub async fn save_floor_builder_doc(pool: &Db, doc: &FloorBuilderDoc) -> Result<i32, DbError> {
    let mut tx = pool.begin().await.map_err(classify)?;
    // `resources` is RLS-guarded (P8); elevate so the writes are allowed.
    set_system_context(&mut tx).await?;

    let bg = doc.background_asset_id.map(AssetId::as_uuid);
    let updated_by = doc.updated_by.map(UserId::as_uuid);
    let published = matches!(doc.status, FloorPlanStatusRow::Published);

    let new_version = match doc.expected_version {
        Some(expected) => sqlx::query_scalar!(
            r#"
            UPDATE floor_plans
            SET scene = $2, scene_schema_version = $3, viewbox = $4,
                background_asset_id = $5, status = $6, version = version + 1,
                updated_by = $7,
                published_at = CASE WHEN $9 THEN now() ELSE published_at END
            WHERE floor_id = $1 AND version = $8
            RETURNING version
            "#,
            doc.floor_id.as_uuid(),
            doc.scene,
            doc.scene_schema_version,
            doc.viewbox,
            bg,
            doc.status as _,
            updated_by,
            expected,
            published,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(classify)?
        .ok_or(DbError::StaleState)?,
        None => sqlx::query_scalar!(
            r#"
            INSERT INTO floor_plans
                (floor_id, scene, scene_schema_version, viewbox, background_asset_id,
                 status, updated_by, published_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                    CASE WHEN $8 THEN now() ELSE NULL END)
            RETURNING version
            "#,
            doc.floor_id.as_uuid(),
            doc.scene,
            doc.scene_schema_version,
            doc.viewbox,
            bg,
            doc.status as _,
            updated_by,
            published,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(classify)?,
    };

    let mut bound_nodes = Vec::with_capacity(doc.resources.len());
    for spec in &doc.resources {
        let resource = upsert_resource(&mut tx, doc.floor_id, spec).await?;
        upsert_position(&mut tx, resource, doc.floor_id, &spec.scene_node_id).await?;
        bound_nodes.push(spec.scene_node_id.clone());
    }
    delete_stale_positions(&mut tx, doc.floor_id, &bound_nodes).await?;

    let mut zone_nodes = Vec::with_capacity(doc.zones.len());
    for zone in &doc.zones {
        upsert_zone(&mut tx, doc.floor_id, zone).await?;
        zone_nodes.push(zone.scene_node_id.clone());
    }
    delete_stale_zones(&mut tx, doc.floor_id, &zone_nodes).await?;

    tx.commit().await.map_err(classify)?;
    Ok(new_version)
}

#[cfg(test)]
mod tests {
    use domain::EquipmentItemId;

    use super::*;

    /// The full path the renderer uses: a scene stored as jsonb loads back through
    /// [`load_floor_plan`] and `floorplan::load_scene` byte-for-byte equal.
    #[sqlx::test]
    async fn scene_round_trips_through_jsonb(pool: Db) {
        let original = floorplan::samples::office();

        let floor: Uuid = sqlx::query_scalar(
            "INSERT INTO locations (kind, name, path, depth) \
             VALUES ('floor', 'Round Trip', '/rt', 0) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let scene_value = serde_json::to_value(&original).unwrap();
        sqlx::query(
            "INSERT INTO floor_plans (floor_id, scene, scene_schema_version) VALUES ($1, $2, $3)",
        )
        .bind(floor)
        .bind(&scene_value)
        .bind(i32::try_from(original.schema_version).unwrap())
        .execute(&pool)
        .await
        .unwrap();

        let row = load_floor_plan(&pool, LocationId::new(floor))
            .await
            .unwrap()
            .expect("the plan row exists");
        assert_eq!(row.status, FloorPlanStatusRow::Published);

        let loaded =
            floorplan::load_scene(row.scene, u32::try_from(row.scene_schema_version).unwrap())
                .unwrap();
        assert_eq!(loaded, original);
    }

    #[sqlx::test]
    async fn missing_floor_plan_is_none(pool: Db) {
        let absent = LocationId::new(Uuid::nil());
        assert!(load_floor_plan(&pool, absent).await.unwrap().is_none());
    }

    async fn seed_floor(pool: &Db) -> LocationId {
        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO locations (kind, name, path, depth) \
             VALUES ('floor', 'Build Floor', $1, 0) RETURNING id",
        )
        .bind(format!("/{}", Uuid::new_v4().simple()))
        .fetch_one(pool)
        .await
        .unwrap();
        LocationId::new(id)
    }

    fn desk_spec(node: &str, name: &str, equipment: Vec<(EquipmentItemId, i32)>) -> ResourceSpec {
        ResourceSpec {
            resource_id: None,
            scene_node_id: node.to_owned(),
            kind: crate::ResourceKindRow::Desk,
            name: name.to_owned(),
            code: Some(name.to_owned()),
            category_id: None,
            capacity: None,
            bookable: true,
            requires_checkin: true,
            is_accessible: false,
            description: None,
            status: crate::ResourceStatusRow::Active,
            rules: crate::ResourceRulesSpec {
                max_duration_minutes: Some(480),
                allow_recurrence: true,
                ..Default::default()
            },
            equipment,
        }
    }

    fn doc(
        floor: LocationId,
        expected: Option<i32>,
        resources: Vec<ResourceSpec>,
    ) -> FloorBuilderDoc {
        FloorBuilderDoc {
            floor_id: floor,
            scene: serde_json::json!({ "schema_version": 1, "nodes": [] }),
            scene_schema_version: 1,
            viewbox: Some("0 0 100 60".to_owned()),
            background_asset_id: None,
            status: FloorPlanStatusRow::Draft,
            expected_version: expected,
            updated_by: None,
            resources,
            zones: Vec::new(),
        }
    }

    #[sqlx::test]
    async fn save_creates_binds_then_unbinds_on_remove(pool: Db) {
        let floor = seed_floor(&pool).await;
        let monitor = crate::create_equipment_item(&pool, "24in Monitor", None)
            .await
            .unwrap();

        // First save (no row yet): one desk seat with equipment.
        let v1 = save_floor_builder_doc(
            &pool,
            &doc(
                floor,
                None,
                vec![desk_spec("seat-1", "Desk A1", vec![(monitor, 2)])],
            ),
        )
        .await
        .unwrap();
        assert_eq!(v1, 1);

        let resources = crate::list_resources(&pool, floor).await.unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].scene_node_id, "seat-1");
        assert_eq!(resources[0].equipment, vec![(monitor.as_uuid(), 2)]);

        // Second save: remove the seat → position unbinds, version bumps.
        let v2 = save_floor_builder_doc(&pool, &doc(floor, Some(v1), Vec::new()))
            .await
            .unwrap();
        assert_eq!(v2, 2);
        assert!(
            crate::list_resources(&pool, floor)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[sqlx::test]
    async fn save_rejects_stale_version(pool: Db) {
        let floor = seed_floor(&pool).await;
        let v1 = save_floor_builder_doc(&pool, &doc(floor, None, Vec::new()))
            .await
            .unwrap();
        // A second writer used v1; ours still thinks v1 but it's now v2.
        save_floor_builder_doc(&pool, &doc(floor, Some(v1), Vec::new()))
            .await
            .unwrap();
        let stale = save_floor_builder_doc(&pool, &doc(floor, Some(v1), Vec::new())).await;
        assert!(matches!(stale, Err(DbError::StaleState)));
    }
}
