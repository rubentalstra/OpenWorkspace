//! Location reads/writes the floor builder + campus editor need: the buildable
//! floor list, floor-zone upsert/reconcile, and the campus map-image + building
//! markers. Marker fractions are stored as `numeric(6,5)`; they cross the boundary
//! as `f64` via `::float8` / `::numeric` casts (no decimal dependency).

use domain::{AssetId, FloorZoneId, LocationId, OrganizationId, TeamId};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{Db, DbError, classify};

/// A floor in the picker (with its building name, if any).
#[derive(Clone, Debug)]
pub struct FloorSummary {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub building: Option<String>,
}

/// All `floor` locations, with their parent building name.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn list_floors(pool: &Db) -> Result<Vec<FloorSummary>, DbError> {
    let rows = sqlx::query!(
        r#"
        SELECT f.id, f.name, f.path, b.name AS "building?"
        FROM locations f
        LEFT JOIN locations b ON b.id = f.parent_id
        WHERE f.kind = 'floor' AND f.status = 'active'
        ORDER BY f.path
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(classify)?;
    Ok(rows
        .into_iter()
        .map(|r| FloorSummary {
            id: r.id,
            name: r.name,
            path: r.path,
            building: r.building,
        })
        .collect())
}

/// A floor zone (neighbourhood) the builder binds to a scene polygon.
#[derive(Clone, Debug)]
pub struct ZoneSpec {
    /// `None` creates a new zone; `Some` updates it.
    pub zone_id: Option<FloorZoneId>,
    pub scene_node_id: String,
    pub name: String,
    pub organization_id: Option<OrganizationId>,
    pub team_id: Option<TeamId>,
}

/// A loaded floor zone.
#[derive(Clone, Debug)]
pub struct ZoneRow {
    pub id: Uuid,
    pub scene_node_id: Option<String>,
    pub name: String,
    pub organization_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
}

/// Loads a floor's zones.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn list_floor_zones(pool: &Db, floor: LocationId) -> Result<Vec<ZoneRow>, DbError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, scene_node_id, name, organization_id, team_id
        FROM floor_zones WHERE floor_id = $1 ORDER BY name
        "#,
        floor.as_uuid(),
    )
    .fetch_all(pool)
    .await
    .map_err(classify)?;
    Ok(rows
        .into_iter()
        .map(|r| ZoneRow {
            id: r.id,
            scene_node_id: r.scene_node_id,
            name: r.name,
            organization_id: r.organization_id,
            team_id: r.team_id,
        })
        .collect())
}

pub(crate) async fn upsert_zone(
    conn: &mut PgConnection,
    floor: LocationId,
    spec: &ZoneSpec,
) -> Result<FloorZoneId, DbError> {
    let org = spec.organization_id.map(OrganizationId::as_uuid);
    let team = spec.team_id.map(TeamId::as_uuid);
    let id = if let Some(existing) = spec.zone_id {
        sqlx::query!(
            r#"
            UPDATE floor_zones
            SET name = $2, organization_id = $3, team_id = $4, scene_node_id = $5
            WHERE id = $1
            "#,
            existing.as_uuid(),
            spec.name,
            org,
            team,
            spec.scene_node_id,
        )
        .execute(&mut *conn)
        .await
        .map_err(classify)?;
        existing
    } else {
        let new = sqlx::query_scalar!(
            r#"
            INSERT INTO floor_zones (floor_id, name, organization_id, team_id, scene_node_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            floor.as_uuid(),
            spec.name,
            org,
            team,
            spec.scene_node_id,
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(classify)?;
        FloorZoneId::new(new)
    };
    Ok(id)
}

/// Deletes the floor's zones whose scene node is no longer present.
pub(crate) async fn delete_stale_zones(
    conn: &mut PgConnection,
    floor: LocationId,
    keep_nodes: &[String],
) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        DELETE FROM floor_zones
        WHERE floor_id = $1 AND COALESCE(scene_node_id, '') <> ALL($2)
        "#,
        floor.as_uuid(),
        keep_nodes,
    )
    .execute(&mut *conn)
    .await
    .map_err(classify)?;
    Ok(())
}

/// The campus + its child building markers, for the campus editor.
#[derive(Clone, Debug)]
pub struct CampusEditor {
    pub campus_id: Uuid,
    pub name: String,
    pub map_image_asset_id: Option<Uuid>,
    pub buildings: Vec<BuildingMarker>,
}

/// A child building and its (optional) marker position on the campus map.
#[derive(Clone, Debug)]
pub struct BuildingMarker {
    pub id: Uuid,
    pub name: String,
    pub marker_x: Option<f64>,
    pub marker_y: Option<f64>,
}

/// Loads a campus and its child buildings (+ marker fractions). `None` if the id is
/// not a campus.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_campus_editor(
    pool: &Db,
    campus: LocationId,
) -> Result<Option<CampusEditor>, DbError> {
    let Some(head) = sqlx::query!(
        r#"SELECT id, name, map_image_asset_id FROM locations WHERE id = $1 AND kind = 'campus'"#,
        campus.as_uuid(),
    )
    .fetch_optional(pool)
    .await
    .map_err(classify)?
    else {
        return Ok(None);
    };

    let buildings = sqlx::query!(
        r#"
        SELECT id, name, marker_x::float8 AS "marker_x?", marker_y::float8 AS "marker_y?"
        FROM locations WHERE parent_id = $1 AND kind = 'building' ORDER BY name
        "#,
        campus.as_uuid(),
    )
    .fetch_all(pool)
    .await
    .map_err(classify)?
    .into_iter()
    .map(|r| BuildingMarker {
        id: r.id,
        name: r.name,
        marker_x: r.marker_x,
        marker_y: r.marker_y,
    })
    .collect();

    Ok(Some(CampusEditor {
        campus_id: head.id,
        name: head.name,
        map_image_asset_id: head.map_image_asset_id,
        buildings,
    }))
}

/// Sets (or clears) a building's marker fraction. The DB `CHECK`s enforce
/// building-only and the 0..1 range.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error (incl. a CHECK violation).
pub async fn update_building_marker(
    pool: &Db,
    building: LocationId,
    marker: Option<(f64, f64)>,
) -> Result<(), DbError> {
    let (x, y) = marker.map_or((None, None), |(x, y)| (Some(x), Some(y)));
    sqlx::query!(
        r#"UPDATE locations SET marker_x = $2::float8::numeric, marker_y = $3::float8::numeric WHERE id = $1"#,
        building.as_uuid(),
        x,
        y,
    )
    .execute(pool)
    .await
    .map_err(classify)?;
    Ok(())
}

/// Sets (or clears) a campus map image. The DB `CHECK` enforces campus-only.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn set_campus_map_image(
    pool: &Db,
    campus: LocationId,
    asset: Option<AssetId>,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE locations SET map_image_asset_id = $2 WHERE id = $1"#,
        campus.as_uuid(),
        asset.map(AssetId::as_uuid),
    )
    .execute(pool)
    .await
    .map_err(classify)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    #[sqlx::test]
    async fn marker_round_trips_and_rejects_non_building(pool: Db) {
        let campus: Uuid = sqlx::query_scalar(
            "INSERT INTO locations (kind, name, path, depth) \
             VALUES ('campus', 'Campus', '/c', 0) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let building: Uuid = sqlx::query_scalar(
            "INSERT INTO locations (kind, name, path, depth, parent_id) \
             VALUES ('building', 'Building A', '/c/a', 1, $1) RETURNING id",
        )
        .bind(campus)
        .fetch_one(&pool)
        .await
        .unwrap();

        update_building_marker(&pool, LocationId::new(building), Some((0.25, 0.75)))
            .await
            .unwrap();
        let editor = load_campus_editor(&pool, LocationId::new(campus))
            .await
            .unwrap()
            .expect("campus exists");
        assert_eq!(editor.buildings.len(), 1);
        let marker = &editor.buildings[0];
        assert!((marker.marker_x.unwrap() - 0.25).abs() < 1e-6);
        assert!((marker.marker_y.unwrap() - 0.75).abs() < 1e-6);

        // The CHECK forbids a marker on a non-building (here, the campus itself).
        assert!(
            update_building_marker(&pool, LocationId::new(campus), Some((0.5, 0.5)))
                .await
                .is_err()
        );
    }
}
