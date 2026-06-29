-- Reverse P8 access enforcement. The runtime role `openworkspace_app` is
-- intentionally NOT dropped: it is cluster-global and shared across parallel
-- `#[sqlx::test]` databases, and the up-migration's create is race-safe and
-- idempotent. We revoke its privileges in this database and drop the RLS/app
-- objects; the harmless NOLOGIN role remains.

ALTER DEFAULT PRIVILEGES IN SCHEMA public
  REVOKE SELECT, INSERT, UPDATE, DELETE ON TABLES FROM openworkspace_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
  REVOKE USAGE, SELECT ON SEQUENCES FROM openworkspace_app;

-- Drop the policies before the helper functions they call.
DROP POLICY IF EXISTS resources_write_delete ON resources;
DROP POLICY IF EXISTS resources_write_update ON resources;
DROP POLICY IF EXISTS resources_write_insert ON resources;
DROP POLICY IF EXISTS resources_segmentation_select ON resources;
ALTER TABLE resources DISABLE ROW LEVEL SECURITY;

REVOKE ALL ON ALL TABLES IN SCHEMA public FROM openworkspace_app;
REVOKE ALL ON ALL SEQUENCES IN SCHEMA public FROM openworkspace_app;
REVOKE ALL ON ALL TABLES IN SCHEMA tower_sessions FROM openworkspace_app;
REVOKE USAGE ON SCHEMA public, tower_sessions FROM openworkspace_app;

-- Drops the helper functions and the role's USAGE/EXECUTE on them with it.
DROP SCHEMA IF EXISTS app CASCADE;
