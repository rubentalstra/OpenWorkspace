-- Server-side session store for the first-party `auth::PgSessionStore`
-- (tower-sessions SessionStore over the workspace's sqlx 0.9 pool). The schema
-- matches the historical tower-sessions-sqlx-store layout (schema "tower_sessions",
-- table "session"; id text PK, data bytea = rmp_serde(Record), expiry_date
-- timestamptz) so previously-written rows remain readable. Owned by this migrator
-- (advisory-locked, reversible), not created at runtime.

CREATE SCHEMA IF NOT EXISTS tower_sessions;

CREATE TABLE tower_sessions.session (
  id          text PRIMARY KEY NOT NULL,
  data        bytea NOT NULL,
  expiry_date timestamptz NOT NULL
);
