//! P8 access-enforcement use-case matrix, end to end against real PostgreSQL.
//!
//! One integration crate (helpers are private fns used across the tests) so there
//! is no shared-`pub` test module to trip `unreachable_pub`/`dead_code`. Gated on
//! `ssr`, which the workspace build activates via the server's `auth/ssr` dep; run
//! standalone with `cargo nextest run -p auth --features ssr`.
//!
//! Setup runs as the test pool's owner/superuser (bypasses RLS); the `rls_*`
//! probes switch to the least-privilege `openworkspace_app` role with `SET LOCAL
//! ROLE` to exercise the policies and the audit-log REVOKE for real.
#![cfg(feature = "ssr")]
#![expect(
    clippy::tests_outside_test_module,
    reason = "integration-test crate: every item is a test or its fixture, so a nested test module adds no separation"
)]
#![expect(
    clippy::unwrap_used,
    reason = "fixture helpers unwrap setup queries; a failure is a test failure (clippy's allow-unwrap-in-tests covers only #[test] fns)"
)]

use std::collections::HashSet;

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use auth::{AuthzBackend, AuthzError, Target};
use domain::authz::Action;
use domain::segmentation::{ViewerSegmentation, visible};
use domain::{
    FloorZoneId, LocationId, OrganizationId, ResourceId, SegmentationMode, TeamId, UserId,
};

// --- fixtures ----------------------------------------------------------------

async fn seed_user(pool: &PgPool, email: &str, is_instance_admin: bool) -> UserId {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, display_name, webauthn_user_handle, is_instance_admin) \
         VALUES ($1::citext, $1, $2, $3) RETURNING id",
    )
    .bind(email)
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .bind(is_instance_admin)
    .fetch_one(pool)
    .await
    .unwrap();
    UserId::new(id)
}

async fn seed_org(pool: &PgPool, slug: &str) -> OrganizationId {
    let id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name, slug) VALUES ($1, $1) RETURNING id")
            .bind(slug)
            .fetch_one(pool)
            .await
            .unwrap();
    OrganizationId::new(id)
}

async fn seed_team(pool: &PgPool, org: OrganizationId, name: &str) -> TeamId {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO teams (organization_id, name) VALUES ($1, $2) RETURNING id",
    )
    .bind(org.as_uuid())
    .bind(name)
    .fetch_one(pool)
    .await
    .unwrap();
    TeamId::new(id)
}

async fn seed_location(
    pool: &PgPool,
    kind: &str,
    path: &str,
    depth: i32,
    org: Option<OrganizationId>,
) -> LocationId {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO locations (kind, name, path, depth, organization_id) \
         VALUES ($1::location_kind, $2, $2, $3, $4) RETURNING id",
    )
    .bind(kind)
    .bind(path)
    .bind(depth)
    .bind(org.map(OrganizationId::as_uuid))
    .fetch_one(pool)
    .await
    .unwrap();
    LocationId::new(id)
}

async fn seed_floor_zone(
    pool: &PgPool,
    floor: LocationId,
    org: Option<OrganizationId>,
    team: Option<TeamId>,
) -> FloorZoneId {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO floor_zones (floor_id, name, organization_id, team_id) \
         VALUES ($1, 'Zone', $2, $3) RETURNING id",
    )
    .bind(floor.as_uuid())
    .bind(org.map(OrganizationId::as_uuid))
    .bind(team.map(TeamId::as_uuid))
    .fetch_one(pool)
    .await
    .unwrap();
    FloorZoneId::new(id)
}

async fn seed_resource(
    pool: &PgPool,
    location: LocationId,
    org: Option<OrganizationId>,
    team: Option<TeamId>,
    zone: Option<FloorZoneId>,
) -> ResourceId {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO resources (location_id, kind, name, organization_id, team_id, floor_zone_id) \
         VALUES ($1, 'desk', 'Desk', $2, $3, $4) RETURNING id",
    )
    .bind(location.as_uuid())
    .bind(org.map(OrganizationId::as_uuid))
    .bind(team.map(TeamId::as_uuid))
    .bind(zone.map(FloorZoneId::as_uuid))
    .fetch_one(pool)
    .await
    .unwrap();
    ResourceId::new(id)
}

async fn role_id(pool: &PgPool, key: &str) -> Uuid {
    sqlx::query_scalar("SELECT id FROM roles WHERE key = $1::citext")
        .bind(key)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn add_membership(
    pool: &PgPool,
    user: UserId,
    org: OrganizationId,
    team: Option<TeamId>,
    role_key: &str,
) {
    let role = role_id(pool, role_key).await;
    sqlx::query(
        "INSERT INTO memberships (user_id, organization_id, team_id, role_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(user.as_uuid())
    .bind(org.as_uuid())
    .bind(team.map(TeamId::as_uuid))
    .bind(role)
    .execute(pool)
    .await
    .unwrap();
}

async fn add_user_grant(
    pool: &PgPool,
    user: UserId,
    role_key: &str,
    location: LocationId,
    valid_from: DateTime<Utc>,
    valid_to: Option<DateTime<Utc>>,
) {
    let role = role_id(pool, role_key).await;
    sqlx::query(
        "INSERT INTO role_grants (subject_user_id, role_id, location_id, valid_from, valid_to) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user.as_uuid())
    .bind(role)
    .bind(location.as_uuid())
    .bind(valid_from)
    .bind(valid_to)
    .execute(pool)
    .await
    .unwrap();
}

async fn add_team_grant(
    pool: &PgPool,
    team: TeamId,
    role_key: &str,
    location: LocationId,
    valid_from: DateTime<Utc>,
    valid_to: Option<DateTime<Utc>>,
) {
    let role = role_id(pool, role_key).await;
    sqlx::query(
        "INSERT INTO role_grants (subject_team_id, role_id, location_id, valid_from, valid_to) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(team.as_uuid())
    .bind(role)
    .bind(location.as_uuid())
    .bind(valid_from)
    .bind(valid_to)
    .execute(pool)
    .await
    .unwrap();
}

async fn add_delegate(
    pool: &PgPool,
    principal: UserId,
    delegate: UserId,
    valid_from: DateTime<Utc>,
    valid_to: Option<DateTime<Utc>>,
) {
    sqlx::query(
        "INSERT INTO booking_delegates (principal_user_id, delegate_user_id, valid_from, valid_to) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(principal.as_uuid())
    .bind(delegate.as_uuid())
    .bind(valid_from)
    .bind(valid_to)
    .execute(pool)
    .await
    .unwrap();
}

async fn set_segmentation_mode(pool: &PgPool, mode: &str) {
    sqlx::query(
        "INSERT INTO instance_settings (id, segmentation_mode) VALUES (true, $1::segmentation_mode) \
         ON CONFLICT (id) DO UPDATE SET segmentation_mode = EXCLUDED.segmentation_mode",
    )
    .bind(mode)
    .execute(pool)
    .await
    .unwrap();
}

async fn count_audit(pool: &PgPool, action: &str, outcome: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM audit_log WHERE action = $1 AND outcome = $2::audit_outcome",
    )
    .bind(action)
    .bind(outcome)
    .fetch_one(pool)
    .await
    .unwrap()
}

fn viewer(is_admin: bool, orgs: &[OrganizationId], teams: &[TeamId]) -> ViewerSegmentation {
    ViewerSegmentation {
        is_instance_admin: is_admin,
        orgs: orgs.iter().copied().collect(),
        teams: teams.iter().copied().collect(),
    }
}

/// The RLS context to apply before probing visibility as the runtime role.
enum Ctx<'a> {
    None,
    System,
    Viewer(&'a ViewerSegmentation, SegmentationMode),
}

/// Whether `resource` is visible to a `SELECT` issued as `openworkspace_app` under
/// `ctx`. Rolls back so neither the role switch nor the GUCs leak on the pooled
/// connection.
async fn rls_exists_as_runtime(pool: &PgPool, resource: ResourceId, ctx: Ctx<'_>) -> bool {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE openworkspace_app")
        .execute(&mut *tx)
        .await
        .unwrap();
    match ctx {
        Ctx::None => {}
        Ctx::System => db::set_system_context(&mut tx).await.unwrap(),
        Ctx::Viewer(v, mode) => db::set_viewer_context(&mut tx, v, mode).await.unwrap(),
    }
    let visible: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM resources WHERE id = $1)")
        .bind(resource.as_uuid())
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    visible
}

fn denied(result: Result<(), AuthzError>) -> bool {
    result.is_err_and(|e| matches!(e, AuthzError::Denied))
}

// --- RBAC --------------------------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn instance_admin_allowed_on_every_target(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let admin = seed_user(&pool, "admin@e.test", true).await;
    let backend = AuthzBackend::new(pool.clone());

    for (action, target) in [
        (Action::BookingCreate, Target::Resource(resource)),
        (Action::ResourceManage, Target::Resource(resource)),
        (Action::FloorBuild, Target::Location(floor)),
        (Action::OrgManage, Target::Organization(org)),
        (Action::InstanceConfigure, Target::Instance),
    ] {
        assert!(
            backend.authorize(admin, action, target, None).await.is_ok(),
            "instance admin must be allowed: {action:?} on {target:?}"
        );
    }
}

#[sqlx::test(migrations = "../db/migrations")]
async fn member_can_book_but_not_manage(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let user = seed_user(&pool, "member@e.test", false).await;
    add_membership(&pool, user, org, None, "member").await;
    let backend = AuthzBackend::new(pool.clone());

    assert!(
        backend
            .authorize(
                user,
                Action::BookingCreate,
                Target::Resource(resource),
                None
            )
            .await
            .is_ok()
    );
    assert!(denied(
        backend
            .authorize(
                user,
                Action::ResourceManage,
                Target::Resource(resource),
                None
            )
            .await
    ));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn admin_and_owner_tiers(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let admin = seed_user(&pool, "admin@e.test", false).await;
    add_membership(&pool, admin, org, None, "admin").await;
    let owner = seed_user(&pool, "owner@e.test", false).await;
    add_membership(&pool, owner, org, None, "owner").await;
    let backend = AuthzBackend::new(pool.clone());

    for action in [
        Action::ResourceManage,
        Action::FloorBuild,
        Action::BookingManageAny,
    ] {
        assert!(
            backend
                .authorize(admin, action, Target::Resource(resource), None)
                .await
                .is_ok()
        );
    }
    // Governance is owner-only; admin is denied, owner allowed.
    for action in [Action::RoleManage, Action::OrgManage, Action::GdprManage] {
        assert!(denied(
            backend
                .authorize(admin, action, Target::Organization(org), None)
                .await
        ));
        assert!(
            backend
                .authorize(owner, action, Target::Organization(org), None)
                .await
                .is_ok()
        );
    }
    // InstanceConfigure is the instance-admin flag only — neither tier holds it.
    assert!(denied(
        backend
            .authorize(owner, Action::InstanceConfigure, Target::Instance, None)
            .await
    ));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn org_role_does_not_reach_across_orgs(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org_a = seed_org(&pool, "alpha").await;
    let org_b = seed_org(&pool, "beta").await;
    let floor_b = seed_location(&pool, "floor", "/beta/f1", 1, Some(org_b)).await;
    let resource_b = seed_resource(&pool, floor_b, Some(org_b), None, None).await;
    let user = seed_user(&pool, "a@e.test", false).await;
    add_membership(&pool, user, org_a, None, "admin").await;
    let backend = AuthzBackend::new(pool.clone());

    assert!(denied(
        backend
            .authorize(
                user,
                Action::BookingCreate,
                Target::Resource(resource_b),
                None
            )
            .await
    ));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn target_without_org_denies_org_role(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/orphan/f1", 1, None).await;
    let resource = seed_resource(&pool, floor, None, None, None).await;
    let user = seed_user(&pool, "m@e.test", false).await;
    add_membership(&pool, user, org, None, "member").await;
    let backend = AuthzBackend::new(pool.clone());

    assert!(denied(
        backend
            .authorize(
                user,
                Action::BookingCreate,
                Target::Resource(resource),
                None
            )
            .await
    ));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn unknown_permission_token_confers_nothing(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let user = seed_user(&pool, "x@e.test", false).await;
    let role: Uuid = sqlx::query_scalar(
        "INSERT INTO roles (key, name) VALUES ('custom'::citext, 'Custom') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO role_permissions (role_id, permission) VALUES ($1, 'does.not.exist')")
        .bind(role)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO memberships (user_id, organization_id, role_id) VALUES ($1, $2, $3)")
        .bind(user.as_uuid())
        .bind(org.as_uuid())
        .bind(role)
        .execute(&pool)
        .await
        .unwrap();
    let backend = AuthzBackend::new(pool.clone());

    assert!(denied(
        backend
            .authorize(
                user,
                Action::BookingCreate,
                Target::Resource(resource),
                None
            )
            .await
    ));
}

// --- location-scoped grants --------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn grant_covers_subtree_not_siblings(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let b1 = seed_location(&pool, "building", "/acme/b1", 1, Some(org)).await;
    let f1 = seed_location(&pool, "floor", "/acme/b1/f1", 2, Some(org)).await;
    // Sibling whose path is a textual prefix sibling of /acme/b1 — the classic
    // /acme/b1 vs /acme/b10 collision.
    let b10 = seed_location(&pool, "building", "/acme/b10", 1, Some(org)).await;
    let f10 = seed_location(&pool, "floor", "/acme/b10/f1", 2, Some(org)).await;
    let _ = b10;
    let inside = seed_resource(&pool, f1, Some(org), None, None).await;
    let sibling = seed_resource(&pool, f10, Some(org), None, None).await;

    let user = seed_user(&pool, "facilities@e.test", false).await;
    add_user_grant(
        &pool,
        user,
        "admin",
        b1,
        Utc::now() - Duration::days(1),
        None,
    )
    .await;
    let backend = AuthzBackend::new(pool.clone());

    assert!(
        backend
            .authorize(user, Action::FloorBuild, Target::Resource(inside), None)
            .await
            .is_ok(),
        "grant on b1 covers its subtree"
    );
    assert!(
        denied(
            backend
                .authorize(user, Action::FloorBuild, Target::Resource(sibling), None)
                .await
        ),
        "grant on b1 must NOT cover the sibling b10"
    );
}

#[sqlx::test(migrations = "../db/migrations")]
async fn grant_validity_window(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let backend = AuthzBackend::new(pool.clone());
    let now = Utc::now();

    let active = seed_user(&pool, "active@e.test", false).await;
    add_user_grant(
        &pool,
        active,
        "admin",
        floor,
        now - Duration::days(1),
        Some(now + Duration::days(1)),
    )
    .await;
    let expired = seed_user(&pool, "expired@e.test", false).await;
    add_user_grant(
        &pool,
        expired,
        "admin",
        floor,
        now - Duration::days(2),
        Some(now - Duration::days(1)),
    )
    .await;
    let future = seed_user(&pool, "future@e.test", false).await;
    add_user_grant(&pool, future, "admin", floor, now + Duration::days(1), None).await;

    assert!(
        backend
            .authorize(active, Action::FloorBuild, Target::Resource(resource), None)
            .await
            .is_ok()
    );
    assert!(denied(
        backend
            .authorize(
                expired,
                Action::FloorBuild,
                Target::Resource(resource),
                None
            )
            .await
    ));
    assert!(denied(
        backend
            .authorize(future, Action::FloorBuild, Target::Resource(resource), None)
            .await
    ));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn grant_never_confers_governance(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let user = seed_user(&pool, "scoped@e.test", false).await;
    // Owner role (which lists governance actions) granted at a location.
    add_user_grant(
        &pool,
        user,
        "owner",
        floor,
        Utc::now() - Duration::days(1),
        None,
    )
    .await;
    let backend = AuthzBackend::new(pool.clone());

    // Delegable action via the grant: allowed.
    assert!(
        backend
            .authorize(user, Action::FloorBuild, Target::Resource(resource), None)
            .await
            .is_ok()
    );
    // Governance is never grant-delegable, and there is no org membership.
    assert!(denied(
        backend
            .authorize(user, Action::RoleManage, Target::Organization(org), None)
            .await
    ));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn team_subject_grant(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let team = seed_team(&pool, org, "facilities").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    add_team_grant(
        &pool,
        team,
        "admin",
        floor,
        Utc::now() - Duration::days(1),
        None,
    )
    .await;

    let member = seed_user(&pool, "in-team@e.test", false).await;
    add_membership(&pool, member, org, Some(team), "member").await;
    let outsider = seed_user(&pool, "no-team@e.test", false).await;
    add_membership(&pool, outsider, org, None, "member").await;
    let backend = AuthzBackend::new(pool.clone());

    assert!(
        backend
            .authorize(member, Action::FloorBuild, Target::Resource(resource), None)
            .await
            .is_ok()
    );
    assert!(denied(
        backend
            .authorize(
                outsider,
                Action::FloorBuild,
                Target::Resource(resource),
                None
            )
            .await
    ));
}

// --- delegation --------------------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn delegate_acts_as_principal_clamped_to_booking(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let principal = seed_user(&pool, "boss@e.test", false).await;
    add_membership(&pool, principal, org, None, "admin").await; // has BookingManageAny + ResourceManage
    let delegate = seed_user(&pool, "assistant@e.test", false).await;
    add_delegate(
        &pool,
        principal,
        delegate,
        Utc::now() - Duration::days(1),
        None,
    )
    .await;
    let backend = AuthzBackend::new(pool.clone());

    // Booking-family power borrowed from the principal: allowed.
    assert!(
        backend
            .authorize(
                delegate,
                Action::BookingManageAny,
                Target::Resource(resource),
                Some(principal)
            )
            .await
            .is_ok()
    );
    // Non-booking authority is clamped away even though the principal has it.
    assert!(denied(
        backend
            .authorize(
                delegate,
                Action::ResourceManage,
                Target::Resource(resource),
                Some(principal)
            )
            .await
    ));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn delegation_absent_inactive_or_wrong_delegate_denied(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let principal = seed_user(&pool, "boss@e.test", false).await;
    add_membership(&pool, principal, org, None, "member").await;
    let delegate = seed_user(&pool, "assistant@e.test", false).await;
    let stranger = seed_user(&pool, "stranger@e.test", false).await;
    let backend = AuthzBackend::new(pool.clone());

    // No delegation at all.
    assert!(denied(
        backend
            .authorize(
                delegate,
                Action::BookingCreate,
                Target::Resource(resource),
                Some(principal)
            )
            .await
    ));

    // Expired delegation.
    add_delegate(
        &pool,
        principal,
        delegate,
        Utc::now() - Duration::days(2),
        Some(Utc::now() - Duration::days(1)),
    )
    .await;
    assert!(denied(
        backend
            .authorize(
                delegate,
                Action::BookingCreate,
                Target::Resource(resource),
                Some(principal)
            )
            .await
    ));

    // Active delegation, but a different user tries to use it.
    add_delegate(
        &pool,
        principal,
        delegate,
        Utc::now() - Duration::hours(1),
        None,
    )
    .await;
    assert!(denied(
        backend
            .authorize(
                stranger,
                Action::BookingCreate,
                Target::Resource(resource),
                Some(principal)
            )
            .await
    ));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn delegated_action_audits_actor_and_principal(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let principal = seed_user(&pool, "boss@e.test", false).await;
    add_membership(&pool, principal, org, None, "member").await;
    let delegate = seed_user(&pool, "assistant@e.test", false).await;
    add_delegate(
        &pool,
        principal,
        delegate,
        Utc::now() - Duration::days(1),
        None,
    )
    .await;
    let backend = AuthzBackend::new(pool.clone());

    backend
        .authorize(
            delegate,
            Action::BookingCreate,
            Target::Resource(resource),
            Some(principal),
        )
        .await
        .unwrap();

    let (actor, on_behalf): (Uuid, Uuid) = sqlx::query_as(
        "SELECT actor_user_id, on_behalf_of_user_id FROM audit_log \
         WHERE action = 'booking.create' AND outcome = 'success'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actor, delegate.as_uuid());
    assert_eq!(on_behalf, principal.as_uuid());
}

// --- segmentation visibility (app-layer visible_resource) --------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn segmentation_open_shows_everything(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    set_segmentation_mode(&pool, "open").await;
    let org_a = seed_org(&pool, "alpha").await;
    let org_b = seed_org(&pool, "beta").await;
    let floor_b = seed_location(&pool, "floor", "/beta/f1", 1, Some(org_b)).await;
    let resource_b = seed_resource(&pool, floor_b, Some(org_b), None, None).await;
    let user = seed_user(&pool, "a@e.test", false).await;
    add_membership(&pool, user, org_a, None, "member").await;
    let backend = AuthzBackend::new(pool.clone());

    assert!(backend.visible_resource(user, resource_b).await.unwrap());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn segmentation_by_organization(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    set_segmentation_mode(&pool, "by_organization").await;
    let org_a = seed_org(&pool, "alpha").await;
    let org_b = seed_org(&pool, "beta").await;
    let floor_a = seed_location(&pool, "floor", "/alpha/f1", 1, Some(org_a)).await;
    let in_a = seed_resource(&pool, floor_a, Some(org_a), None, None).await;
    let orphan_floor = seed_location(&pool, "floor", "/none/f1", 1, None).await;
    let orphan = seed_resource(&pool, orphan_floor, None, None, None).await;

    let member_a = seed_user(&pool, "a@e.test", false).await;
    add_membership(&pool, member_a, org_a, None, "member").await;
    let member_b = seed_user(&pool, "b@e.test", false).await;
    add_membership(&pool, member_b, org_b, None, "member").await;
    let backend = AuthzBackend::new(pool.clone());

    assert!(backend.visible_resource(member_a, in_a).await.unwrap());
    assert!(!backend.visible_resource(member_b, in_a).await.unwrap());
    // No effective org ⇒ fail-closed under by_organization.
    assert!(!backend.visible_resource(member_a, orphan).await.unwrap());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn segmentation_by_org_and_team(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    set_segmentation_mode(&pool, "by_organization_and_team").await;
    let org = seed_org(&pool, "acme").await;
    let team = seed_team(&pool, org, "eng").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let team_resource = seed_resource(&pool, floor, Some(org), Some(team), None).await;
    let org_wide = seed_resource(&pool, floor, Some(org), None, None).await;

    let in_team = seed_user(&pool, "t@e.test", false).await;
    add_membership(&pool, in_team, org, Some(team), "member").await;
    let org_only = seed_user(&pool, "o@e.test", false).await;
    add_membership(&pool, org_only, org, None, "member").await;
    let backend = AuthzBackend::new(pool.clone());

    assert!(
        backend
            .visible_resource(in_team, team_resource)
            .await
            .unwrap()
    );
    assert!(
        !backend
            .visible_resource(org_only, team_resource)
            .await
            .unwrap()
    );
    // Org-wide (no team) resource is visible to any org member.
    assert!(backend.visible_resource(org_only, org_wide).await.unwrap());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn segmentation_mode_change_flips_visibility(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let outsider = seed_org(&pool, "beta").await;
    let user = seed_user(&pool, "b@e.test", false).await;
    add_membership(&pool, user, outsider, None, "member").await;
    let backend = AuthzBackend::new(pool.clone());

    set_segmentation_mode(&pool, "open").await;
    assert!(backend.visible_resource(user, resource).await.unwrap());
    set_segmentation_mode(&pool, "by_organization").await;
    assert!(!backend.visible_resource(user, resource).await.unwrap());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn segmentation_effective_precedence_resource_over_zone(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    set_segmentation_mode(&pool, "by_organization").await;
    let org_resource = seed_org(&pool, "alpha").await;
    let org_zone = seed_org(&pool, "beta").await;
    let floor = seed_location(&pool, "floor", "/loc/f1", 1, Some(org_zone)).await;
    let zone = seed_floor_zone(&pool, floor, Some(org_zone), None).await;
    // Resource explicitly bound to org_resource, sitting in a zone of org_zone.
    let resource = seed_resource(&pool, floor, Some(org_resource), None, Some(zone)).await;

    let sees = seed_user(&pool, "a@e.test", false).await;
    add_membership(&pool, sees, org_resource, None, "member").await;
    let blind = seed_user(&pool, "z@e.test", false).await;
    add_membership(&pool, blind, org_zone, None, "member").await;
    let backend = AuthzBackend::new(pool.clone());

    // Resource binding wins over the zone binding.
    assert!(backend.visible_resource(sees, resource).await.unwrap());
    assert!(!backend.visible_resource(blind, resource).await.unwrap());
}

// --- RLS parity (runtime role) -----------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn rls_matches_pure_visibility(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org_a = seed_org(&pool, "alpha").await;
    let org_b = seed_org(&pool, "beta").await;
    let team = seed_team(&pool, org_a, "eng").await;
    let floor_a = seed_location(&pool, "floor", "/alpha/f1", 1, Some(org_a)).await;
    let plain = seed_resource(&pool, floor_a, Some(org_a), None, None).await;
    let team_res = seed_resource(&pool, floor_a, Some(org_a), Some(team), None).await;

    let in_a_and_team = viewer(false, &[org_a], &[team]);
    let in_a_only = viewer(false, &[org_a], &[]);
    let in_b = viewer(false, &[org_b], &[]);

    for resource in [plain, team_res] {
        let effective = db::load_resource_segmentation(&pool, resource)
            .await
            .unwrap()
            .unwrap()
            .effective();
        for mode in [
            SegmentationMode::Open,
            SegmentationMode::ByOrganization,
            SegmentationMode::ByOrganizationAndTeam,
        ] {
            for v in [&in_a_and_team, &in_a_only, &in_b] {
                let expected = visible(effective, v, mode);
                let actual = rls_exists_as_runtime(&pool, resource, Ctx::Viewer(v, mode)).await;
                assert_eq!(
                    actual, expected,
                    "RLS vs visible() disagree: resource={resource:?} mode={mode:?} viewer={v:?}"
                );
            }
        }
    }
}

#[sqlx::test(migrations = "../db/migrations")]
async fn rls_no_context_is_fail_closed_and_system_sees_all(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;

    // No context set ⇒ no rows.
    let no_ctx = rls_exists_as_runtime(&pool, resource, Ctx::None).await;
    assert!(!no_ctx, "no context must be fail-closed");

    // System/elevated context ⇒ visible even without org membership.
    let sys = rls_exists_as_runtime(&pool, resource, Ctx::System).await;
    assert!(sys, "system context must see all rows");
}

// --- audit -------------------------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn audit_records_one_row_per_decision(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let user = seed_user(&pool, "u@e.test", false).await;
    add_membership(&pool, user, org, None, "member").await;
    let backend = AuthzBackend::new(pool.clone());

    backend
        .authorize(
            user,
            Action::BookingCreate,
            Target::Resource(resource),
            None,
        )
        .await
        .unwrap();
    assert!(denied(
        backend
            .authorize(
                user,
                Action::ResourceManage,
                Target::Resource(resource),
                None
            )
            .await
    ));

    assert_eq!(count_audit(&pool, "booking.create", "success").await, 1);
    assert_eq!(count_audit(&pool, "resource.manage", "denied").await, 1);

    // metadata carries no PII (IDs only) — here an empty JSON object.
    let meta: serde_json::Value =
        sqlx::query_scalar("SELECT metadata FROM audit_log WHERE action = 'booking.create'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(meta, serde_json::json!({}));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn audit_log_is_immutable_to_runtime_role(pool: PgPool) {
    db::seed_system_roles(&pool).await.unwrap();
    let org = seed_org(&pool, "acme").await;
    let floor = seed_location(&pool, "floor", "/acme/f1", 1, Some(org)).await;
    let resource = seed_resource(&pool, floor, Some(org), None, None).await;
    let user = seed_user(&pool, "u@e.test", false).await;
    add_membership(&pool, user, org, None, "member").await;
    let backend = AuthzBackend::new(pool.clone());
    backend
        .authorize(
            user,
            Action::BookingCreate,
            Target::Resource(resource),
            None,
        )
        .await
        .unwrap();

    // As the runtime role, UPDATE/DELETE are refused at the privilege level (42501),
    // independent of the immutability trigger.
    for stmt in ["UPDATE audit_log SET action = 'x'", "DELETE FROM audit_log"] {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL ROLE openworkspace_app")
            .execute(&mut *tx)
            .await
            .unwrap();
        let err = sqlx::query(stmt).execute(&mut *tx).await.unwrap_err();
        let code = err
            .as_database_error()
            .and_then(sqlx::error::DatabaseError::code)
            .map(std::borrow::Cow::into_owned);
        assert_eq!(
            code.as_deref(),
            Some("42501"),
            "expected insufficient_privilege for: {stmt}"
        );
        tx.rollback().await.unwrap();
    }
}

// --- system-role seed --------------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn seed_system_roles_is_idempotent_and_matches_domain(pool: PgPool) {
    use domain::authz::PermissionSet;

    db::seed_system_roles(&pool).await.unwrap();
    db::seed_system_roles(&pool).await.unwrap(); // idempotent

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM roles WHERE is_system")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 3);

    for (key, expected) in [
        ("owner", PermissionSet::builtin_owner()),
        ("admin", PermissionSet::builtin_admin()),
        ("member", PermissionSet::builtin_member()),
    ] {
        let tokens: Vec<String> = sqlx::query_scalar(
            "SELECT permission FROM role_permissions rp \
             JOIN roles r ON r.id = rp.role_id WHERE r.key = $1::citext",
        )
        .bind(key)
        .fetch_all(&pool)
        .await
        .unwrap();
        let seeded: HashSet<&str> = tokens.iter().map(String::as_str).collect();
        let domain_tokens: HashSet<&str> = expected.iter().map(Action::token).collect();
        assert_eq!(
            seeded, domain_tokens,
            "seeded `{key}` permissions drift from domain"
        );
    }
}
