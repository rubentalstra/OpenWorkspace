-- Operational/admin tables and the append-only audit log.
-- The runtime-role REVOKE on audit_log is deferred to P20; immutability here is
-- enforced by the BEFORE UPDATE OR DELETE trigger.

CREATE TABLE email_outbox (
  id                 uuid PRIMARY KEY DEFAULT uuidv7(),
  to_address         citext NOT NULL,
  to_user_id         uuid REFERENCES users(id) ON DELETE SET NULL,
  cc                 text,
  reply_to           text,
  message_id         text,
  template_key       text NOT NULL,
  locale             text NOT NULL DEFAULT 'en',
  subject            text NOT NULL,
  body_html          text NOT NULL,
  body_text          text NOT NULL,
  ics_body           text,
  related_booking_id uuid REFERENCES bookings(id) ON DELETE SET NULL,
  idempotency_key    text UNIQUE NOT NULL,
  status             outbox_status NOT NULL DEFAULT 'pending',
  attempts           integer NOT NULL DEFAULT 0,
  last_error         text,
  scheduled_for      timestamptz NOT NULL DEFAULT now(),
  sent_at            timestamptz,
  created_at         timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX email_outbox_pending_idx ON email_outbox (scheduled_for) WHERE status = 'pending';

CREATE TABLE mail_templates (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  key             citext NOT NULL,
  locale          text NOT NULL DEFAULT 'en',
  subject_template text NOT NULL,
  html_template   text NOT NULL,
  text_template   text NOT NULL,
  enabled         boolean NOT NULL DEFAULT true,
  updated_by      uuid REFERENCES users(id) ON DELETE SET NULL,
  updated_at      timestamptz NOT NULL DEFAULT now(),
  UNIQUE (key, locale)
);
CREATE TRIGGER mail_templates_set_updated_at BEFORE UPDATE ON mail_templates
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE instance_settings (
  id                              boolean PRIMARY KEY DEFAULT true,
  segmentation_mode               segmentation_mode NOT NULL DEFAULT 'open',
  default_locale                  text NOT NULL DEFAULT 'en',
  default_timezone                text NOT NULL DEFAULT 'Europe/Amsterdam',
  product_name                    text NOT NULL DEFAULT 'OpenWorkspace',
  logo_asset_id                   uuid REFERENCES assets(id) ON DELETE SET NULL,
  primary_color                   text,
  checkin_window_minutes          integer NOT NULL DEFAULT 15,
  checkin_grace_minutes           integer NOT NULL DEFAULT 15,
  booking_horizon_days            integer NOT NULL DEFAULT 90,
  audit_retention_days            integer NOT NULL DEFAULT 365,
  booking_retention_days          integer NOT NULL DEFAULT 730,
  default_slot_granularity_minutes integer NOT NULL DEFAULT 30,
  default_max_advance_days        integer NOT NULL DEFAULT 90,
  local_login_enabled             boolean NOT NULL DEFAULT true,
  passkeys_enabled                boolean NOT NULL DEFAULT true,
  totp_enabled                    boolean NOT NULL DEFAULT true,
  allow_self_registration         boolean NOT NULL DEFAULT false,
  require_mfa                     boolean NOT NULL DEFAULT false,
  min_password_length             integer NOT NULL DEFAULT 12,
  session_idle_minutes            integer NOT NULL DEFAULT 480,
  session_absolute_hours          integer NOT NULL DEFAULT 24,
  lockout_threshold               integer NOT NULL DEFAULT 10,
  lockout_minutes                 integer NOT NULL DEFAULT 15,
  smtp_host                       text,
  smtp_port                       integer,
  smtp_username                   text,
  smtp_password_encrypted         bytea,
  smtp_use_starttls               boolean NOT NULL DEFAULT true,
  smtp_from_address               text,
  smtp_reply_to                   text,
  updated_at                      timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT instance_settings_singleton_check CHECK (id),
  -- m7: non-negative / positive guards on duration/cap/threshold policy fields.
  -- (smtp_port range and hex-color format CHECKs are deferred hardening.)
  CONSTRAINT instance_settings_checkin_window_check     CHECK (checkin_window_minutes >= 0),
  CONSTRAINT instance_settings_checkin_grace_check       CHECK (checkin_grace_minutes >= 0),
  CONSTRAINT instance_settings_booking_horizon_check     CHECK (booking_horizon_days >= 0),
  CONSTRAINT instance_settings_audit_retention_check     CHECK (audit_retention_days >= 0),
  CONSTRAINT instance_settings_booking_retention_check    CHECK (booking_retention_days >= 0),
  CONSTRAINT instance_settings_slot_granularity_check     CHECK (default_slot_granularity_minutes > 0),
  CONSTRAINT instance_settings_max_advance_check          CHECK (default_max_advance_days >= 0),
  CONSTRAINT instance_settings_min_password_length_check  CHECK (min_password_length > 0),
  CONSTRAINT instance_settings_session_idle_check         CHECK (session_idle_minutes > 0),
  CONSTRAINT instance_settings_session_absolute_check      CHECK (session_absolute_hours > 0),
  CONSTRAINT instance_settings_lockout_threshold_check     CHECK (lockout_threshold >= 0),
  CONSTRAINT instance_settings_lockout_minutes_check       CHECK (lockout_minutes >= 0)
);
CREATE TRIGGER instance_settings_set_updated_at BEFORE UPDATE ON instance_settings
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE oidc_role_mappings (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  provider_id     uuid NOT NULL REFERENCES oidc_providers(id) ON DELETE CASCADE,
  external_value  text NOT NULL,
  role_id         uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  organization_id uuid REFERENCES organizations(id) ON DELETE CASCADE,
  team_id         uuid REFERENCES teams(id) ON DELETE CASCADE,
  created_at      timestamptz NOT NULL DEFAULT now(),
  -- M9: a mapping's (team, org) pair must be a real team-in-that-org.
  CONSTRAINT oidc_role_mappings_team_org_fk
    FOREIGN KEY (team_id, organization_id) REFERENCES teams (id, organization_id)
);
-- m11: a plain 5-column UNIQUE treats NULLs as distinct, so NULL-scoped duplicates
-- slip through. COALESCE-expression unique indexes (mirroring how memberships
-- handled it) actually reject them.
CREATE UNIQUE INDEX oidc_role_mappings_scope_uq ON oidc_role_mappings (
  provider_id,
  external_value,
  role_id,
  COALESCE(organization_id, '00000000-0000-0000-0000-000000000000'::uuid),
  COALESCE(team_id,         '00000000-0000-0000-0000-000000000000'::uuid)
);
CREATE INDEX oidc_role_mappings_provider_idx ON oidc_role_mappings (provider_id);

CREATE TABLE data_subject_requests (
  id                uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id           uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  kind              dsr_kind NOT NULL,
  status            dsr_status NOT NULL DEFAULT 'received',
  requested_by      uuid REFERENCES users(id) ON DELETE SET NULL,
  export_asset_id   uuid REFERENCES assets(id) ON DELETE SET NULL,
  notes             text,
  identity_verified boolean NOT NULL DEFAULT false,
  requested_at      timestamptz NOT NULL DEFAULT now(),
  due_at            timestamptz,
  completed_at      timestamptz
);

CREATE TABLE import_jobs (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  kind            import_kind NOT NULL,
  status          import_status NOT NULL DEFAULT 'pending',
  source_asset_id uuid REFERENCES assets(id) ON DELETE SET NULL,
  total_rows      integer,
  processed_rows  integer NOT NULL DEFAULT 0,
  error_rows      integer NOT NULL DEFAULT 0,
  errors          jsonb NOT NULL DEFAULT '[]'::jsonb,
  created_by      uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at      timestamptz NOT NULL DEFAULT now(),
  completed_at    timestamptz
);

-- M5: native RANGE partitioning by occurred_at so retention is enforced by the
-- owner DROPping whole partitions (DDL), never row-level DELETE -- which the
-- immutability trigger/REVOKE forbid. This resolves the append-only vs retention
-- contradiction. The partition key must be part of every unique key, so the PK is
-- (id, occurred_at).
-- M12: audit_log.metadata MUST store only IDs, never names/emails, so anonymising
-- the users row suffices for erasure and no redaction of this append-only table is
-- needed. Enforced as an app rule (free-text shape is not statically checkable here).
CREATE TABLE audit_log (
  id                  uuid NOT NULL DEFAULT uuidv7(),
  occurred_at         timestamptz NOT NULL DEFAULT now(),
  actor_kind          actor_kind NOT NULL,
  actor_user_id       uuid REFERENCES users(id) ON DELETE RESTRICT,
  on_behalf_of_user_id uuid REFERENCES users(id) ON DELETE RESTRICT,
  api_key_id          uuid REFERENCES api_keys(id) ON DELETE SET NULL,
  action              text NOT NULL,
  outcome             audit_outcome NOT NULL DEFAULT 'success',  -- m3: native enum
  target_type         text,
  target_id           uuid,
  ip                  inet,
  user_agent          text,
  request_id          uuid,
  metadata            jsonb NOT NULL DEFAULT '{}'::jsonb,
  prev_hash           bytea,
  entry_hash          bytea,
  PRIMARY KEY (id, occurred_at)
) PARTITION BY RANGE (occurred_at);
-- Default partition holds everything until the owner provisions explicit time-range
-- partitions (and drops aged ones per audit_retention_days). DROP PARTITION is the
-- retention mechanism -- never row DELETE.
CREATE TABLE audit_log_default PARTITION OF audit_log DEFAULT;
CREATE INDEX audit_log_time_brin  ON audit_log USING brin (occurred_at);
CREATE INDEX audit_log_actor_idx  ON audit_log (actor_user_id);
CREATE INDEX audit_log_target_idx ON audit_log (target_type, target_id);

-- Immutability: block any UPDATE/DELETE/TRUNCATE.
CREATE FUNCTION audit_log_immutable() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'audit_log is append-only';
END $$;
CREATE TRIGGER audit_log_no_change
  BEFORE UPDATE OR DELETE ON audit_log
  FOR EACH ROW EXECUTE FUNCTION audit_log_immutable();
-- M5: row triggers do NOT fire on TRUNCATE; a statement-level BEFORE TRUNCATE
-- trigger closes that hole. Defined on the partitioned parent.
CREATE TRIGGER audit_log_no_truncate
  BEFORE TRUNCATE ON audit_log
  FOR EACH STATEMENT EXECUTE FUNCTION audit_log_immutable();
-- M5: defence in depth -- the runtime role must also be denied UPDATE/DELETE/
-- TRUNCATE at the grant level. The runtime role does not exist until P20, so this
-- is kept as a comment to apply then (the trigger is the active guard until):
--   REVOKE UPDATE, DELETE, TRUNCATE ON audit_log FROM <runtime_role>;
