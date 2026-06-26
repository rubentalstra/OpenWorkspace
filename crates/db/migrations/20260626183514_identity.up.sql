-- Identity and credentials.
-- users is created WITHOUT its cycle-closing FKs (home_zone_id -> locations,
-- default_category_id -> resource_categories, invited_by -> users); those are
-- added in the cycle_fks migration once the referenced tables exist.
-- oidc_providers' default_role_id / default_organization_id FKs are likewise
-- deferred to cycle_fks (roles/organizations live in org_location_resource).

CREATE TABLE crypto_keys (
  id           uuid PRIMARY KEY DEFAULT uuidv7(),
  purpose      text NOT NULL,
  suite_id     text NOT NULL,
  wrapped_key  bytea NOT NULL,
  kek_label    text,
  active       boolean NOT NULL DEFAULT true,
  created_at   timestamptz NOT NULL DEFAULT now(),
  retired_at   timestamptz
);
CREATE UNIQUE INDEX crypto_keys_active_purpose ON crypto_keys (purpose) WHERE active;

CREATE TABLE users (
  id                    uuid PRIMARY KEY DEFAULT uuidv7(),
  email                 citext UNIQUE NOT NULL,
  display_name          text NOT NULL,
  status                user_status NOT NULL DEFAULT 'active',
  locale                text NOT NULL DEFAULT 'en',
  timezone              text NOT NULL DEFAULT 'Europe/Amsterdam',
  default_view          user_default_view NOT NULL DEFAULT 'map',
  home_zone_id          uuid,             -- FK -> locations(id) added in cycle_fks
  default_category_id   uuid,             -- FK -> resource_categories(id) added in cycle_fks
  webauthn_user_handle  bytea UNIQUE NOT NULL,
  is_instance_admin     boolean NOT NULL DEFAULT false,
  email_verified_at     timestamptz,
  last_login_at         timestamptz,
  failed_login_count    integer NOT NULL DEFAULT 0,
  locked_until          timestamptz,
  invited_by            uuid,             -- FK -> users(id) added in cycle_fks
  notification_prefs    jsonb NOT NULL DEFAULT '{}'::jsonb,
  anonymized_at         timestamptz,
  created_at            timestamptz NOT NULL DEFAULT now(),
  updated_at            timestamptz NOT NULL DEFAULT now()
  -- status / default_view are native enums (m3); CHECK constraints no longer needed.
);
CREATE TRIGGER users_set_updated_at BEFORE UPDATE ON users
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE password_credentials (
  user_id              uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  password_hash        text NOT NULL,
  must_change          boolean NOT NULL DEFAULT false,
  password_changed_at  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE oidc_providers (
  id                          uuid PRIMARY KEY DEFAULT uuidv7(),
  display_name                text NOT NULL,
  slug                        citext UNIQUE NOT NULL,
  issuer_url                  text NOT NULL,
  use_discovery               boolean NOT NULL DEFAULT true,
  authorization_endpoint      text,
  token_endpoint              text,
  userinfo_endpoint           text,
  jwks_uri                    text,
  end_session_endpoint        text,
  metadata_cache_seconds      integer NOT NULL DEFAULT 3600,
  client_id                   text NOT NULL,
  client_auth_method          text NOT NULL DEFAULT 'client_secret_basic',
  client_secret_encrypted     bytea,
  client_assertion_key_id     uuid REFERENCES crypto_keys(id) ON DELETE RESTRICT,
  scopes                      text[] NOT NULL DEFAULT ARRAY['openid','email','profile'],
  response_mode               text NOT NULL DEFAULT 'query',
  prompt                      text,
  acr_values                  text[],
  max_age_seconds             integer,
  id_token_signed_response_alg text[] NOT NULL DEFAULT ARRAY['RS256'],
  clock_skew_seconds          integer NOT NULL DEFAULT 60,
  email_claim                 text NOT NULL DEFAULT 'email',
  email_verified_claim        text NOT NULL DEFAULT 'email_verified',
  name_claim                  text NOT NULL DEFAULT 'name',
  username_claim              text NOT NULL DEFAULT 'preferred_username',
  groups_claim                text,
  enabled                     boolean NOT NULL DEFAULT true,
  jit_provisioning            boolean NOT NULL DEFAULT true,
  default_role_id             uuid,        -- FK -> roles(id) added in cycle_fks
  default_organization_id     uuid,        -- FK -> organizations(id) added in cycle_fks
  allowed_email_domains       text[] NOT NULL DEFAULT '{}',
  account_linking             text NOT NULL DEFAULT 'verified_email',
  update_profile_on_login     boolean NOT NULL DEFAULT true,
  sync_roles_on_login         boolean NOT NULL DEFAULT true,
  rp_initiated_logout         boolean NOT NULL DEFAULT true,
  button_label                text,
  icon                        text,
  sort_order                  integer NOT NULL DEFAULT 0,
  created_at                  timestamptz NOT NULL DEFAULT now(),
  updated_at                  timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT oidc_providers_client_auth_method_check
    CHECK (client_auth_method IN ('client_secret_basic','client_secret_post','client_secret_jwt','private_key_jwt','none')),
  CONSTRAINT oidc_providers_response_mode_check
    CHECK (response_mode IN ('query','form_post')),
  CONSTRAINT oidc_providers_account_linking_check
    CHECK (account_linking IN ('disabled','verified_email')),
  CONSTRAINT oidc_providers_discovery_check
    CHECK (use_discovery OR (authorization_endpoint IS NOT NULL AND token_endpoint IS NOT NULL AND jwks_uri IS NOT NULL)),
  CONSTRAINT oidc_providers_client_auth_material_check
    CHECK ((client_auth_method LIKE 'client_secret_%' AND client_secret_encrypted IS NOT NULL)
        OR (client_auth_method = 'private_key_jwt' AND client_assertion_key_id IS NOT NULL)
        OR (client_auth_method = 'none'))
);
CREATE TRIGGER oidc_providers_set_updated_at BEFORE UPDATE ON oidc_providers
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE oidc_identities (
  id            uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id       uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider_id   uuid NOT NULL REFERENCES oidc_providers(id) ON DELETE RESTRICT,
  subject       text NOT NULL,
  email_at_link citext,
  last_login_at timestamptz,
  created_at    timestamptz NOT NULL DEFAULT now(),
  UNIQUE (provider_id, subject)
);

CREATE TABLE passkeys (
  id               uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id          uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  credential_id    bytea UNIQUE NOT NULL,
  passkey          jsonb NOT NULL,
  sign_count       bigint NOT NULL DEFAULT 0,
  aaguid           uuid,
  transports       text[],
  label            text,
  backup_eligible  boolean NOT NULL DEFAULT false,
  backup_state     boolean NOT NULL DEFAULT false,
  created_at       timestamptz NOT NULL DEFAULT now(),
  last_used_at     timestamptz
);
CREATE INDEX passkeys_user_idx ON passkeys (user_id);

CREATE TABLE totp_credentials (
  user_id          uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  secret_encrypted bytea NOT NULL,
  digits           smallint NOT NULL DEFAULT 6,
  period_seconds   smallint NOT NULL DEFAULT 30,
  algorithm        text NOT NULL DEFAULT 'SHA1',
  confirmed_at     timestamptz,
  created_at       timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE recovery_codes (
  id         uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  code_hash  bytea NOT NULL,
  used_at    timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX recovery_codes_user_idx ON recovery_codes (user_id);

CREATE TABLE user_tokens (
  id          uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  kind        token_kind NOT NULL,
  token_hash  bytea UNIQUE NOT NULL,
  expires_at  timestamptz NOT NULL,
  used_at     timestamptz,
  created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX user_tokens_user_idx ON user_tokens (user_id);

CREATE TABLE api_keys (
  id                    uuid PRIMARY KEY DEFAULT uuidv7(),
  owner_user_id         uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name                  text NOT NULL,
  prefix                text NOT NULL,
  token_hash            bytea UNIQUE NOT NULL,
  scopes                text[] NOT NULL DEFAULT '{}',
  expires_at            timestamptz,
  last_used_at          timestamptz,
  last_used_ip          inet,
  rate_limit_per_minute integer,
  revoked_at            timestamptz,
  revoked_by            uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at            timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX api_keys_owner_idx  ON api_keys (owner_user_id);
CREATE INDEX api_keys_prefix_idx ON api_keys (prefix);
