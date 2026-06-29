//! Loaders for resource segmentation facts and the instance mode.

use domain::segmentation::ResourceSegmentation;
use domain::{OrganizationId, ResourceId, SegmentationMode, TeamId};

use crate::access::context;
use crate::{Db, DbError};

/// Persistence-mapped mirror of the `segmentation_mode` enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "segmentation_mode", rename_all = "snake_case")]
pub enum SegmentationModeRow {
    /// All bookable resources are visible to every authenticated user.
    Open,
    /// Visible only to members of the resource's organization.
    ByOrganization,
    /// Visible only to members of the resource's organization and team.
    ByOrganizationAndTeam,
}

impl From<SegmentationModeRow> for SegmentationMode {
    fn from(value: SegmentationModeRow) -> Self {
        match value {
            SegmentationModeRow::Open => Self::Open,
            SegmentationModeRow::ByOrganization => Self::ByOrganization,
            SegmentationModeRow::ByOrganizationAndTeam => Self::ByOrganizationAndTeam,
        }
    }
}

/// The instance-wide segmentation mode. Defaults to [`SegmentationMode::Open`]
/// when no `instance_settings` row exists yet (matching the column default).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_segmentation_mode(pool: &Db) -> Result<SegmentationMode, DbError> {
    let mode = sqlx::query_scalar!(
        r#"SELECT segmentation_mode AS "mode: SegmentationModeRow" FROM instance_settings WHERE id = true"#,
    )
    .fetch_optional(pool)
    .await?;
    Ok(mode
        .map(SegmentationMode::from)
        .unwrap_or(SegmentationMode::Open))
}

/// Loads a resource's org/team bindings at each hierarchy level (resource → zone
/// → location), the input to [`ResourceSegmentation::effective`]. Returns `None`
/// if the resource does not exist.
///
/// Runs under the elevated context so it reads the resource's bindings regardless
/// of the caller's own visibility — it is the authority gathering facts, not a
/// segmentation-filtered read.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_resource_segmentation(
    pool: &Db,
    resource: ResourceId,
) -> Result<Option<ResourceSegmentation>, DbError> {
    let mut tx = pool.begin().await?;
    context::set_system_context(&mut tx).await?;
    let row = sqlx::query!(
        r#"
        SELECT
            r.organization_id  AS res_org,
            r.team_id          AS res_team,
            fz.organization_id AS zone_org,
            fz.team_id         AS zone_team,
            l.organization_id  AS loc_org
        FROM resources r
        LEFT JOIN floor_zones fz ON fz.id = r.floor_zone_id
        JOIN locations l ON l.id = r.location_id
        WHERE r.id = $1
        "#,
        resource.as_uuid(),
    )
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(row.map(|r| ResourceSegmentation {
        resource_org: r.res_org.map(OrganizationId::new),
        resource_team: r.res_team.map(TeamId::new),
        zone_org: r.zone_org.map(OrganizationId::new),
        zone_team: r.zone_team.map(TeamId::new),
        location_org: r.loc_org.map(OrganizationId::new),
    }))
}
