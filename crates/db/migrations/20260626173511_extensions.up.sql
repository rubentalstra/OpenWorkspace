-- Required PostgreSQL extensions. btree_gist backs the EXCLUDE USING gist
-- no-double-booking constraint (P2); pg_trgm for fuzzy search; citext for
-- case-insensitive text (emails). Requires the owner role to CREATE EXTENSION.
CREATE EXTENSION IF NOT EXISTS btree_gist;
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS citext;
