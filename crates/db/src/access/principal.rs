//! Loaders that materialize a principal's authorization facts from the schema:
//! the [`AuthzContext`] the decision runs against, the [`ViewerSegmentation`] the
//! visibility check runs against, and an active booking [`Delegation`].
//!
//! Roles are resolved to [`PermissionSet`]s here (via [`PermissionSet::from_tokens`],
//! which silently drops unknown tokens — fail-closed) so `domain` stays I/O-free.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use domain::authz::{
    AuthzContext, Delegation, GrantSubject, LocationNode, Membership, PermissionSet, RoleGrant,
    ValidityWindow,
};
use domain::segmentation::ViewerSegmentation;
use domain::{LocationId, OrganizationId, TeamId, UserId};

use crate::{Db, DbError};

/// Accumulator for grouping the permission rows of one `role_grants` row.
struct GrantAcc {
    subject: GrantSubject,
    node: LocationNode,
    validity: ValidityWindow,
    tokens: Vec<String>,
}

async fn is_instance_admin(pool: &Db, user: Uuid) -> Result<bool, DbError> {
    let flag = sqlx::query_scalar!(r#"SELECT is_instance_admin FROM users WHERE id = $1"#, user)
        .fetch_optional(pool)
        .await?;
    Ok(flag.unwrap_or(false))
}

/// Loads everything the authorizer needs about `user`: the instance-admin flag,
/// org/team memberships with their resolved permissions, the set of teams the user
/// belongs to, and the location-scoped grants applicable to the user (directly or
/// via a team), pre-filtered to grants not yet expired (`authorize` re-checks the
/// full validity window against the injected clock).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_authz_context(pool: &Db, user: UserId) -> Result<AuthzContext, DbError> {
    let uid = user.as_uuid();
    let is_admin = is_instance_admin(pool, uid).await?;

    let team_ids: HashSet<TeamId> = sqlx::query_scalar!(
        r#"SELECT team_id AS "team_id!" FROM memberships WHERE user_id = $1 AND team_id IS NOT NULL"#,
        uid,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(TeamId::new)
    .collect();

    // Memberships with their role's permission tokens, grouped per (org, team).
    let membership_rows = sqlx::query!(
        r#"
        SELECT m.organization_id, m.team_id, rp.permission
        FROM memberships m
        JOIN role_permissions rp ON rp.role_id = m.role_id
        WHERE m.user_id = $1
        "#,
        uid,
    )
    .fetch_all(pool)
    .await?;
    let mut by_membership: HashMap<(Uuid, Option<Uuid>), Vec<String>> = HashMap::new();
    for row in membership_rows {
        by_membership
            .entry((row.organization_id, row.team_id))
            .or_default()
            .push(row.permission);
    }
    let memberships = by_membership
        .into_iter()
        .map(|((org, team), tokens)| Membership {
            organization: OrganizationId::new(org),
            team: team.map(TeamId::new),
            permissions: PermissionSet::from_tokens(tokens.iter().map(String::as_str)),
        })
        .collect();

    // Location-scoped grants for this user or any of the user's teams, joined to
    // their role's permissions and the grant's location node.
    let team_vec: Vec<Uuid> = team_ids.iter().map(|t| t.as_uuid()).collect();
    let grant_rows = sqlx::query!(
        r#"
        SELECT g.id, g.subject_user_id, g.subject_team_id, g.valid_from, g.valid_to,
               l.id AS location_id, l.path AS location_path, rp.permission
        FROM role_grants g
        JOIN locations l ON l.id = g.location_id
        JOIN role_permissions rp ON rp.role_id = g.role_id
        WHERE (g.subject_user_id = $1 OR g.subject_team_id = ANY($2))
          AND (g.valid_to IS NULL OR g.valid_to > now())
        "#,
        uid,
        &team_vec,
    )
    .fetch_all(pool)
    .await?;

    let mut by_grant: HashMap<Uuid, GrantAcc> = HashMap::new();
    for row in grant_rows {
        let acc = by_grant.entry(row.id).or_insert_with(|| {
            let subject = match (row.subject_user_id, row.subject_team_id) {
                (Some(u), _) => GrantSubject::User(UserId::new(u)),
                (None, Some(t)) => GrantSubject::Team(TeamId::new(t)),
                // The schema's num_nonnulls(...) = 1 check makes this unreachable.
                (None, None) => GrantSubject::User(user),
            };
            GrantAcc {
                subject,
                node: LocationNode {
                    id: LocationId::new(row.location_id),
                    path: row.location_path.clone(),
                },
                validity: ValidityWindow {
                    from: Some(row.valid_from),
                    to: row.valid_to,
                },
                tokens: Vec::new(),
            }
        });
        acc.tokens.push(row.permission);
    }
    let grants = by_grant
        .into_values()
        .map(|acc| RoleGrant {
            subject: acc.subject,
            permissions: PermissionSet::from_tokens(acc.tokens.iter().map(String::as_str)),
            node: acc.node,
            validity: acc.validity,
        })
        .collect();

    Ok(AuthzContext {
        user,
        is_instance_admin: is_admin,
        memberships,
        team_ids,
        grants,
    })
}

/// Loads the viewer's segmentation facts: the instance-admin flag and the sets of
/// organizations and teams the user belongs to.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_viewer_segmentation(
    pool: &Db,
    user: UserId,
) -> Result<ViewerSegmentation, DbError> {
    let uid = user.as_uuid();
    let is_admin = is_instance_admin(pool, uid).await?;
    let orgs = sqlx::query_scalar!(
        r#"SELECT DISTINCT organization_id FROM memberships WHERE user_id = $1"#,
        uid,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(OrganizationId::new)
    .collect();
    let teams = sqlx::query_scalar!(
        r#"SELECT DISTINCT team_id AS "team_id!" FROM memberships WHERE user_id = $1 AND team_id IS NOT NULL"#,
        uid,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(TeamId::new)
    .collect();
    Ok(ViewerSegmentation {
        is_instance_admin: is_admin,
        orgs,
        teams,
    })
}

/// Loads the active booking delegation from `delegate` for `principal` at `now`,
/// with the principal's full [`AuthzContext`] resolved so the caller can apply
/// [`Delegation::as_principal`]. Returns `None` when no delegation is in force.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_active_delegation(
    pool: &Db,
    delegate: UserId,
    principal: UserId,
    now: DateTime<Utc>,
) -> Result<Option<Delegation>, DbError> {
    let row = sqlx::query!(
        r#"
        SELECT valid_from, valid_to
        FROM booking_delegates
        WHERE delegate_user_id = $1 AND principal_user_id = $2
          AND valid_from <= $3 AND (valid_to IS NULL OR valid_to > $3)
        ORDER BY valid_from DESC
        LIMIT 1
        "#,
        delegate.as_uuid(),
        principal.as_uuid(),
        now,
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };
    let principal_ctx = load_authz_context(pool, principal).await?;
    Ok(Some(Delegation {
        delegate,
        principal: principal_ctx,
        window: ValidityWindow {
            from: Some(row.valid_from),
            to: row.valid_to,
        },
    }))
}
