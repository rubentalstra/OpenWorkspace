//! RLS connection-context helpers.
//!
//! The `resources` row-security policies (see the `p8_access_enforcement`
//! migration) read transaction-local GUCs via `current_setting('app.…', true)`.
//! These functions set them with `set_config('app.…', …, true)` (the trailing
//! `true` scopes the value to the current transaction, so it resets on commit or
//! rollback). Run them on a `pool.begin()` transaction, then issue the queries on
//! the same transaction.
//!
//! [`set_viewer_context`] scopes reads to one viewer; [`set_system_context`] grants
//! the trusted/elevated access an authorized write (or an authority fact-load like
//! [`super::segmentation::load_resource_segmentation`]) needs.

use sqlx::PgConnection;

use domain::SegmentationMode;
use domain::segmentation::ViewerSegmentation;

use crate::DbError;

fn mode_token(mode: SegmentationMode) -> &'static str {
    match mode {
        SegmentationMode::Open => "open",
        SegmentationMode::ByOrganization => "by_organization",
        SegmentationMode::ByOrganizationAndTeam => "by_organization_and_team",
    }
}

fn csv_of(ids: impl Iterator<Item = uuid::Uuid>) -> String {
    let mut out = String::new();
    for id in ids {
        if !out.is_empty() {
            out.push(',');
        }
        out.push_str(&id.to_string());
    }
    out
}

/// Sets the transaction-local viewer context the RLS policies evaluate against:
/// the viewer's instance-admin flag, the active segmentation mode, and the
/// comma-separated org/team id lists. Clears any elevated `app.bypass` first.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn set_viewer_context(
    conn: &mut PgConnection,
    viewer: &ViewerSegmentation,
    mode: SegmentationMode,
) -> Result<(), DbError> {
    let org_csv = csv_of(viewer.orgs.iter().map(|o| o.as_uuid()));
    let team_csv = csv_of(viewer.teams.iter().map(|t| t.as_uuid()));
    sqlx::query(
        "SELECT set_config('app.bypass', 'false', true), \
                set_config('app.is_instance_admin', $1, true), \
                set_config('app.segmentation_mode', $2, true), \
                set_config('app.org_ids', $3, true), \
                set_config('app.team_ids', $4, true)",
    )
    .bind(viewer.is_instance_admin.to_string())
    .bind(mode_token(mode))
    .bind(org_csv)
    .bind(team_csv)
    .execute(conn)
    .await?;
    Ok(())
}

/// Sets the transaction-local elevated context: every RLS-protected row is visible
/// and writable for the duration of the transaction. Use for trusted system work
/// (authority fact-loads, and authorized writes once the `AuthzBackend` has
/// allowed them). Not a security boundary against the runtime role itself — the
/// hard boundaries are the table grants and the `audit_log` REVOKE.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn set_system_context(conn: &mut PgConnection) -> Result<(), DbError> {
    sqlx::query("SELECT set_config('app.bypass', 'true', true)")
        .execute(conn)
        .await?;
    Ok(())
}
