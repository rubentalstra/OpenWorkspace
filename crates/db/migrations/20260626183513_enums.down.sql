-- Reverse of enums.up.sql. Drop the function then all enum types.
-- Extensions are owned by the P1 migration and are NOT dropped here.
DROP FUNCTION IF EXISTS set_updated_at();

DROP TYPE IF EXISTS audit_outcome;
DROP TYPE IF EXISTS booking_source;
DROP TYPE IF EXISTS floor_plan_status;
DROP TYPE IF EXISTS location_status;
DROP TYPE IF EXISTS org_status;
DROP TYPE IF EXISTS user_default_view;
DROP TYPE IF EXISTS user_status;
DROP TYPE IF EXISTS import_status;
DROP TYPE IF EXISTS import_kind;
DROP TYPE IF EXISTS token_kind;
DROP TYPE IF EXISTS dsr_status;
DROP TYPE IF EXISTS dsr_kind;
DROP TYPE IF EXISTS actor_kind;
DROP TYPE IF EXISTS outbox_status;
DROP TYPE IF EXISTS asset_kind;
DROP TYPE IF EXISTS segmentation_mode;
DROP TYPE IF EXISTS booking_visibility;
DROP TYPE IF EXISTS occurrence_kind;
DROP TYPE IF EXISTS booking_status;
DROP TYPE IF EXISTS resource_status;
DROP TYPE IF EXISTS resource_kind;
DROP TYPE IF EXISTS location_kind;
