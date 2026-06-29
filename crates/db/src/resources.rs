//! Resource persistence for the floor builder: the `resources` row, its
//! `resource_rules`, its `resource_equipment` assignment, and the
//! `resource_positions` binding to a scene node. The upsert/position helpers take a
//! `&mut PgConnection` so the floor-plan save composes them in one transaction.

use domain::{EquipmentItemId, LocationId, ResourceId, ResourceKind, ResourceStatus};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{Db, DbError, classify};

/// Persistence mirror of the `resource_kind` enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "resource_kind", rename_all = "snake_case")]
pub enum ResourceKindRow {
    Desk,
    Room,
    Parking,
    Vehicle,
    Equipment,
}

impl From<ResourceKind> for ResourceKindRow {
    fn from(kind: ResourceKind) -> Self {
        match kind {
            ResourceKind::Desk => Self::Desk,
            ResourceKind::Room => Self::Room,
            ResourceKind::Parking => Self::Parking,
            ResourceKind::Vehicle => Self::Vehicle,
            ResourceKind::Equipment => Self::Equipment,
        }
    }
}

/// Persistence mirror of the `resource_status` enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "resource_status", rename_all = "snake_case")]
pub enum ResourceStatusRow {
    Active,
    Inactive,
    Maintenance,
}

impl From<ResourceStatus> for ResourceStatusRow {
    fn from(status: ResourceStatus) -> Self {
        match status {
            ResourceStatus::Active => Self::Active,
            ResourceStatus::Inactive => Self::Inactive,
            ResourceStatus::Maintenance => Self::Maintenance,
        }
    }
}

/// Booking-policy fields for a resource (the `resource_rules` row). All limits are
/// optional; absent means "no limit".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceRulesSpec {
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

/// A resource the builder creates or updates for a bound bookable node.
#[derive(Clone, Debug)]
pub struct ResourceSpec {
    /// `None` creates a new resource; `Some` updates it.
    pub resource_id: Option<ResourceId>,
    pub scene_node_id: String,
    pub kind: ResourceKindRow,
    pub name: String,
    pub code: Option<String>,
    pub category_id: Option<Uuid>,
    pub capacity: Option<i32>,
    pub bookable: bool,
    pub requires_checkin: bool,
    pub is_accessible: bool,
    pub description: Option<String>,
    pub status: ResourceStatusRow,
    pub rules: ResourceRulesSpec,
    /// Equipment-catalog assignment: `(item, quantity)`.
    pub equipment: Vec<(EquipmentItemId, i32)>,
}

/// A loaded resource (+ rules + equipment) bound on a floor.
#[derive(Clone, Debug)]
pub struct ResourceRow {
    pub id: Uuid,
    pub scene_node_id: String,
    pub kind: ResourceKindRow,
    pub name: String,
    pub code: Option<String>,
    pub category_id: Option<Uuid>,
    pub capacity: Option<i32>,
    pub bookable: bool,
    pub requires_checkin: bool,
    pub is_accessible: bool,
    pub description: Option<String>,
    pub status: ResourceStatusRow,
    pub rules: ResourceRulesSpec,
    pub equipment: Vec<(Uuid, i32)>,
}

/// Upserts a resource + its rules + its equipment within a transaction, and returns
/// its id. Does not touch `resource_positions` (the caller binds the node).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub(crate) async fn upsert_resource(
    conn: &mut PgConnection,
    floor: LocationId,
    spec: &ResourceSpec,
) -> Result<ResourceId, DbError> {
    let id = if let Some(existing) = spec.resource_id {
        sqlx::query!(
            r#"
            UPDATE resources
            SET kind = $2, name = $3, code = $4, category_id = $5, capacity = $6,
                bookable = $7, requires_checkin = $8, is_accessible = $9,
                description = $10, status = $11
            WHERE id = $1
            "#,
            existing.as_uuid(),
            spec.kind as _,
            spec.name,
            spec.code,
            spec.category_id,
            spec.capacity,
            spec.bookable,
            spec.requires_checkin,
            spec.is_accessible,
            spec.description,
            spec.status as _,
        )
        .execute(&mut *conn)
        .await
        .map_err(classify)?;
        existing
    } else {
        let new = sqlx::query_scalar!(
            r#"
            INSERT INTO resources
                (location_id, kind, name, code, category_id, capacity, bookable,
                 requires_checkin, is_accessible, description, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#,
            floor.as_uuid(),
            spec.kind as _,
            spec.name,
            spec.code,
            spec.category_id,
            spec.capacity,
            spec.bookable,
            spec.requires_checkin,
            spec.is_accessible,
            spec.description,
            spec.status as _,
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(classify)?;
        ResourceId::new(new)
    };

    upsert_rules(&mut *conn, id, &spec.rules).await?;
    replace_equipment(&mut *conn, id, &spec.equipment).await?;
    Ok(id)
}

async fn upsert_rules(
    conn: &mut PgConnection,
    resource: ResourceId,
    rules: &ResourceRulesSpec,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        INSERT INTO resource_rules
            (resource_id, max_advance_days, min_advance_minutes, min_duration_minutes,
             max_duration_minutes, slot_granularity_minutes, buffer_minutes,
             max_per_user_per_day, max_active_per_user, cancellation_deadline_minutes,
             allow_recurrence, max_recurrence_count, max_recurrence_horizon_days,
             require_approval)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (resource_id) DO UPDATE SET
            max_advance_days = EXCLUDED.max_advance_days,
            min_advance_minutes = EXCLUDED.min_advance_minutes,
            min_duration_minutes = EXCLUDED.min_duration_minutes,
            max_duration_minutes = EXCLUDED.max_duration_minutes,
            slot_granularity_minutes = EXCLUDED.slot_granularity_minutes,
            buffer_minutes = EXCLUDED.buffer_minutes,
            max_per_user_per_day = EXCLUDED.max_per_user_per_day,
            max_active_per_user = EXCLUDED.max_active_per_user,
            cancellation_deadline_minutes = EXCLUDED.cancellation_deadline_minutes,
            allow_recurrence = EXCLUDED.allow_recurrence,
            max_recurrence_count = EXCLUDED.max_recurrence_count,
            max_recurrence_horizon_days = EXCLUDED.max_recurrence_horizon_days,
            require_approval = EXCLUDED.require_approval
        "#,
        resource.as_uuid(),
        rules.max_advance_days,
        rules.min_advance_minutes,
        rules.min_duration_minutes,
        rules.max_duration_minutes,
        rules.slot_granularity_minutes,
        rules.buffer_minutes,
        rules.max_per_user_per_day,
        rules.max_active_per_user,
        rules.cancellation_deadline_minutes,
        rules.allow_recurrence,
        rules.max_recurrence_count,
        rules.max_recurrence_horizon_days,
        rules.require_approval,
    )
    .execute(&mut *conn)
    .await
    .map_err(classify)?;
    Ok(())
}

async fn replace_equipment(
    conn: &mut PgConnection,
    resource: ResourceId,
    equipment: &[(EquipmentItemId, i32)],
) -> Result<(), DbError> {
    sqlx::query!(
        r#"DELETE FROM resource_equipment WHERE resource_id = $1"#,
        resource.as_uuid(),
    )
    .execute(&mut *conn)
    .await
    .map_err(classify)?;

    for (item, quantity) in equipment {
        sqlx::query!(
            r#"
            INSERT INTO resource_equipment (resource_id, equipment_item_id, quantity)
            VALUES ($1, $2, $3)
            "#,
            resource.as_uuid(),
            item.as_uuid(),
            quantity,
        )
        .execute(&mut *conn)
        .await
        .map_err(classify)?;
    }
    Ok(())
}

/// Binds a resource to a scene node (idempotent on the `(resource_id)` and
/// `(floor_id, scene_node_id)` uniqueness — re-binding moves the node).
pub(crate) async fn upsert_position(
    conn: &mut PgConnection,
    resource: ResourceId,
    floor: LocationId,
    scene_node_id: &str,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        INSERT INTO resource_positions (resource_id, floor_id, scene_node_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (resource_id) DO UPDATE
            SET floor_id = EXCLUDED.floor_id, scene_node_id = EXCLUDED.scene_node_id
        "#,
        resource.as_uuid(),
        floor.as_uuid(),
        scene_node_id,
    )
    .execute(&mut *conn)
    .await
    .map_err(classify)?;
    Ok(())
}

/// Deletes the floor's `resource_positions` whose `scene_node_id` is not among the
/// still-bound nodes (unbinds removed seats; the resource entity is kept).
pub(crate) async fn delete_stale_positions(
    conn: &mut PgConnection,
    floor: LocationId,
    keep_nodes: &[String],
) -> Result<(), DbError> {
    sqlx::query!(
        r#"DELETE FROM resource_positions WHERE floor_id = $1 AND scene_node_id <> ALL($2)"#,
        floor.as_uuid(),
        keep_nodes,
    )
    .execute(&mut *conn)
    .await
    .map_err(classify)?;
    Ok(())
}

/// Loads the resources bound on a floor (+ rules + equipment) so the builder can
/// hydrate an existing plan.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn list_resources(pool: &Db, floor: LocationId) -> Result<Vec<ResourceRow>, DbError> {
    let rows = sqlx::query!(
        r#"
        SELECT r.id, p.scene_node_id, r.kind AS "kind: ResourceKindRow", r.name, r.code,
               r.category_id, r.capacity, r.bookable, r.requires_checkin, r.is_accessible,
               r.description, r.status AS "status: ResourceStatusRow",
               rr.max_advance_days, rr.min_advance_minutes, rr.min_duration_minutes,
               rr.max_duration_minutes, rr.slot_granularity_minutes, rr.buffer_minutes,
               rr.max_per_user_per_day, rr.max_active_per_user, rr.cancellation_deadline_minutes,
               rr.allow_recurrence AS "allow_recurrence?", rr.max_recurrence_count,
               rr.max_recurrence_horizon_days, rr.require_approval AS "require_approval?"
        FROM resource_positions p
        JOIN resources r ON r.id = p.resource_id
        LEFT JOIN resource_rules rr ON rr.resource_id = r.id
        WHERE p.floor_id = $1
        ORDER BY r.name
        "#,
        floor.as_uuid(),
    )
    .fetch_all(pool)
    .await
    .map_err(classify)?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let equipment = sqlx::query!(
            r#"SELECT equipment_item_id, quantity FROM resource_equipment WHERE resource_id = $1"#,
            row.id,
        )
        .fetch_all(pool)
        .await
        .map_err(classify)?
        .into_iter()
        .map(|e| (e.equipment_item_id, e.quantity))
        .collect();

        out.push(ResourceRow {
            id: row.id,
            scene_node_id: row.scene_node_id,
            kind: row.kind,
            name: row.name,
            code: row.code,
            category_id: row.category_id,
            capacity: row.capacity,
            bookable: row.bookable,
            requires_checkin: row.requires_checkin,
            is_accessible: row.is_accessible,
            description: row.description,
            status: row.status,
            rules: ResourceRulesSpec {
                max_advance_days: row.max_advance_days,
                min_advance_minutes: row.min_advance_minutes,
                min_duration_minutes: row.min_duration_minutes,
                max_duration_minutes: row.max_duration_minutes,
                slot_granularity_minutes: row.slot_granularity_minutes,
                buffer_minutes: row.buffer_minutes,
                max_per_user_per_day: row.max_per_user_per_day,
                max_active_per_user: row.max_active_per_user,
                cancellation_deadline_minutes: row.cancellation_deadline_minutes,
                allow_recurrence: row.allow_recurrence.unwrap_or(true),
                max_recurrence_count: row.max_recurrence_count,
                max_recurrence_horizon_days: row.max_recurrence_horizon_days,
                require_approval: row.require_approval.unwrap_or(false),
            },
            equipment,
        });
    }
    Ok(out)
}
