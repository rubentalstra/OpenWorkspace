//! Floor-plan persistence: the `floor_plans` table (one row per floor location).
//!
//! The scene is stored and read as opaque jsonb (`serde_json::Value`); this crate
//! carries no `floorplan` dependency in production. The consumer feeds
//! `(row.scene, row.scene_schema_version)` into `floorplan::load_scene` so the scene
//! migration chain runs on load and any older persisted scene upgrades to current.

use domain::LocationId;
use serde_json::Value;
use uuid::Uuid;

use crate::{Db, DbError};

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

#[cfg(test)]
mod tests {
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
}
