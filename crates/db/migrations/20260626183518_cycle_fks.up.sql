-- Cycle-closing foreign keys, added last once every referenced table exists.
-- These close the users <-> locations / resource_categories / users(self),
-- locations <-> assets, assets <-> users, and oidc_providers -> roles/organizations
-- back-references that could not be declared inline without a creation-order cycle.

ALTER TABLE users
  ADD CONSTRAINT users_home_zone_id_fkey
    FOREIGN KEY (home_zone_id) REFERENCES locations(id) ON DELETE SET NULL;
ALTER TABLE users
  ADD CONSTRAINT users_default_category_id_fkey
    FOREIGN KEY (default_category_id) REFERENCES resource_categories(id) ON DELETE SET NULL;
ALTER TABLE users
  ADD CONSTRAINT users_invited_by_fkey
    FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE SET NULL;

ALTER TABLE locations
  ADD CONSTRAINT locations_map_image_asset_id_fkey
    FOREIGN KEY (map_image_asset_id) REFERENCES assets(id) ON DELETE SET NULL;

ALTER TABLE assets
  ADD CONSTRAINT assets_uploaded_by_fkey
    FOREIGN KEY (uploaded_by) REFERENCES users(id) ON DELETE SET NULL;

ALTER TABLE oidc_providers
  ADD CONSTRAINT oidc_providers_default_role_id_fkey
    FOREIGN KEY (default_role_id) REFERENCES roles(id) ON DELETE SET NULL;
ALTER TABLE oidc_providers
  ADD CONSTRAINT oidc_providers_default_organization_id_fkey
    FOREIGN KEY (default_organization_id) REFERENCES organizations(id) ON DELETE SET NULL;
