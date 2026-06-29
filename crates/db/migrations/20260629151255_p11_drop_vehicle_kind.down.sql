-- Restore the 'vehicle' resource kind.
ALTER TYPE resource_kind RENAME TO resource_kind__new;
CREATE TYPE resource_kind AS ENUM ('desk', 'room', 'parking', 'vehicle', 'equipment');
ALTER TABLE resources
  ALTER COLUMN kind TYPE resource_kind USING (kind::text::resource_kind);
DROP TYPE resource_kind__new;
