-- Reverse of cycle_fks.up.sql: drop the deferred constraints.
ALTER TABLE oidc_providers DROP CONSTRAINT IF EXISTS oidc_providers_default_organization_id_fkey;
ALTER TABLE oidc_providers DROP CONSTRAINT IF EXISTS oidc_providers_default_role_id_fkey;
ALTER TABLE assets         DROP CONSTRAINT IF EXISTS assets_uploaded_by_fkey;
ALTER TABLE locations      DROP CONSTRAINT IF EXISTS locations_map_image_asset_id_fkey;
ALTER TABLE users          DROP CONSTRAINT IF EXISTS users_invited_by_fkey;
ALTER TABLE users          DROP CONSTRAINT IF EXISTS users_default_category_id_fkey;
ALTER TABLE users          DROP CONSTRAINT IF EXISTS users_home_zone_id_fkey;
