-- Organizations, RBAC, locations, assets and resources.
-- assets.uploaded_by and locations.map_image_asset_id are cycle-closing FKs
-- (assets <-> users, locations <-> assets) added in the cycle_fks migration;
-- the columns exist here but their FK constraints are deferred.

CREATE TABLE organizations (
  id         uuid PRIMARY KEY DEFAULT uuidv7(),
  name       text NOT NULL,
  slug       citext UNIQUE NOT NULL,
  status     org_status NOT NULL DEFAULT 'active',  -- m3: native enum
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TRIGGER organizations_set_updated_at BEFORE UPDATE ON organizations
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE teams (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  name            text NOT NULL,
  created_at      timestamptz NOT NULL DEFAULT now(),
  updated_at      timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, name),
  -- M9: lets resources reference (team_id, organization_id) as a composite FK so a
  -- team can never be tagged with a different org than it belongs to.
  CONSTRAINT teams_id_org_uq UNIQUE (id, organization_id)
);
CREATE TRIGGER teams_set_updated_at BEFORE UPDATE ON teams
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE roles (
  id          uuid PRIMARY KEY DEFAULT uuidv7(),
  key         citext UNIQUE NOT NULL,
  name        text NOT NULL,
  description text,
  is_system   boolean NOT NULL DEFAULT false,
  created_at  timestamptz NOT NULL DEFAULT now(),
  updated_at  timestamptz NOT NULL DEFAULT now()  -- m9: mutable; maintained by trigger
);
CREATE TRIGGER roles_set_updated_at BEFORE UPDATE ON roles
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE role_permissions (
  role_id    uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  permission text NOT NULL,
  PRIMARY KEY (role_id, permission)
);

CREATE TABLE memberships (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id         uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  team_id         uuid REFERENCES teams(id) ON DELETE CASCADE,
  role_id         uuid NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
  created_at      timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX memberships_org_unique  ON memberships (user_id, organization_id) WHERE team_id IS NULL;
CREATE UNIQUE INDEX memberships_team_unique ON memberships (user_id, team_id)        WHERE team_id IS NOT NULL;

CREATE TABLE booking_delegates (
  id                uuid PRIMARY KEY DEFAULT uuidv7(),
  principal_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  delegate_user_id  uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  valid_from        timestamptz NOT NULL DEFAULT now(),
  valid_to          timestamptz,
  created_by        uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at        timestamptz NOT NULL DEFAULT now(),
  updated_at        timestamptz NOT NULL DEFAULT now(),  -- m9: mutable; maintained by trigger
  CONSTRAINT booking_delegates_distinct_check CHECK (principal_user_id <> delegate_user_id),
  -- M11: reject inverted/already-expired validity windows.
  CONSTRAINT booking_delegates_validity_check CHECK (valid_to IS NULL OR valid_to > valid_from),
  -- M11: no two overlapping delegate windows for the same (principal, delegate).
  -- Uses btree_gist for the equality columns over a half-open tstzrange window.
  CONSTRAINT booking_delegates_no_overlap
    EXCLUDE USING gist (
      principal_user_id WITH =,
      delegate_user_id  WITH =,
      tstzrange(valid_from, valid_to, '[)') WITH &&
    )
);
CREATE TRIGGER booking_delegates_set_updated_at BEFORE UPDATE ON booking_delegates
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX booking_delegates_principal_idx ON booking_delegates (principal_user_id);
CREATE INDEX booking_delegates_delegate_idx  ON booking_delegates (delegate_user_id);
-- M11: fast lookup of currently-active delegations.
CREATE INDEX booking_delegates_active_idx ON booking_delegates (principal_user_id, delegate_user_id)
  WHERE valid_to IS NULL;

CREATE TABLE assets (
  id                uuid PRIMARY KEY DEFAULT uuidv7(),
  kind              asset_kind NOT NULL,
  storage_key       text NOT NULL,
  content_type      text NOT NULL,
  byte_size         bigint NOT NULL,
  width             integer,
  height            integer,
  checksum          bytea,
  original_filename text,
  alt_text          text,
  parent_asset_id   uuid REFERENCES assets(id) ON DELETE CASCADE,
  variant           text,
  uploaded_by       uuid,             -- FK -> users(id) added in cycle_fks
  created_at        timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT assets_content_type_check
    CHECK (kind = 'export' OR content_type IN ('image/png','image/jpeg','image/webp','image/avif'))
);

CREATE TABLE locations (
  id                  uuid PRIMARY KEY DEFAULT uuidv7(),
  parent_id           uuid REFERENCES locations(id) ON DELETE RESTRICT,
  kind                location_kind NOT NULL,
  name                text NOT NULL,
  slug                citext,
  path                text NOT NULL,
  depth               integer NOT NULL,
  sort_order          integer NOT NULL DEFAULT 0,
  code                text,
  timezone            text,
  status              location_status NOT NULL DEFAULT 'active',  -- m3: native enum
  archived_at         timestamptz,
  address             text,
  latitude            numeric(9,6),
  longitude           numeric(9,6),
  organization_id     uuid REFERENCES organizations(id) ON DELETE SET NULL,
  map_image_asset_id  uuid,             -- FK -> assets(id) added in cycle_fks
  marker_x            numeric(6,5),
  marker_y            numeric(6,5),
  metadata            jsonb NOT NULL DEFAULT '{}'::jsonb,  -- m1: §3.2 promised; mirrors resources.attributes
  created_at          timestamptz NOT NULL DEFAULT now(),
  updated_at          timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT locations_map_image_kind_check CHECK (map_image_asset_id IS NULL OR kind = 'campus'),
  CONSTRAINT locations_marker_kind_check CHECK ((marker_x IS NULL AND marker_y IS NULL) OR kind = 'building'),
  CONSTRAINT locations_marker_range_check
    CHECK (marker_x IS NULL OR (marker_x BETWEEN 0 AND 1 AND marker_y BETWEEN 0 AND 1))
);
CREATE TRIGGER locations_set_updated_at BEFORE UPDATE ON locations
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX locations_parent_idx ON locations (parent_id);
CREATE INDEX locations_path_idx   ON locations (path text_pattern_ops);
CREATE INDEX locations_kind_idx   ON locations (kind);

CREATE TABLE role_grants (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  subject_user_id uuid REFERENCES users(id) ON DELETE CASCADE,
  subject_team_id uuid REFERENCES teams(id) ON DELETE CASCADE,
  role_id         uuid NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
  location_id     uuid NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
  valid_from      timestamptz NOT NULL DEFAULT now(),
  valid_to        timestamptz,
  created_by      uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at      timestamptz NOT NULL DEFAULT now(),
  updated_at      timestamptz NOT NULL DEFAULT now(),  -- m9: mutable; maintained by trigger
  CONSTRAINT role_grants_one_subject_check CHECK (num_nonnulls(subject_user_id, subject_team_id) = 1),
  -- M11: reject inverted/already-expired validity windows.
  CONSTRAINT role_grants_validity_check CHECK (valid_to IS NULL OR valid_to > valid_from)
);
CREATE TRIGGER role_grants_set_updated_at BEFORE UPDATE ON role_grants
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX role_grants_location_idx ON role_grants (location_id);

CREATE TABLE floor_plans (
  floor_id            uuid PRIMARY KEY REFERENCES locations(id) ON DELETE CASCADE,
  scene               jsonb NOT NULL DEFAULT '{}'::jsonb,
  -- m12: schema-version discriminator for the scene document, distinct from the
  -- optimistic-lock `version` counter, so the renderer can branch and scene
  -- migrations are mechanical.
  scene_schema_version integer NOT NULL DEFAULT 1,
  background_asset_id  uuid REFERENCES assets(id) ON DELETE SET NULL,
  viewbox             text,
  status              floor_plan_status NOT NULL DEFAULT 'published',  -- m3: native enum
  published_at        timestamptz,
  version             integer NOT NULL DEFAULT 1,
  updated_by          uuid REFERENCES users(id) ON DELETE SET NULL,
  updated_at          timestamptz NOT NULL DEFAULT now()
);
CREATE TRIGGER floor_plans_set_updated_at BEFORE UPDATE ON floor_plans
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- B5: floor zones (neighbourhoods). A zone assigns organization/team to every
-- resource inside it; effective org/team resolves resource override -> zone ->
-- location. scene_node_id binds the zone to its scene polygon.
CREATE TABLE floor_zones (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  floor_id        uuid NOT NULL REFERENCES locations(id) ON DELETE RESTRICT,
  name            text NOT NULL,
  organization_id uuid REFERENCES organizations(id),
  team_id         uuid REFERENCES teams(id),
  scene_node_id   text,
  created_at      timestamptz NOT NULL DEFAULT now(),
  -- M9 pattern: a zone's team must belong to its organization.
  CONSTRAINT floor_zones_team_org_fk
    FOREIGN KEY (team_id, organization_id) REFERENCES teams (id, organization_id)
);
CREATE INDEX floor_zones_floor_idx ON floor_zones (floor_id);

CREATE TABLE resource_categories (
  id         uuid PRIMARY KEY DEFAULT uuidv7(),
  name       text NOT NULL,
  color      text,
  icon       text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()  -- m9: mutable; maintained by trigger
);
CREATE TRIGGER resource_categories_set_updated_at BEFORE UPDATE ON resource_categories
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE resources (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  -- M10: generalised from floor_id -> location_id so non-floor resources (parking
  -- lots, vehicle pools at a site) have a home. Desks/rooms being floor-scoped is a
  -- domain rule, not a DB CHECK.
  location_id     uuid NOT NULL REFERENCES locations(id) ON DELETE RESTRICT,
  -- B5: optional zone the resource sits in; effective org/team resolves
  -- resource override -> zone -> location.
  floor_zone_id   uuid REFERENCES floor_zones(id) ON DELETE SET NULL,
  kind            resource_kind NOT NULL,
  name            text NOT NULL,
  code            text,
  category_id     uuid REFERENCES resource_categories(id) ON DELETE SET NULL,
  organization_id uuid REFERENCES organizations(id) ON DELETE SET NULL,
  team_id         uuid REFERENCES teams(id) ON DELETE SET NULL,
  description     text,
  photo_asset_id  uuid REFERENCES assets(id) ON DELETE SET NULL,
  capacity        integer,
  status          resource_status NOT NULL DEFAULT 'active',
  bookable        boolean NOT NULL DEFAULT true,
  requires_checkin boolean NOT NULL DEFAULT true,
  is_accessible   boolean NOT NULL DEFAULT false,  -- m2: first-class accessibility (EN 301 549)
  attributes      jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at      timestamptz NOT NULL DEFAULT now(),
  updated_at      timestamptz NOT NULL DEFAULT now(),
  archived_at     timestamptz,
  UNIQUE (location_id, code),
  -- M9: a resource's (team, org) pair must be a real team-in-that-org, preventing
  -- cross-org team tagging that would corrupt by_organization_and_team visibility.
  CONSTRAINT resources_team_org_fk
    FOREIGN KEY (team_id, organization_id) REFERENCES teams (id, organization_id)
);
CREATE TRIGGER resources_set_updated_at BEFORE UPDATE ON resources
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX resources_location_idx ON resources (location_id);
CREATE INDEX resources_category_idx ON resources (category_id);
CREATE INDEX resources_name_trgm    ON resources USING gin (name gin_trgm_ops);
-- M9: segmentation filter columns are the hottest read-path predicates.
CREATE INDEX resources_org_idx  ON resources (organization_id);
CREATE INDEX resources_team_idx ON resources (team_id) WHERE team_id IS NOT NULL;
-- m2: accessibility filter / best-fit search.
CREATE INDEX resources_accessible_idx ON resources (is_accessible) WHERE is_accessible;

CREATE TABLE resource_positions (
  id            uuid PRIMARY KEY DEFAULT uuidv7(),
  resource_id   uuid NOT NULL UNIQUE REFERENCES resources(id) ON DELETE CASCADE,
  -- M10: kept as floor_id since only floor-scoped desks/rooms populate positions.
  floor_id      uuid NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
  scene_node_id text NOT NULL,
  created_at    timestamptz NOT NULL DEFAULT now(),
  UNIQUE (floor_id, scene_node_id)
);

CREATE TABLE resource_tags (
  id   uuid PRIMARY KEY DEFAULT uuidv7(),
  name citext UNIQUE NOT NULL
);

CREATE TABLE resource_tag_map (
  resource_id uuid NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
  tag_id      uuid NOT NULL REFERENCES resource_tags(id) ON DELETE CASCADE,
  PRIMARY KEY (resource_id, tag_id)
);

CREATE TABLE resource_rules (
  id                            uuid PRIMARY KEY DEFAULT uuidv7(),
  resource_id                   uuid NOT NULL UNIQUE REFERENCES resources(id) ON DELETE CASCADE,
  max_advance_days              integer,
  min_advance_minutes           integer,
  min_duration_minutes          integer,
  max_duration_minutes          integer,
  slot_granularity_minutes      integer,
  buffer_minutes                integer,
  max_per_user_per_day          integer,
  max_active_per_user           integer,
  cancellation_deadline_minutes integer,
  allow_recurrence              boolean NOT NULL DEFAULT true,
  -- M7: bound RRULE expansion so an unbounded rule cannot materialise unbounded rows.
  max_recurrence_count          integer,
  max_recurrence_horizon_days   integer,
  require_approval              boolean NOT NULL DEFAULT false,
  allowed_window                jsonb,
  created_at                    timestamptz NOT NULL DEFAULT now(),
  updated_at                    timestamptz NOT NULL DEFAULT now(),
  -- m7: non-negative duration/cap/threshold guards + duration ordering.
  CONSTRAINT resource_rules_max_advance_days_check        CHECK (max_advance_days IS NULL OR max_advance_days >= 0),
  CONSTRAINT resource_rules_min_advance_minutes_check     CHECK (min_advance_minutes IS NULL OR min_advance_minutes >= 0),
  CONSTRAINT resource_rules_min_duration_minutes_check    CHECK (min_duration_minutes IS NULL OR min_duration_minutes > 0),
  CONSTRAINT resource_rules_max_duration_minutes_check    CHECK (max_duration_minutes IS NULL OR max_duration_minutes > 0),
  CONSTRAINT resource_rules_duration_order_check
    CHECK (min_duration_minutes IS NULL OR max_duration_minutes IS NULL OR max_duration_minutes >= min_duration_minutes),
  CONSTRAINT resource_rules_slot_granularity_check        CHECK (slot_granularity_minutes IS NULL OR slot_granularity_minutes > 0),
  CONSTRAINT resource_rules_buffer_minutes_check          CHECK (buffer_minutes IS NULL OR buffer_minutes >= 0),
  CONSTRAINT resource_rules_max_per_user_per_day_check    CHECK (max_per_user_per_day IS NULL OR max_per_user_per_day >= 0),
  CONSTRAINT resource_rules_max_active_per_user_check     CHECK (max_active_per_user IS NULL OR max_active_per_user >= 0),
  CONSTRAINT resource_rules_cancellation_deadline_check   CHECK (cancellation_deadline_minutes IS NULL OR cancellation_deadline_minutes >= 0),
  CONSTRAINT resource_rules_max_recurrence_count_check    CHECK (max_recurrence_count IS NULL OR max_recurrence_count > 0),
  CONSTRAINT resource_rules_max_recurrence_horizon_check  CHECK (max_recurrence_horizon_days IS NULL OR max_recurrence_horizon_days > 0)
);
CREATE TRIGGER resource_rules_set_updated_at BEFORE UPDATE ON resource_rules
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE permanent_assignments (
  id          uuid PRIMARY KEY DEFAULT uuidv7(),
  resource_id uuid NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
  user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  validity    daterange NOT NULL DEFAULT daterange(CURRENT_DATE, NULL, '[)'),
  created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at  timestamptz NOT NULL DEFAULT now(),
  -- B3: pin canonical half-open bounds so the no-overlap guarantee is structural.
  CONSTRAINT permanent_assignments_validity_halfopen
    CHECK (validity = daterange(lower(validity), upper(validity), '[)')),
  CONSTRAINT permanent_assignments_no_overlap
    EXCLUDE USING gist (resource_id WITH =, validity WITH &&)
);

CREATE TABLE blackouts (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  resource_id     uuid REFERENCES resources(id) ON DELETE CASCADE,
  location_id     uuid REFERENCES locations(id) ON DELETE CASCADE,
  title           text,
  period          tstzrange NOT NULL,
  recurrence_rule text,
  reason          text,
  created_by      uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at      timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT blackouts_one_target_check CHECK (num_nonnulls(resource_id, location_id) = 1),
  CONSTRAINT blackouts_period_bounded_check
    CHECK (NOT isempty(period) AND lower(period) IS NOT NULL AND upper(period) IS NOT NULL),
  -- B3: canonical half-open bounds, consistent with booking_occurrences.
  CONSTRAINT blackouts_period_halfopen
    CHECK (period = tstzrange(lower(period), upper(period), '[)'))
);
-- B4: blackouts are scanned on the availability hot path; the PK alone forces full scans.
CREATE INDEX blackouts_resource_idx ON blackouts (resource_id) WHERE resource_id IS NOT NULL;
CREATE INDEX blackouts_location_idx ON blackouts (location_id) WHERE location_id IS NOT NULL;
CREATE INDEX blackouts_period_gist  ON blackouts USING gist (period);

CREATE TABLE favorite_resources (
  user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  resource_id uuid NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
  created_at  timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, resource_id)
);
