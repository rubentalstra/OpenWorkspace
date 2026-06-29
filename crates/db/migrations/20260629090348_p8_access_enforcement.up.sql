-- P8 access enforcement: the database half of the authorization layer.
--
-- Two parts, both reversible:
--   1. The `app` schema + RLS on `resources` — defence-in-depth on segmentation,
--      keyed on the resolved organization/team (Appendix H §"Least privilege at
--      the database"). The policy MIRRORS `domain::segmentation::visible` /
--      `ResourceSegmentation::effective`; `crates/auth/tests/authz_rls.rs` pins the
--      two together so they cannot drift.
--   2. The least-privilege runtime role `openworkspace_app` (DML only, no DDL, no
--      UPDATE/DELETE on the append-only `audit_log`). The app/worker connect as
--      this role; migrations run as the owner. Superusers/owner bypass RLS, so
--      every migration and `#[sqlx::test]` (run as the owner) is unaffected — only
--      the runtime role is governed by these policies.
--
-- The RLS context is carried in transaction-local GUCs set by the `db::access`
-- helpers (`set_config('app.…', …, true)`), read here via
-- `current_setting('app.…', true)` (missing_ok ⇒ NULL ⇒ fail-closed).

CREATE SCHEMA IF NOT EXISTS app;

-- True when the current transaction runs with elevated authority: either a trusted
-- system path (`app.bypass`, set after the AuthzBackend has authorized a write) or
-- a viewer who is the instance admin. Either bypasses the segmentation predicate.
CREATE FUNCTION app.full_access() RETURNS boolean LANGUAGE sql STABLE
  SET search_path = pg_catalog AS $$
  SELECT coalesce(nullif(current_setting('app.bypass', true), '')::boolean, false)
      OR coalesce(nullif(current_setting('app.is_instance_admin', true), '')::boolean, false);
$$;

-- Membership test against a comma-separated UUID list GUC (the viewer's orgs/teams).
-- Compares as text to avoid casting an empty/partial list to uuid[].
CREATE FUNCTION app.csv_contains(csv text, val uuid) RETURNS boolean LANGUAGE sql STABLE
  SET search_path = pg_catalog AS $$
  SELECT coalesce(csv, '') <> '' AND val::text = ANY (string_to_array(csv, ','));
$$;

-- Whether a resource with the given own bindings is visible to the viewer carried
-- in the GUCs, under `app.segmentation_mode`. A faithful SQL port of
-- `ResourceSegmentation::effective` + `segmentation::visible`: effective org is
-- resource→zone→location; the effective team is paired with the org of the SAME
-- level (and is dropped if that level has no org — the domain `.map` behaviour),
-- and under by_organization_and_team the team's owning org must equal the
-- effective org (mixed-provenance rejection). Args are the resource row's own
-- columns, so this is never recursive into `resources` (no self-SELECT).
CREATE FUNCTION app.resource_visible(res_org uuid, res_team uuid, zone_id uuid, loc_id uuid)
  RETURNS boolean LANGUAGE plpgsql STABLE
  SET search_path = pg_catalog, public AS $$
DECLARE
  mode         text := current_setting('app.segmentation_mode', true);
  org_csv      text := current_setting('app.org_ids', true);
  team_csv     text := current_setting('app.team_ids', true);
  zone_org     uuid;
  zone_team    uuid;
  loc_org      uuid;
  eff_org      uuid;
  eff_team     uuid;
  eff_team_org uuid;
BEGIN
  IF mode IS NULL THEN
    RETURN false;               -- no context set ⇒ fail-closed
  END IF;
  IF mode = 'open' THEN
    RETURN true;
  END IF;

  IF zone_id IS NOT NULL THEN
    SELECT fz.organization_id, fz.team_id INTO zone_org, zone_team
    FROM public.floor_zones fz WHERE fz.id = zone_id;
  END IF;
  IF loc_id IS NOT NULL THEN
    SELECT l.organization_id INTO loc_org FROM public.locations l WHERE l.id = loc_id;
  END IF;

  eff_org := coalesce(res_org, zone_org, loc_org);

  -- Effective team travels with the org of the level that supplied it; if that
  -- level has no org, the team is dropped (matches the domain `.map(|o| (o, t))`).
  IF res_team IS NOT NULL AND res_org IS NOT NULL THEN
    eff_team := res_team; eff_team_org := res_org;
  ELSIF res_team IS NULL AND zone_team IS NOT NULL AND zone_org IS NOT NULL THEN
    eff_team := zone_team; eff_team_org := zone_org;
  ELSE
    eff_team := NULL; eff_team_org := NULL;
  END IF;

  -- by_organization (and the org half of by_organization_and_team): the effective
  -- org must be one of the viewer's orgs; a NULL effective org is fail-closed.
  IF eff_org IS NULL OR NOT app.csv_contains(org_csv, eff_org) THEN
    RETURN false;
  END IF;
  IF mode = 'by_organization' THEN
    RETURN true;
  END IF;

  IF mode = 'by_organization_and_team' THEN
    IF eff_team IS NULL THEN
      RETURN true;             -- org-wide resource: visible to any org member
    END IF;
    RETURN eff_team_org = eff_org AND app.csv_contains(team_csv, eff_team);
  END IF;

  RETURN false;               -- unknown mode ⇒ fail-closed
END;
$$;

-- Segmentation defence-in-depth on the resource read path. The AuthzBackend stays
-- authoritative; this guarantees a query that forgets to filter still cannot leak
-- a resource across the segmentation boundary. Writes are gated to the elevated
-- context (resource CRUD authorization is the AuthzBackend's job, P17).
ALTER TABLE resources ENABLE ROW LEVEL SECURITY;

CREATE POLICY resources_segmentation_select ON resources
  FOR SELECT
  USING (app.full_access()
         OR app.resource_visible(organization_id, team_id, floor_zone_id, location_id));

CREATE POLICY resources_write_insert ON resources
  FOR INSERT WITH CHECK (app.full_access());
CREATE POLICY resources_write_update ON resources
  FOR UPDATE USING (app.full_access()) WITH CHECK (app.full_access());
CREATE POLICY resources_write_delete ON resources
  FOR DELETE USING (app.full_access());

-- The least-privilege runtime role. Cluster-global, so created here race-safely as
-- a NOLOGIN/NOBYPASSRLS safety net (deployment grants LOGIN + a password: the dev
-- compose init, prod secrets). NOBYPASSRLS is the default but stated for intent.
DO $$
BEGIN
  CREATE ROLE openworkspace_app NOLOGIN NOBYPASSRLS;
EXCEPTION WHEN duplicate_object THEN
  NULL;  -- already provisioned (deployment) or created by a parallel test database
END $$;

GRANT USAGE ON SCHEMA public, app, tower_sessions TO openworkspace_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO openworkspace_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA tower_sessions TO openworkspace_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO openworkspace_app;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA app TO openworkspace_app;

-- Append-only audit log: INSERT + SELECT only, never mutate history. The
-- privilege-level guarantee that complements the immutability trigger — even a
-- SQL-injection foothold as the runtime role cannot rewrite the log. Revoked on
-- the partitioned parent AND its default partition (a direct partition write would
-- otherwise skip the parent ACL; the trigger still blocks it, this is belt-and-
-- suspenders). Future partitions (P20 retention) inherit this via the owner.
REVOKE UPDATE, DELETE ON audit_log, audit_log_default FROM openworkspace_app;

-- Tables/sequences created by later migrations (run as the owner) auto-grant.
ALTER DEFAULT PRIVILEGES IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO openworkspace_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO openworkspace_app;
