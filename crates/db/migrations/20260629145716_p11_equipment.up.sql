-- P11: a simple, reusable equipment catalog. Each item is just a name; a resource
-- assigns items with a quantity. `resources.attributes` stays for other specifics.

CREATE TABLE equipment_items (
  id          uuid PRIMARY KEY DEFAULT uuidv7(),
  name        citext NOT NULL UNIQUE,          -- case-insensitive: one "24\" Monitor"
  description text,
  created_at  timestamptz NOT NULL DEFAULT now(),
  updated_at  timestamptz NOT NULL DEFAULT now()
);
CREATE TRIGGER equipment_items_set_updated_at BEFORE UPDATE ON equipment_items
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE resource_equipment (                -- which catalog items a resource has
  resource_id       uuid NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
  equipment_item_id uuid NOT NULL REFERENCES equipment_items(id) ON DELETE RESTRICT,
  quantity          integer NOT NULL DEFAULT 1 CHECK (quantity > 0),
  created_at        timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (resource_id, equipment_item_id)
);
CREATE INDEX resource_equipment_item_idx ON resource_equipment (equipment_item_id);

-- Runtime-role least privilege (P8). The P8 migration's ALTER DEFAULT PRIVILEGES
-- already auto-grants owner-created tables to openworkspace_app; these explicit
-- grants document the runtime role's DML access on the new tables. No RLS: the
-- equipment catalog is global and floor writes run under the elevated app context.
GRANT SELECT, INSERT, UPDATE, DELETE ON equipment_items, resource_equipment
  TO openworkspace_app;
