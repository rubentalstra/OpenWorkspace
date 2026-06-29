-- Dev-only provisioning of the least-privilege runtime role the app and worker
-- connect as (config `dev.database.url`). Runs once on a fresh cluster, before
-- migrations. The role's GRANTs, the audit-log REVOKE and RLS are applied by the
-- `p8_access_enforcement` migration; this only makes the role loginnable for local
-- development. Production provisions an equivalent role via deployment secrets and
-- never uses this password.
DO $$
BEGIN
  CREATE ROLE openworkspace_app LOGIN PASSWORD 'devapp' NOBYPASSRLS;
EXCEPTION WHEN duplicate_object THEN
  ALTER ROLE openworkspace_app LOGIN PASSWORD 'devapp' NOBYPASSRLS;
END $$;
