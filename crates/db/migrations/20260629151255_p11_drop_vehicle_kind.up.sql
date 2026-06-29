-- Drop the unsupported 'vehicle' resource kind. Postgres can't remove an enum
-- value in place, so recreate the type without it. No 'vehicle' resources exist
-- (the type was never delivered), so the column re-cast is safe.
ALTER TYPE resource_kind RENAME TO resource_kind__old;
CREATE TYPE resource_kind AS ENUM ('desk', 'room', 'parking', 'equipment');
ALTER TABLE resources
  ALTER COLUMN kind TYPE resource_kind USING (kind::text::resource_kind);
DROP TYPE resource_kind__old;
