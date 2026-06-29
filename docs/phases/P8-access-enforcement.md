# P8 — Access enforcement

> Source of truth: `OpenWorkspace-architecture-plan.md` (Appendix D "P8";
> §6.2 identity/segmentation/delegation; §6.4 scoped admin; Appendix H schema and
> "Least privilege at the database"). Read those before changing this phase.

## Goal

Wire the pure, already-tested authorization model from P4 into the database so
**allow and deny are enforced end to end across roles, scopes, delegation and
segmentation**, with PostgreSQL row-level security as defence in depth.

The decision logic itself is **not** re-implemented here — it lives in `domain`:

- `domain::authz::authorize(ctx, action, target, sep, now)` — deny-by-default
  union of instance-admin / org-role / location-grant.
- `domain::authz::Delegation::as_principal(actor, on_behalf_of, now)` — single-hop
  delegation, authority clamped to the booking family.
- `domain::segmentation::visible(effective, viewer, mode)` — fail-closed resource
  visibility.

P8 supplies these functions' inputs from the schema, records every outcome, and
adds the RLS backstop.

## Scope

In scope (headless — proven by integration tests, no UI/server-fns yet; those are
P13/P14):

1. **`db::access`** — loaders that materialize `AuthzContext`, `ViewerSegmentation`,
   `ResourceSegmentation`/mode, an active `Delegation`, and a target's
   `ManagementTarget`; the append-only audit-log writer; the RLS
   connection-context helpers; and the idempotent system-role seed (sourced from
   the `domain` builtin permission sets).
2. **`auth::authz::AuthzBackend`** — the single place permissions are decided:
   `load → domain::authorize → audit`, plus `visible_resource`. Typed
   `AuthzError` with an HTTP mapping for P13/P14 consumers.
3. **Postgres RLS** on `resources` — a SQL port of `segmentation::visible`,
   evaluated against transaction-local GUCs; parity with the pure function is
   pinned by tests.
4. **DB least-privilege** (the privilege-separation slice of P20, pulled forward so
   RLS is real): the runtime role `openworkspace_app` (DML only, no DDL, no
   `UPDATE`/`DELETE` on `audit_log`); the app/worker serve under it, migrations run
   as the owner.

Out of scope (later phases): `#[server]` functions, booking/admin UI and the
"book on behalf" dialog (P13/P14/P17); custom runtime roles and the role editor
(P17); the booking/occurrence read-path visibility matrix
(`public`/`org_visible`/`private`) and its RLS (P13/P14, where its consuming
queries are built); audit hash-chaining, the runtime-role REVOKE on future audit
partitions, backups, load testing, security review and CBOM (P20).

## Data model touched

Read, not changed (P2 schema): `users.is_instance_admin`, `memberships`,
`roles`/`role_permissions`, `role_grants` (location-scoped, validity windows),
`booking_delegates`, `locations.path`, `resources`/`floor_zones` org+team columns,
`instance_settings.segmentation_mode`, `audit_log`.

Added by the `p8_access_enforcement` migration: the `app` schema + RLS helper
functions, RLS + policies on `resources`, the `openworkspace_app` role with its
grants, the `audit_log` `UPDATE`/`DELETE` REVOKE, and `ALTER DEFAULT PRIVILEGES`
for future tables.

## Enforcement points

- **Server-side (the authority):** `auth::AuthzBackend`. Every check loads the
  actor's context, resolves delegation, resolves the target, calls the pure
  decision, and writes one audit row (`success` / `denied`). This is where P13/P14
  server functions gate every mutating/visibility-sensitive action.
- **Query-layer (defence in depth):** RLS on `resources`. A query that forgets to
  filter still cannot leak a resource across the segmentation boundary. The runtime
  role is `NOBYPASSRLS`, so `ENABLE ROW LEVEL SECURITY` alone governs it; the owner
  (migrations, tests) bypasses RLS.
- **Privilege-level (hard boundary):** the runtime role cannot run DDL and cannot
  mutate `audit_log` — independent of, and complementing, the immutability trigger.

## Acceptance criteria

Proven by `crates/auth/tests/authz_*.rs` against real PostgreSQL 18:

- instance admin allowed on every action × target kind; `member`/`admin`/`owner`
  tiers confer exactly their builtin permission sets; org roles confined to their
  org; unknown permission tokens confer nothing.
- location grants cover their subtree (and only it — prefix-collision safe), honour
  the validity window, never confer governance actions, and apply to team subjects.
- a delegate acts as the principal clamped to the booking family; missing / inactive
  / wrong-delegate ⇒ denied; the audit row carries `actor` + `on_behalf_of`.
- segmentation `open` / `by_organization` / `by_organization_and_team` allow and
  hide resources correctly; changing the mode changes visibility.
- RLS parity: a `SELECT` as the runtime role under a viewer context returns exactly
  what `segmentation::visible` predicts; no context ⇒ zero rows.
- every decision writes one audit row; the runtime role is refused `UPDATE`/`DELETE`
  on `audit_log` at the privilege level (and the trigger blocks the owner too).

## Verification

`cargo nextest run -p auth -p db` against the dev container (`DATABASE_URL` = the
owner/superuser); then the full workspace gate. Offline cache: `cargo sqlx prepare
--workspace`.
