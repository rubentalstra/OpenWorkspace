//! Resolve an action's target to the `domain` [`ManagementTarget`] the authorizer
//! evaluates against: the owning organization (for the org-role path) and the
//! location node with its materialized path (for the location-grant subtree test).

use domain::authz::{LocationNode, ManagementTarget};
use domain::{LocationId, OrganizationId, ResourceId};

use crate::access::context;
use crate::{Db, DbError};

/// Resolves a resource to its management target: the resource's location node and
/// its effective organization (its own, else its location's). Returns `None` if
/// the resource does not exist.
///
/// Runs under the elevated context so it resolves the target regardless of the
/// actor's visibility — authorization must see the target to decide on it.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_resource_target(
    pool: &Db,
    resource: ResourceId,
) -> Result<Option<ManagementTarget>, DbError> {
    let mut tx = pool.begin().await?;
    context::set_system_context(&mut tx).await?;
    let row = sqlx::query!(
        r#"
        SELECT l.id AS location_id, l.path AS location_path,
               COALESCE(r.organization_id, l.organization_id) AS org
        FROM resources r
        JOIN locations l ON l.id = r.location_id
        WHERE r.id = $1
        "#,
        resource.as_uuid(),
    )
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(row.map(|r| ManagementTarget {
        location: Some(LocationNode {
            id: LocationId::new(r.location_id),
            path: r.location_path,
        }),
        organization: r.org.map(OrganizationId::new),
    }))
}

/// Resolves a location to its management target (the node itself and the org it
/// belongs to). Returns `None` if the location does not exist.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_location_target(
    pool: &Db,
    location: LocationId,
) -> Result<Option<ManagementTarget>, DbError> {
    let row = sqlx::query!(
        r#"SELECT id AS location_id, path AS location_path, organization_id AS org FROM locations WHERE id = $1"#,
        location.as_uuid(),
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| ManagementTarget {
        location: Some(LocationNode {
            id: LocationId::new(r.location_id),
            path: r.location_path,
        }),
        organization: r.org.map(OrganizationId::new),
    }))
}
