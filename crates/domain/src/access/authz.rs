//! The pure authorization model.
//!
//! [`authorize`] is total and clock-injected: it never panics, never reads a
//! clock, performs no I/O, and returns a [`Decision`] from caller-materialized
//! facts (memberships, location-scoped grants, the instance-admin flag) already
//! resolved from the database. The actual query wiring lives in a later phase;
//! this module is the authoritative, exhaustively-tested decision layer.
//!
//! The composition is **deny-by-default**, a union of three independent
//! sources of authority:
//!
//! 1. **Instance admin** — bypasses scope, validity and segmentation entirely.
//! 2. **Organization role** — a [`Membership`] authorizes only against a target
//!    in the *same* organization (org roles are confined to their org; a target
//!    with no organization denies org-role access — fail-closed).
//! 3. **Location-scoped grant** — a [`RoleGrant`] authorizes a delegable action
//!    on a target located within the grant's subtree, while the grant's
//!    [`ValidityWindow`] is active.
//!
//! [`Delegation`] lets one principal act on another's behalf (single-hop,
//! bounded to the principal's own rights).

use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::ids::{LocationId, OrganizationId, TeamId, UserId};

/// A permissioned action a caller may attempt.
///
/// Closed vocabulary (13 variants). Each maps to a stable string token used in
/// the `role_permissions` table; [`Action::token`] / [`Action::from_token`]
/// convert at the `db` boundary. An unknown token resolves to `None`
/// (fail-closed): a permission string the binary does not understand grants
/// nothing.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    /// Create one's own bookings.
    BookingCreate,
    /// Create, edit and cancel one's own bookings.
    BookingManageOwn,
    /// Create bookings on behalf of others / manage anyone's bookings.
    BookingManageAny,
    /// Create, edit, archive bookable resources.
    ResourceManage,
    /// Edit the location hierarchy (campuses, buildings, floors).
    HierarchyEdit,
    /// Edit a floor's plan / scene and its zones.
    FloorBuild,
    /// Manage users (invite, assign roles, remove) within scope.
    UserManage,
    /// Read audit logs within scope.
    AuditView,
    /// Read usage statistics / analytics within scope.
    StatsView,
    /// Create, edit and delete roles and their grants.
    RoleManage,
    /// Create, rename, archive organizations and edit org settings.
    OrgManage,
    /// Administer GDPR / data-subject requests.
    GdprManage,
    /// Configure instance-wide settings (super-admin only).
    InstanceConfigure,
}

impl Action {
    /// The stable string token persisted in `role_permissions.permission`.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::BookingCreate => "booking.create",
            Self::BookingManageOwn => "booking.manage_own",
            Self::BookingManageAny => "booking.manage_any",
            Self::ResourceManage => "resource.manage",
            Self::HierarchyEdit => "hierarchy.edit",
            Self::FloorBuild => "floor.build",
            Self::UserManage => "user.manage",
            Self::AuditView => "audit.view",
            Self::StatsView => "stats.view",
            Self::RoleManage => "role.manage",
            Self::OrgManage => "org.manage",
            Self::GdprManage => "gdpr.manage",
            Self::InstanceConfigure => "instance.configure",
        }
    }

    /// Parses a persisted token back to an [`Action`]. An unrecognized token
    /// yields `None` (fail-closed: it confers no authority).
    #[must_use]
    pub fn from_token(token: &str) -> Option<Self> {
        Some(match token {
            "booking.create" => Self::BookingCreate,
            "booking.manage_own" => Self::BookingManageOwn,
            "booking.manage_any" => Self::BookingManageAny,
            "resource.manage" => Self::ResourceManage,
            "hierarchy.edit" => Self::HierarchyEdit,
            "floor.build" => Self::FloorBuild,
            "user.manage" => Self::UserManage,
            "audit.view" => Self::AuditView,
            "stats.view" => Self::StatsView,
            "role.manage" => Self::RoleManage,
            "org.manage" => Self::OrgManage,
            "gdpr.manage" => Self::GdprManage,
            "instance.configure" => Self::InstanceConfigure,
            _ => return None,
        })
    }

    /// Whether a location-scoped [`RoleGrant`] may confer this action.
    ///
    /// The four governance actions ([`Action::RoleManage`],
    /// [`Action::OrgManage`], [`Action::GdprManage`],
    /// [`Action::InstanceConfigure`]) are **never** delegable through a
    /// location grant: they require the instance admin or an organization
    /// membership. Every other action — the booking family and the scoped-admin
    /// actions — is location-delegable.
    #[must_use]
    pub const fn is_delegable_by_grant(self) -> bool {
        !matches!(
            self,
            Self::RoleManage | Self::OrgManage | Self::GdprManage | Self::InstanceConfigure
        )
    }

    /// Whether this action belongs to the booking family
    /// ([`Action::BookingCreate`], [`Action::BookingManageOwn`],
    /// [`Action::BookingManageAny`]).
    ///
    /// A delegation restricts the principal's borrowed authority to exactly
    /// this family (see [`Delegation::as_principal`]).
    #[must_use]
    pub const fn is_booking_family(self) -> bool {
        matches!(
            self,
            Self::BookingCreate | Self::BookingManageOwn | Self::BookingManageAny
        )
    }
}

/// A resolved set of [`Action`]s — what a role (membership or grant) confers.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PermissionSet(HashSet<Action>);

impl PermissionSet {
    /// An empty set (confers nothing).
    #[must_use]
    pub fn empty() -> Self {
        Self(HashSet::new())
    }

    /// Builds a set from explicit actions.
    #[must_use]
    pub fn new(actions: impl IntoIterator<Item = Action>) -> Self {
        Self(actions.into_iter().collect())
    }

    /// Builds a set from persisted permission tokens, **silently dropping**
    /// unknown tokens (fail-closed: an unrecognized permission grants nothing).
    #[must_use]
    pub fn from_tokens<'a>(tokens: impl IntoIterator<Item = &'a str>) -> Self {
        Self(tokens.into_iter().filter_map(Action::from_token).collect())
    }

    /// Whether this set confers `action`.
    #[must_use]
    pub fn contains(&self, action: Action) -> bool {
        self.0.contains(&action)
    }

    /// The number of distinct actions in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates the actions in the set (unspecified order).
    pub fn iter(&self) -> impl Iterator<Item = Action> + '_ {
        self.0.iter().copied()
    }

    /// A new set containing only the actions present in both `self` and
    /// `other` (set intersection). Used to clamp borrowed authority during
    /// delegation.
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Self {
        Self(self.0.intersection(&other.0).copied().collect())
    }

    /// The booking-family action set
    /// ({[`Action::BookingCreate`], [`Action::BookingManageOwn`],
    /// [`Action::BookingManageAny`]}). A delegation clamps the principal's
    /// authority to this set.
    #[must_use]
    pub fn booking_family() -> Self {
        Self::new([
            Action::BookingCreate,
            Action::BookingManageOwn,
            Action::BookingManageAny,
        ])
    }

    /// The built-in `member` tier: day-to-day use, no management authority.
    ///
    /// A member may create and manage their own bookings, but holds no
    /// management action. This is the single source of truth that also seeds
    /// `role_permissions` in a later phase.
    #[must_use]
    pub fn builtin_member() -> Self {
        Self::new([Action::BookingCreate, Action::BookingManageOwn])
    }

    /// The built-in `admin` tier: everything `member` has, plus full
    /// operational management within scope — excluding role administration,
    /// org lifecycle and instance/GDPR governance.
    ///
    /// [`Action::InstanceConfigure`] is never in a builtin set: it is held only
    /// via the instance-admin flag.
    #[must_use]
    pub fn builtin_admin() -> Self {
        let mut set = Self::builtin_member();
        set.0.extend([
            Action::ResourceManage,
            Action::HierarchyEdit,
            Action::FloorBuild,
            Action::UserManage,
            Action::AuditView,
            Action::StatsView,
            Action::BookingManageAny,
        ]);
        set
    }

    /// The built-in `owner` tier: everything `admin` has, plus role
    /// administration, organization lifecycle and GDPR governance. Owner is an
    /// org-level tier and intentionally does **not** include
    /// [`Action::InstanceConfigure`], which belongs to the instance admin.
    #[must_use]
    pub fn builtin_owner() -> Self {
        let mut set = Self::builtin_admin();
        set.0
            .extend([Action::RoleManage, Action::OrgManage, Action::GdprManage]);
        set
    }
}

/// A half-open `[from, to)` validity window for a time-bounded grant. A `None`
/// bound is unbounded on that side.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ValidityWindow {
    /// Inclusive lower bound; `None` ⇒ active from the beginning of time.
    pub from: Option<DateTime<Utc>>,
    /// Exclusive upper bound; `None` ⇒ never expires.
    pub to: Option<DateTime<Utc>>,
}

impl ValidityWindow {
    /// An always-active window (both bounds open).
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            from: None,
            to: None,
        }
    }

    /// Whether `now` lies in `[from, to)`. Half-open: `now == from` is active,
    /// `now == to` is **not**.
    #[must_use]
    pub fn active_at(&self, now: DateTime<Utc>) -> bool {
        let after_start = self.from.is_none_or(|from| now >= from);
        let before_end = self.to.is_none_or(|to| now < to);
        after_start && before_end
    }
}

/// A user's membership in an organization (optionally scoped to a team), with
/// its role's permissions pre-resolved by the caller.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Membership {
    /// The organization this membership belongs to. Org roles are confined
    /// here: the membership authorizes only targets in this organization.
    pub organization: OrganizationId,
    /// The team the membership is scoped to, if any.
    pub team: Option<TeamId>,
    /// The actions the membership's role confers.
    pub permissions: PermissionSet,
}

/// The subject a [`RoleGrant`] is bound to — a single user or a team.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GrantSubject {
    /// Bound to one user directly.
    User(UserId),
    /// Bound to every member of a team.
    Team(TeamId),
}

/// A location node in the hierarchy, identified by id and its `/`-joined path.
///
/// `path` is the materialized ancestry path (`locations.path`); the path
/// separator is injected at call time so this type makes no assumption about
/// it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LocationNode {
    /// The location's id.
    pub id: LocationId,
    /// The `/`-joined materialized path (e.g. `/root/b1/f1`).
    pub path: String,
}

/// A location-scoped role grant: a subject is granted a role's permissions over
/// a location subtree for a validity window.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RoleGrant {
    /// Who the grant applies to.
    pub subject: GrantSubject,
    /// The actions the granted role confers.
    pub permissions: PermissionSet,
    /// The root of the location subtree the grant covers.
    pub node: LocationNode,
    /// When the grant is active.
    pub validity: ValidityWindow,
}

/// Everything the authorizer needs about the acting user, materialized by the
/// caller from the database. Roles are already resolved to [`PermissionSet`]s.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AuthzContext {
    /// The acting user.
    pub user: UserId,
    /// Whether the user is the platform instance admin (super-admin).
    pub is_instance_admin: bool,
    /// The user's organization/team memberships.
    pub memberships: Vec<Membership>,
    /// Teams the user belongs to — used to match team-subject grants.
    pub team_ids: HashSet<TeamId>,
    /// Location-scoped grants applicable to the user (already filtered to
    /// grants whose subject is the user or one of the user's teams).
    pub grants: Vec<RoleGrant>,
}

/// What is being acted upon: an optional location node and the organization the
/// target belongs to. A `None` organization denies all org-role authority
/// (fail-closed); a `None` location denies all location-grant authority.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ManagementTarget {
    /// The location the action targets, if it is location-scoped.
    pub location: Option<LocationNode>,
    /// The organization the target belongs to, if known.
    pub organization: Option<OrganizationId>,
}

/// Why a [`Decision`] resolved the way it did.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DecisionReason {
    /// Allowed because the user is the instance admin.
    InstanceAdmin,
    /// Allowed by an organization-role membership in the target's org.
    OrgRole,
    /// Allowed by a location-scoped grant covering the target subtree.
    LocationGrant,
    /// Denied: no source of authority covered the action/target.
    NotCovered,
}

/// The outcome of an authorization check.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Decision {
    /// Whether the action is permitted.
    pub allowed: bool,
    /// The authority (or lack thereof) behind the outcome.
    pub reason: DecisionReason,
}

impl Decision {
    const fn allow(reason: DecisionReason) -> Self {
        Self {
            allowed: true,
            reason,
        }
    }

    const fn deny() -> Self {
        Self {
            allowed: false,
            reason: DecisionReason::NotCovered,
        }
    }
}

/// Whether `target` is within (or equal to) the subtree rooted at `node`, by
/// the materialized-path test.
///
/// `target.path` equals `node`'s path (the node itself) **or** begins with that
/// path followed by `sep`. Appending the separator before the prefix test is
/// what prevents the classic prefix-collision bug: with `sep = "/"`, `/r/b1`
/// does **not** contain `/r/b10`, because `/r/b10` does not start with
/// `/r/b1/`.
///
/// Fail-closed on degenerate inputs: an empty `node.path` (which would
/// otherwise prefix-match every target — an instance-wide escalation) or an
/// empty `sep` (which would collapse the separator boundary and admit siblings)
/// both return `false`. A single trailing separator on `node.path` is
/// normalized away first, so a node stored as `/r/b1/` still contains
/// `/r/b1/f1`.
#[must_use]
pub fn within_subtree(target: &LocationNode, node: &LocationNode, sep: &str) -> bool {
    if node.path.is_empty() || sep.is_empty() {
        return false;
    }
    let base = node.path.strip_suffix(sep).unwrap_or(node.path.as_str());
    if base.is_empty() {
        return false;
    }
    if target.path == base {
        return true;
    }
    let mut prefix = String::with_capacity(base.len() + sep.len());
    prefix.push_str(base);
    prefix.push_str(sep);
    target.path.starts_with(&prefix)
}

/// Cycle-safe subtree test by walking `parent_of` from `target` up to `node`.
///
/// A fallback for callers that have the parent relation but not materialized
/// paths. Bounded by `max_depth` ancestor hops so a corrupt cyclic parent chain
/// can never loop forever — on exceeding the bound it returns `false`
/// (fail-closed).
#[must_use]
pub fn within_subtree_by_parent(
    target: LocationId,
    node: LocationId,
    parent_of: impl Fn(LocationId) -> Option<LocationId>,
    max_depth: usize,
) -> bool {
    let mut current = Some(target);
    let mut hops = 0;
    while let Some(id) = current {
        if id == node {
            return true;
        }
        if hops >= max_depth {
            return false;
        }
        hops += 1;
        current = parent_of(id);
    }
    false
}

/// Decides whether `ctx` may perform `action` on `target` at instant `now`.
///
/// Total and deny-by-default. Evaluates the three authority sources in order
/// and returns the first that allows:
///
/// 1. **Instance admin** — `ctx.is_instance_admin` ⇒ allow
///    ([`DecisionReason::InstanceAdmin`]), bypassing scope, validity and
///    segmentation.
/// 2. **Org-confined role** — a membership whose `organization` equals
///    `target.organization` and whose permissions contain `action` ⇒ allow
///    ([`DecisionReason::OrgRole`]). A `None` target organization denies this
///    path (fail-closed): an org role never reaches across orgs.
/// 3. **Location grant** — `action.is_delegable_by_grant()` and a grant that is
///    `active_at(now)`, contains `action`, and whose subtree contains
///    `target.location` ⇒ allow ([`DecisionReason::LocationGrant`]).
///
/// Otherwise denies with [`DecisionReason::NotCovered`].
#[must_use]
pub fn authorize(
    ctx: &AuthzContext,
    action: Action,
    target: &ManagementTarget,
    sep: &str,
    now: DateTime<Utc>,
) -> Decision {
    // (1) Instance admin bypasses everything.
    if ctx.is_instance_admin {
        return Decision::allow(DecisionReason::InstanceAdmin);
    }

    // (2) Organization-confined role. Only a membership in the target's own
    // organization can authorize; a target with no org is fail-closed here.
    if let Some(target_org) = target.organization {
        let by_org = ctx
            .memberships
            .iter()
            .any(|m| m.organization == target_org && m.permissions.contains(action));
        if by_org {
            return Decision::allow(DecisionReason::OrgRole);
        }
    }

    // (3) Location-scoped grant. Governance actions are never grant-delegable.
    if action.is_delegable_by_grant()
        && let Some(target_loc) = target.location.as_ref()
    {
        let by_grant = ctx.grants.iter().any(|g| {
            g.validity.active_at(now)
                && g.permissions.contains(action)
                && within_subtree(target_loc, &g.node, sep)
        });
        if by_grant {
            return Decision::allow(DecisionReason::LocationGrant);
        }
    }

    Decision::deny()
}

/// A delegation letting one user act on another's behalf for a window.
///
/// The `principal` is the full authorization context of the user being acted
/// for; acting as the principal is **bounded** — and further **clamped to the
/// booking family** (see [`Delegation::as_principal`]). A booking delegate of
/// an owner or instance admin therefore gets only that principal's booking
/// capabilities, never any administrative authority. Single-hop: a delegate
/// cannot re-delegate.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Delegation {
    /// The user this delegation authorizes to act for the principal. The
    /// delegation is honored only when this equals the acting user.
    pub delegate: UserId,
    /// The principal's authorization context (the rights being borrowed).
    pub principal: AuthzContext,
    /// When the delegation is active.
    pub window: ValidityWindow,
}

impl Delegation {
    /// Resolves the effective [`AuthzContext`] to authorize with when `actor`
    /// acts, optionally on behalf of a principal.
    ///
    /// - `on_behalf_of` is `None` ⇒ act as `actor` unchanged (returns `actor`).
    /// - an **active** delegation whose [`Delegation::delegate`] equals
    ///   `actor.user` ⇒ act as the principal, but with authority **restricted
    ///   to the booking family**: the derived context forces
    ///   `is_instance_admin` to `false` and intersects every membership's and
    ///   grant's permissions with [`PermissionSet::booking_family`]. The result
    ///   exposes only the principal's `booking.create` / `booking.manage_own` /
    ///   `booking.manage_any` capabilities — never `org.manage`, `user.manage`,
    ///   `role.manage`, `instance.configure`, `floor.build`, and so on.
    /// - an **inactive** delegation, or one whose `delegate` does not match
    ///   `actor.user` ⇒ `None` (deny: the delegation is not in force or was
    ///   issued to a different user).
    ///
    /// Single-hop: the returned context is the principal's own and carries no
    /// further delegation.
    #[must_use]
    pub fn as_principal(
        actor: AuthzContext,
        on_behalf_of: Option<&Self>,
        now: DateTime<Utc>,
    ) -> Option<AuthzContext> {
        let Some(delegation) = on_behalf_of else {
            return Some(actor);
        };
        if !delegation.window.active_at(now) || delegation.delegate != actor.user {
            return None;
        }
        let family = PermissionSet::booking_family();
        let principal = &delegation.principal;
        Some(AuthzContext {
            user: principal.user,
            is_instance_admin: false,
            memberships: principal
                .memberships
                .iter()
                .map(|m| Membership {
                    organization: m.organization,
                    team: m.team,
                    permissions: m.permissions.intersect(&family),
                })
                .collect(),
            team_ids: principal.team_ids.clone(),
            grants: principal
                .grants
                .iter()
                .map(|g| RoleGrant {
                    subject: g.subject,
                    permissions: g.permissions.intersect(&family),
                    node: g.node.clone(),
                    validity: g.validity,
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use proptest::prelude::*;
    use uuid::Uuid;

    fn at(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(secs, 0).single().expect("valid instant")
    }

    fn uid(n: u128) -> UserId {
        UserId::new(Uuid::from_u128(n))
    }
    fn org(n: u128) -> OrganizationId {
        OrganizationId::new(Uuid::from_u128(n))
    }
    fn loc(n: u128, path: &str) -> LocationNode {
        LocationNode {
            id: LocationId::new(Uuid::from_u128(n)),
            path: path.to_owned(),
        }
    }

    const SEP: &str = "/";

    fn empty_ctx() -> AuthzContext {
        AuthzContext {
            user: uid(1),
            is_instance_admin: false,
            memberships: Vec::new(),
            team_ids: HashSet::new(),
            grants: Vec::new(),
        }
    }

    // --- token round-trip + fail-closed ---

    const ALL_ACTIONS: [Action; 13] = [
        Action::BookingCreate,
        Action::BookingManageOwn,
        Action::BookingManageAny,
        Action::ResourceManage,
        Action::HierarchyEdit,
        Action::FloorBuild,
        Action::UserManage,
        Action::AuditView,
        Action::StatsView,
        Action::RoleManage,
        Action::OrgManage,
        Action::GdprManage,
        Action::InstanceConfigure,
    ];

    #[test]
    fn every_action_token_round_trips() {
        assert_eq!(ALL_ACTIONS.len(), 13);
        for a in ALL_ACTIONS {
            assert_eq!(Action::from_token(a.token()), Some(a));
        }
        // The exact approved token strings (a persisted wire contract).
        let expected = [
            (Action::BookingCreate, "booking.create"),
            (Action::BookingManageOwn, "booking.manage_own"),
            (Action::BookingManageAny, "booking.manage_any"),
            (Action::ResourceManage, "resource.manage"),
            (Action::HierarchyEdit, "hierarchy.edit"),
            (Action::FloorBuild, "floor.build"),
            (Action::UserManage, "user.manage"),
            (Action::AuditView, "audit.view"),
            (Action::StatsView, "stats.view"),
            (Action::RoleManage, "role.manage"),
            (Action::OrgManage, "org.manage"),
            (Action::GdprManage, "gdpr.manage"),
            (Action::InstanceConfigure, "instance.configure"),
        ];
        for (a, tok) in expected {
            assert_eq!(a.token(), tok);
            assert_eq!(Action::from_token(tok), Some(a));
        }
    }

    #[test]
    fn unknown_token_is_none() {
        assert_eq!(Action::from_token("does.not.exist"), None);
        assert_eq!(Action::from_token(""), None);
        // Removed / renamed tokens no longer resolve.
        for stale in [
            "team.manage",
            "member.manage",
            "location.manage",
            "resource.policy.manage",
            "schedule.manage",
            "report.view",
            "booking.manage_others",
        ] {
            assert_eq!(Action::from_token(stale), None, "stale token {stale}");
        }
        // A permission set built from a bad token confers nothing.
        let set = PermissionSet::from_tokens(["bogus", "resource.manage"]);
        assert!(set.contains(Action::ResourceManage));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn governance_actions_are_not_grant_delegable() {
        for a in [
            Action::RoleManage,
            Action::OrgManage,
            Action::GdprManage,
            Action::InstanceConfigure,
        ] {
            assert!(!a.is_delegable_by_grant(), "{a:?} must not be delegable");
        }
        // Everything else is: the booking family and the scoped-admin actions.
        for a in [
            Action::BookingCreate,
            Action::BookingManageOwn,
            Action::BookingManageAny,
            Action::ResourceManage,
            Action::HierarchyEdit,
            Action::FloorBuild,
            Action::UserManage,
            Action::AuditView,
            Action::StatsView,
        ] {
            assert!(a.is_delegable_by_grant(), "{a:?} must be delegable");
        }
    }

    // --- builtin tiers ---

    #[test]
    fn builtin_tiers_are_nested_correctly() {
        let member = PermissionSet::builtin_member();
        let admin = PermissionSet::builtin_admin();
        let owner = PermissionSet::builtin_owner();

        // Member is the weakest: only the two own-booking actions.
        assert!(member.contains(Action::BookingCreate));
        assert!(member.contains(Action::BookingManageOwn));
        assert!(!member.contains(Action::BookingManageAny));
        assert!(!member.contains(Action::ResourceManage));
        assert!(!member.contains(Action::AuditView));
        assert!(!member.contains(Action::StatsView));
        assert!(!member.contains(Action::OrgManage));
        assert_eq!(member.len(), 2);

        // Admin ⊇ member, plus scoped operations and manage-any bookings.
        for a in member.iter() {
            assert!(admin.contains(a), "admin must include member {a:?}");
        }
        for a in [
            Action::ResourceManage,
            Action::HierarchyEdit,
            Action::FloorBuild,
            Action::UserManage,
            Action::AuditView,
            Action::StatsView,
            Action::BookingManageAny,
        ] {
            assert!(admin.contains(a), "admin must include {a:?}");
        }
        assert!(!admin.contains(Action::RoleManage));
        assert!(!admin.contains(Action::OrgManage));
        assert!(!admin.contains(Action::GdprManage));
        assert!(!admin.contains(Action::InstanceConfigure));

        // Owner ⊇ admin, plus role/org/GDPR governance, but never instance.
        for a in admin.iter() {
            assert!(owner.contains(a), "owner must include admin {a:?}");
        }
        assert!(owner.contains(Action::RoleManage));
        assert!(owner.contains(Action::OrgManage));
        assert!(owner.contains(Action::GdprManage));
        assert!(!owner.contains(Action::InstanceConfigure));
    }

    // --- validity window ---

    #[test]
    fn validity_window_is_half_open() {
        let w = ValidityWindow {
            from: Some(at(100)),
            to: Some(at(200)),
        };
        assert!(!w.active_at(at(99)));
        assert!(w.active_at(at(100))); // inclusive start
        assert!(w.active_at(at(199)));
        assert!(!w.active_at(at(200))); // exclusive end
        assert!(ValidityWindow::unbounded().active_at(at(0)));
    }

    // --- within_subtree: the /r/b1 vs /r/b10 collision ---

    #[test]
    fn subtree_is_reflexive() {
        let n = loc(1, "/r/b1");
        assert!(within_subtree(&n, &n, SEP));
    }

    #[test]
    fn subtree_includes_strict_descendant() {
        let node = loc(1, "/r/b1");
        let desc = loc(2, "/r/b1/f1");
        assert!(within_subtree(&desc, &node, SEP));
    }

    #[test]
    fn subtree_prefix_collision_is_rejected() {
        let node = loc(1, "/r/b1");
        let sibling = loc(2, "/r/b10");
        // /r/b10 must NOT be considered inside /r/b1.
        assert!(!within_subtree(&sibling, &node, SEP));
        // and a true sibling is also out.
        let b2 = loc(3, "/r/b2");
        assert!(!within_subtree(&b2, &node, SEP));
    }

    // --- FIX 3 regression: fail-closed on degenerate inputs ---

    #[test]
    fn subtree_empty_node_path_contains_nothing() {
        let empty = loc(1, "");
        for desc in ["/r", "/r/b1", "/anything", ""] {
            // An empty node path must never be treated as containing a target
            // (no instance-wide escalation). Even the empty-vs-empty case is
            // rejected because the node path is degenerate.
            assert!(
                !within_subtree(&loc(2, desc), &empty, SEP),
                "empty node must not contain {desc:?}"
            );
        }
    }

    #[test]
    fn subtree_empty_separator_rejects_sibling() {
        // With an empty separator the prefix boundary collapses, so a sibling
        // that merely shares a textual prefix must still be rejected.
        let node = loc(1, "/r/b1");
        let sibling = loc(2, "/r/b10");
        assert!(!within_subtree(&sibling, &node, ""));
        // A non-equal child is also rejected under an empty separator.
        assert!(!within_subtree(&loc(3, "/r/b1/f1"), &node, ""));
    }

    #[test]
    fn subtree_normalizes_trailing_separator() {
        // A node stored with a trailing separator resolves descendants exactly
        // like the same node without one.
        let with_sep = loc(1, "/r/b1/");
        let without = loc(1, "/r/b1");
        let desc = loc(2, "/r/b1/f1");
        let self_node = loc(3, "/r/b1");
        assert!(within_subtree(&desc, &with_sep, SEP));
        assert!(within_subtree(&desc, &without, SEP));
        assert!(within_subtree(&self_node, &with_sep, SEP));
        // And the collision is still rejected after normalization.
        assert!(!within_subtree(&loc(4, "/r/b10"), &with_sep, SEP));
    }

    #[test]
    fn instance_wide_grant_does_not_escalate() {
        // A delegable grant whose node.path is empty must not authorize over a
        // foreign-org target: no instance-wide escalation through within_subtree.
        let mut ctx = empty_ctx();
        ctx.grants.push(RoleGrant {
            subject: GrantSubject::User(ctx.user),
            permissions: PermissionSet::new([Action::ResourceManage]),
            node: loc(1, ""),
            validity: ValidityWindow::unbounded(),
        });
        let foreign = target(Some(org(42)), Some(loc(2, "/r/b1/f1")));
        let d = authorize(&ctx, Action::ResourceManage, &foreign, SEP, at(0));
        assert!(!d.allowed);
        assert_eq!(d.reason, DecisionReason::NotCovered);
    }

    #[test]
    fn subtree_by_parent_is_cycle_safe() {
        // 3 -> 2 -> 1 (root); and a cyclic chain 5 -> 6 -> 5.
        let parent_of = |id: LocationId| -> Option<LocationId> {
            let n = id.as_uuid().as_u128();
            match n {
                3 => Some(LocationId::new(Uuid::from_u128(2))),
                2 => Some(LocationId::new(Uuid::from_u128(1))),
                5 => Some(LocationId::new(Uuid::from_u128(6))),
                6 => Some(LocationId::new(Uuid::from_u128(5))),
                _ => None,
            }
        };
        let l = |n: u128| LocationId::new(Uuid::from_u128(n));
        assert!(within_subtree_by_parent(l(3), l(1), parent_of, 16));
        assert!(within_subtree_by_parent(l(1), l(1), parent_of, 16));
        assert!(!within_subtree_by_parent(l(3), l(4), parent_of, 16));
        // Cycle never loops; bounded and fail-closed.
        assert!(!within_subtree_by_parent(l(5), l(1), parent_of, 16));
    }

    // --- authorization matrix ---

    fn member_ctx(in_org: OrganizationId) -> AuthzContext {
        AuthzContext {
            memberships: vec![Membership {
                organization: in_org,
                team: None,
                permissions: PermissionSet::builtin_member(),
            }],
            ..empty_ctx()
        }
    }

    fn admin_ctx(in_org: OrganizationId) -> AuthzContext {
        AuthzContext {
            memberships: vec![Membership {
                organization: in_org,
                team: None,
                permissions: PermissionSet::builtin_admin(),
            }],
            ..empty_ctx()
        }
    }

    fn owner_ctx(in_org: OrganizationId) -> AuthzContext {
        AuthzContext {
            memberships: vec![Membership {
                organization: in_org,
                team: None,
                permissions: PermissionSet::builtin_owner(),
            }],
            ..empty_ctx()
        }
    }

    fn target(org_id: Option<OrganizationId>, location: Option<LocationNode>) -> ManagementTarget {
        ManagementTarget {
            location,
            organization: org_id,
        }
    }

    #[test]
    fn instance_admin_allows_everything_everywhere() {
        let mut ctx = empty_ctx();
        ctx.is_instance_admin = true;
        for a in [
            Action::InstanceConfigure,
            Action::GdprManage,
            Action::OrgManage,
            Action::ResourceManage,
        ] {
            // Even with no org and no location, and even for non-delegable actions.
            let d = authorize(&ctx, a, &target(None, None), SEP, at(0));
            assert!(d.allowed);
            assert_eq!(d.reason, DecisionReason::InstanceAdmin);
        }
    }

    #[test]
    fn member_denies_all_management() {
        let ctx = member_ctx(org(1));
        let t = target(Some(org(1)), Some(loc(1, "/r")));
        // All management and reporting actions are denied for a member.
        for a in [
            Action::ResourceManage,
            Action::HierarchyEdit,
            Action::FloorBuild,
            Action::UserManage,
            Action::AuditView,
            Action::StatsView,
            Action::BookingManageAny,
            Action::RoleManage,
            Action::OrgManage,
            Action::GdprManage,
            Action::InstanceConfigure,
        ] {
            let d = authorize(&ctx, a, &t, SEP, at(0));
            assert!(!d.allowed, "member must not {a:?}");
            assert_eq!(d.reason, DecisionReason::NotCovered);
        }
        // But the member's own booking actions in-org are allowed (OrgRole arm,
        // since the membership org matches the target org).
        for a in [Action::BookingCreate, Action::BookingManageOwn] {
            let d = authorize(&ctx, a, &t, SEP, at(0));
            assert!(d.allowed, "member should {a:?}");
            assert_eq!(d.reason, DecisionReason::OrgRole);
        }
    }

    #[test]
    fn org_admin_manages_in_org_but_not_cross_org() {
        let ctx = admin_ctx(org(1));
        // In-org resource.manage: allowed via OrgRole.
        let in_org = target(Some(org(1)), Some(loc(1, "/r")));
        let d = authorize(&ctx, Action::ResourceManage, &in_org, SEP, at(0));
        assert!(d.allowed);
        assert_eq!(d.reason, DecisionReason::OrgRole);

        // Cross-org: DENIED (confinement decision).
        let cross = target(Some(org(2)), Some(loc(1, "/r")));
        let d = authorize(&ctx, Action::ResourceManage, &cross, SEP, at(0));
        assert!(!d.allowed);
        assert_eq!(d.reason, DecisionReason::NotCovered);

        // Null target org: also denied (fail-closed).
        let no_org = target(None, Some(loc(1, "/r")));
        let d = authorize(&ctx, Action::ResourceManage, &no_org, SEP, at(0));
        assert!(!d.allowed);
    }

    #[test]
    fn org_admin_cannot_org_manage_but_owner_can() {
        let admin = admin_ctx(org(1));
        let owner = owner_ctx(org(1));
        let t = target(Some(org(1)), None);
        assert!(!authorize(&admin, Action::OrgManage, &t, SEP, at(0)).allowed);
        let d = authorize(&owner, Action::OrgManage, &t, SEP, at(0));
        assert!(d.allowed);
        assert_eq!(d.reason, DecisionReason::OrgRole);
    }

    #[test]
    fn instance_configure_only_via_instance_admin() {
        // Owner cannot instance.configure.
        let owner = owner_ctx(org(1));
        let t = target(Some(org(1)), Some(loc(1, "/r")));
        assert!(!authorize(&owner, Action::InstanceConfigure, &t, SEP, at(0)).allowed);

        // Not even a grant can (non-delegable), regardless of subtree/validity.
        let mut ctx = empty_ctx();
        ctx.grants.push(RoleGrant {
            subject: GrantSubject::User(ctx.user),
            permissions: PermissionSet::new([Action::InstanceConfigure, Action::GdprManage]),
            node: loc(1, "/r"),
            validity: ValidityWindow::unbounded(),
        });
        assert!(!authorize(&ctx, Action::InstanceConfigure, &t, SEP, at(0)).allowed);
        assert!(!authorize(&ctx, Action::GdprManage, &t, SEP, at(0)).allowed);
    }

    #[test]
    fn location_grant_respects_subtree_and_collision() {
        let mut ctx = empty_ctx();
        ctx.grants.push(RoleGrant {
            subject: GrantSubject::User(ctx.user),
            permissions: PermissionSet::new([Action::FloorBuild]),
            node: loc(1, "/r/b1"),
            validity: ValidityWindow::unbounded(),
        });
        // Under /r/b1/f1: allowed.
        let under = target(Some(org(9)), Some(loc(2, "/r/b1/f1")));
        let d = authorize(&ctx, Action::FloorBuild, &under, SEP, at(0));
        assert!(d.allowed);
        assert_eq!(d.reason, DecisionReason::LocationGrant);

        // The node itself: allowed.
        let node_self = target(Some(org(9)), Some(loc(1, "/r/b1")));
        assert!(authorize(&ctx, Action::FloorBuild, &node_self, SEP, at(0)).allowed);

        // Under /r/b2: denied.
        let b2 = target(Some(org(9)), Some(loc(3, "/r/b2/f1")));
        assert!(!authorize(&ctx, Action::FloorBuild, &b2, SEP, at(0)).allowed);

        // The /r/b10 collision: denied.
        let b10 = target(Some(org(9)), Some(loc(4, "/r/b10")));
        assert!(!authorize(&ctx, Action::FloorBuild, &b10, SEP, at(0)).allowed);
    }

    #[test]
    fn expired_grant_denies() {
        let mut ctx = empty_ctx();
        ctx.grants.push(RoleGrant {
            subject: GrantSubject::User(ctx.user),
            permissions: PermissionSet::new([Action::ResourceManage]),
            node: loc(1, "/r"),
            validity: ValidityWindow {
                from: Some(at(100)),
                to: Some(at(200)),
            },
        });
        let t = target(Some(org(9)), Some(loc(2, "/r/x")));
        // Active inside the window.
        assert!(authorize(&ctx, Action::ResourceManage, &t, SEP, at(150)).allowed);
        // Expired after the window.
        assert!(!authorize(&ctx, Action::ResourceManage, &t, SEP, at(250)).allowed);
        // Not yet active before the window.
        assert!(!authorize(&ctx, Action::ResourceManage, &t, SEP, at(50)).allowed);
    }

    #[test]
    fn grant_for_action_not_in_set_denies() {
        let mut ctx = empty_ctx();
        ctx.grants.push(RoleGrant {
            subject: GrantSubject::User(ctx.user),
            permissions: PermissionSet::new([Action::FloorBuild]),
            node: loc(1, "/r"),
            validity: ValidityWindow::unbounded(),
        });
        let t = target(Some(org(9)), Some(loc(2, "/r/x")));
        assert!(!authorize(&ctx, Action::ResourceManage, &t, SEP, at(0)).allowed);
    }

    // --- delegation ---

    #[test]
    fn delegation_none_acts_as_actor() {
        let actor = member_ctx(org(1));
        let resolved = Delegation::as_principal(actor.clone(), None, at(0));
        assert_eq!(resolved, Some(actor));
    }

    #[test]
    fn delegation_restricts_admin_to_booking_only() {
        // A is an org-1 admin; the actor is the named booking delegate. Acting
        // for A yields ONLY A's booking capabilities — admin actions are denied.
        let principal = admin_ctx(org(1));
        let mut actor = empty_ctx();
        actor.user = uid(2);
        let delegation = Delegation {
            delegate: actor.user,
            principal,
            window: ValidityWindow::unbounded(),
        };
        let resolved = Delegation::as_principal(actor, Some(&delegation), at(0))
            .expect("active delegation resolves");
        let t = target(Some(org(1)), Some(loc(1, "/r")));

        // Admin powers are DENIED through the delegation.
        for a in [
            Action::ResourceManage,
            Action::OrgManage,
            Action::InstanceConfigure,
            Action::UserManage,
            Action::FloorBuild,
        ] {
            let d = authorize(&resolved, a, &t, SEP, at(0));
            assert!(!d.allowed, "delegate must not {a:?}");
            assert_eq!(d.reason, DecisionReason::NotCovered);
        }

        // Booking-family powers the principal holds ARE allowed.
        for a in [Action::BookingCreate, Action::BookingManageOwn] {
            assert!(
                authorize(&resolved, a, &t, SEP, at(0)).allowed,
                "delegate should {a:?}"
            );
        }
        // The admin's manage-any booking authority survives the family clamp.
        assert!(authorize(&resolved, Action::BookingManageAny, &t, SEP, at(0)).allowed);
    }

    #[test]
    fn delegation_strips_instance_admin_to_booking_only() {
        // Owner/instance-admin principal + booking delegate: management denied,
        // booking allowed. (Mirrors FIX 1 (i) for an instance-admin principal.)
        let mut principal = owner_ctx(org(1));
        principal.is_instance_admin = true;
        let mut actor = empty_ctx();
        actor.user = uid(2);
        let delegation = Delegation {
            delegate: actor.user,
            principal,
            window: ValidityWindow::unbounded(),
        };
        let resolved = Delegation::as_principal(actor, Some(&delegation), at(0)).expect("active");
        assert!(!resolved.is_instance_admin);
        let t = target(Some(org(1)), Some(loc(1, "/r")));
        for a in [
            Action::ResourceManage,
            Action::OrgManage,
            Action::InstanceConfigure,
        ] {
            let d = authorize(&resolved, a, &t, SEP, at(0));
            assert!(!d.allowed, "delegate must not {a:?}");
            assert_eq!(d.reason, DecisionReason::NotCovered);
        }
        assert!(authorize(&resolved, Action::BookingManageOwn, &t, SEP, at(0)).allowed);
    }

    #[test]
    fn delegation_for_unrelated_actor_is_rejected() {
        // A delegation issued to user 9, exercised by actor user 2, is None.
        let principal = admin_ctx(org(1));
        let mut actor = empty_ctx();
        actor.user = uid(2);
        let delegation = Delegation {
            delegate: uid(9),
            principal,
            window: ValidityWindow::unbounded(),
        };
        assert_eq!(
            Delegation::as_principal(actor, Some(&delegation), at(0)),
            None
        );
    }

    #[test]
    fn inactive_delegation_denies() {
        let principal = admin_ctx(org(1));
        let actor = empty_ctx();
        let delegation = Delegation {
            delegate: actor.user,
            principal,
            window: ValidityWindow {
                from: Some(at(100)),
                to: Some(at(200)),
            },
        };
        assert_eq!(
            Delegation::as_principal(actor, Some(&delegation), at(250)),
            None
        );
    }

    // --- proptests: totality and the security invariants ---

    fn arb_action() -> impl Strategy<Value = Action> {
        prop_oneof![
            Just(Action::BookingCreate),
            Just(Action::BookingManageOwn),
            Just(Action::BookingManageAny),
            Just(Action::ResourceManage),
            Just(Action::HierarchyEdit),
            Just(Action::FloorBuild),
            Just(Action::UserManage),
            Just(Action::AuditView),
            Just(Action::StatsView),
            Just(Action::RoleManage),
            Just(Action::OrgManage),
            Just(Action::GdprManage),
            Just(Action::InstanceConfigure),
        ]
    }

    fn arb_permset() -> impl Strategy<Value = PermissionSet> {
        proptest::collection::vec(arb_action(), 0..6).prop_map(PermissionSet::new)
    }

    fn arb_path() -> impl Strategy<Value = String> {
        // A small alphabet that produces prefix-collision shapes, PLUS the empty
        // string and bare/non-rooted shapes (no leading separator) to exercise
        // the degenerate-input edges.
        let segs = proptest::collection::vec(
            prop_oneof![Just("a"), Just("b"), Just("b1"), Just("b10")],
            0..4,
        );
        prop_oneof![
            // Empty path.
            Just(String::new()),
            // Rooted: "/r" then segments.
            segs.clone().prop_map(|segs| {
                let mut p = String::from("/r");
                for s in segs {
                    p.push('/');
                    p.push_str(s);
                }
                p
            }),
            // Bare / non-rooted: no leading separator.
            proptest::collection::vec(prop_oneof![Just("r"), Just("b1"), Just("b10")], 1..4,)
                .prop_map(|segs| segs.join("/")),
        ]
    }

    fn arb_sep() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("/".to_owned()),
            Just(String::new()),
            Just("::".to_owned()),
        ]
    }

    fn arb_node() -> impl Strategy<Value = LocationNode> {
        (any::<u64>(), arb_path()).prop_map(|(n, path)| LocationNode {
            id: LocationId::new(Uuid::from_u128(u128::from(n))),
            path,
        })
    }

    fn arb_window() -> impl Strategy<Value = ValidityWindow> {
        (
            proptest::option::of(-1000i64..1000),
            proptest::option::of(-1000i64..1000),
        )
            .prop_map(|(f, t)| ValidityWindow {
                from: f.map(at),
                to: t.map(at),
            })
    }

    fn arb_ctx() -> impl Strategy<Value = AuthzContext> {
        let membership = (any::<u64>(), arb_permset()).prop_map(|(o, perms)| Membership {
            organization: OrganizationId::new(Uuid::from_u128(u128::from(o))),
            team: None,
            permissions: perms,
        });
        let grant =
            (arb_permset(), arb_node(), arb_window()).prop_map(|(perms, node, w)| RoleGrant {
                subject: GrantSubject::User(uid(1)),
                permissions: perms,
                node,
                validity: w,
            });
        (
            any::<bool>(),
            proptest::collection::vec(membership, 0..3),
            proptest::collection::vec(grant, 0..3),
        )
            .prop_map(|(is_admin, memberships, grants)| AuthzContext {
                user: uid(1),
                is_instance_admin: is_admin,
                memberships,
                team_ids: HashSet::new(),
                grants,
            })
    }

    fn arb_target() -> impl Strategy<Value = ManagementTarget> {
        (
            proptest::option::of(any::<u64>()),
            proptest::option::of(arb_node()),
        )
            .prop_map(|(o, loc)| ManagementTarget {
                location: loc,
                organization: o.map(|n| OrganizationId::new(Uuid::from_u128(u128::from(n)))),
            })
    }

    proptest! {
        #[test]
        fn authorize_is_total(
            ctx in arb_ctx(),
            action in arb_action(),
            t in arb_target(),
            now in -2000i64..2000,
        ) {
            // Never panics. The decision is internally consistent.
            let d = authorize(&ctx, action, &t, SEP, at(now));
            prop_assert_eq!(d.allowed, d.reason != DecisionReason::NotCovered);
        }

        #[test]
        fn instance_admin_always_allows(
            mut ctx in arb_ctx(),
            action in arb_action(),
            t in arb_target(),
            now in -2000i64..2000,
        ) {
            ctx.is_instance_admin = true;
            let d = authorize(&ctx, action, &t, SEP, at(now));
            prop_assert!(d.allowed);
            prop_assert_eq!(d.reason, DecisionReason::InstanceAdmin);
        }

        #[test]
        fn non_admin_deny_by_default_with_no_authority(
            action in arb_action(),
            t in arb_target(),
            now in -2000i64..2000,
        ) {
            // Empty context (no admin, no memberships, no grants) denies everything.
            let ctx = empty_ctx();
            let d = authorize(&ctx, action, &t, SEP, at(now));
            prop_assert!(!d.allowed);
            prop_assert_eq!(d.reason, DecisionReason::NotCovered);
        }

        #[test]
        fn org_role_never_authorizes_cross_org(
            perms in arb_permset(),
            action in arb_action(),
            loc in proptest::option::of(arb_node()),
            now in -2000i64..2000,
        ) {
            // A user whose ONLY authority is a membership in org 1.
            let ctx = AuthzContext {
                memberships: vec![Membership {
                    organization: org(1),
                    team: None,
                    permissions: perms,
                }],
                ..empty_ctx()
            };
            // Target is in org 2 (a different org), no grants exist.
            let t = ManagementTarget { location: loc, organization: Some(org(2)) };
            let d = authorize(&ctx, action, &t, SEP, at(now));
            // The org role can never reach across to org 2.
            prop_assert_ne!(d.reason, DecisionReason::OrgRole);
            // With no grants, the only possible allow would be OrgRole, so deny.
            prop_assert!(!d.allowed);
        }

        #[test]
        fn within_subtree_reflexive_and_never_sibling(node in arb_node()) {
            // Reflexivity holds only for a well-formed (non-empty) node path;
            // a degenerate empty node contains nothing, by design (FIX 3).
            if node.path.is_empty() {
                prop_assert!(!within_subtree(&node, &node, SEP));
            } else {
                prop_assert!(within_subtree(&node, &node, SEP));
            }
            // A true sibling (same parent, different leaf) is never inside.
            let sibling = LocationNode { id: node.id, path: format!("{}x_sib", node.path) };
            // `{path}x_sib` shares `node.path` as a prefix but without the separator,
            // so it must NOT be considered inside `node`.
            prop_assert!(!within_subtree(&sibling, &node, SEP));
        }

        #[test]
        fn empty_node_path_never_contains_distinct_target(
            target in arb_node(),
            sep in arb_sep(),
        ) {
            // An empty node path contains no distinct target under any sep.
            let empty = LocationNode { id: target.id, path: String::new() };
            prop_assume!(!target.path.is_empty());
            prop_assert!(!within_subtree(&target, &empty, &sep));
        }

        #[test]
        fn empty_sep_never_matches_non_equal_sibling(
            node in arb_node(),
            target in arb_node(),
        ) {
            // With an empty separator, only an exact (post-normalization) equal
            // path could match — and even that is fail-closed here.
            prop_assume!(target.path != node.path);
            prop_assert!(!within_subtree(&target, &node, ""));
        }

        #[test]
        fn within_subtree_implies_prefix_at_boundary(
            node in arb_node(),
            target in arb_node(),
            sep in arb_sep(),
        ) {
            // For well-formed inputs, a positive result implies node.path
            // (normalized) is a non-empty prefix of target.path up to a sep
            // boundary (equal, or followed by sep).
            if within_subtree(&target, &node, &sep) {
                prop_assert!(!node.path.is_empty());
                prop_assert!(!sep.is_empty());
                let base = node.path.strip_suffix(&sep).unwrap_or(node.path.as_str());
                prop_assert!(!base.is_empty());
                let boundary = target.path == base
                    || target.path.starts_with(&format!("{base}{sep}"));
                prop_assert!(boundary);
            }
        }

        #[test]
        fn authorize_is_total_over_all_seps(
            ctx in arb_ctx(),
            action in arb_action(),
            t in arb_target(),
            sep in arb_sep(),
            now in -2000i64..2000,
        ) {
            // authorize never panics across all generators (paths, seps), and
            // the decision stays internally consistent.
            let d = authorize(&ctx, action, &t, &sep, at(now));
            prop_assert_eq!(d.allowed, d.reason != DecisionReason::NotCovered);
        }

        #[test]
        fn delegation_result_is_subset_of_principal(
            actor in arb_ctx(),
            principal in arb_ctx(),
            action in arb_action(),
            t in arb_target(),
            now in -2000i64..2000,
            win in arb_window(),
        ) {
            // Bind the delegation to the actor so it can resolve.
            let delegation = Delegation {
                delegate: actor.user,
                principal: principal.clone(),
                window: win,
            };
            match Delegation::as_principal(actor, Some(&delegation), at(now)) {
                Some(resolved) => {
                    // The delegation clamps to the booking family, so any
                    // decision via the delegation is NEVER more permissive than
                    // the principal's own decision (delegation ⊆ principal).
                    let via = authorize(&resolved, action, &t, SEP, at(now));
                    let direct = authorize(&principal, action, &t, SEP, at(now));
                    if via.allowed {
                        prop_assert!(direct.allowed);
                    }
                    // A non-booking action is never allowed through a delegation.
                    if !action.is_booking_family() {
                        prop_assert!(!via.allowed);
                    }
                }
                None => {
                    // delegate == actor.user, so None means the window is inactive.
                    prop_assert!(!delegation.window.active_at(at(now)));
                }
            }
        }
    }
}
