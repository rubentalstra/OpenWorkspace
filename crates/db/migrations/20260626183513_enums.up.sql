-- P2 enums + shared trigger function.
-- Extensions (btree_gist, pg_trgm, citext) are created by the P1 migration.
-- IF NOT EXISTS here is harmless and keeps this migration self-describing.
CREATE EXTENSION IF NOT EXISTS btree_gist;
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS citext;

-- location_kind carries 'site' and 'lot' (M10) so non-floor resources (parking
-- lots, vehicle pools) have a real location node. ALTER TYPE ... ADD VALUE cannot
-- run inside a multi-statement txn, so the full set is declared here up front.
CREATE TYPE location_kind     AS ENUM ('continent','country','city','campus','building','floor','site','lot');
CREATE TYPE resource_kind     AS ENUM ('desk','room','parking','vehicle','equipment');
CREATE TYPE resource_status   AS ENUM ('active','inactive','maintenance');
-- booking_status carries the full documented lifecycle including 'released' (B2:
-- auto-release distinct from no_show/cancelled). Declared in full here because
-- ALTER TYPE ... ADD VALUE cannot run in a txn alongside other uses of the type.
CREATE TYPE booking_status    AS ENUM ('booked','checked_in','checked_out','released','no_show','cancelled');
-- occurrence_kind (B4): booking_occurrences is the single blocking table for user
-- bookings AND materialised system blocks (permanent assignments, blackouts).
CREATE TYPE occurrence_kind   AS ENUM ('booking','permanent_assignment','blackout');
-- booking_visibility (B1): per-series appointment visibility for the read-path filter.
CREATE TYPE booking_visibility AS ENUM ('public','org_visible','private');
CREATE TYPE segmentation_mode AS ENUM ('open','by_organization','by_organization_and_team');
CREATE TYPE asset_kind        AS ENUM ('reference_image','campus_map','floor_background','object_photo','logo','export');
CREATE TYPE outbox_status     AS ENUM ('pending','sent','failed','cancelled');
CREATE TYPE actor_kind        AS ENUM ('user','api_key','system');
CREATE TYPE dsr_kind          AS ENUM ('export','erasure','rectification');
CREATE TYPE dsr_status        AS ENUM ('received','in_progress','completed','rejected');
CREATE TYPE token_kind        AS ENUM ('invitation','password_reset','email_verification');
CREATE TYPE import_kind       AS ENUM ('users','resources');
CREATE TYPE import_status     AS ENUM ('pending','processing','completed','failed');

-- m3: adopt native ENUM for closed, app-controlled status/view/source/outcome
-- sets (CHECK-text was applied inconsistently). Policy: native ENUM for closed
-- sets the application owns; CHECK-text/lookup only for admin-editable/churning
-- sets. Documented in the migrations README convention.
CREATE TYPE user_status        AS ENUM ('active','suspended','deactivated');
CREATE TYPE user_default_view  AS ENUM ('map','list','calendar');
CREATE TYPE org_status         AS ENUM ('active','archived');
CREATE TYPE location_status    AS ENUM ('active','archived');
CREATE TYPE floor_plan_status  AS ENUM ('draft','published');
CREATE TYPE booking_source     AS ENUM ('web','api','import','delegate');
CREATE TYPE audit_outcome      AS ENUM ('success','failure','denied');

-- Shared trigger function maintaining updated_at on mutable tables (UTC now()).
CREATE FUNCTION set_updated_at() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END $$;
