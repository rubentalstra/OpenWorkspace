-- Bookings and their materialised occurrences. booking_occurrences carries the
-- canonical no-double-booking GiST exclusion constraint over (resource_id, period)
-- using half-open [start, end) tstzrange ranges; a clashing insert raises 23P01 ->
-- HTTP 409.
--
-- B4: booking_occurrences is the SINGLE blocking table for user bookings AND
-- system blocks. User bookings have booking_id NOT NULL and occurrence_kind
-- 'booking'. System blocks (permanent_assignments, blackouts) are materialised by
-- the 02:00 worker (a later phase) as rows with booking_id NULL and
-- occurrence_kind <> 'booking'. The permanent_assignments and blackouts tables
-- (in org_location_resource) remain the source of truth; this table is what the
-- single GiST constraint governs so all three block uniformly.

CREATE TABLE bookings (
  id                    uuid PRIMARY KEY DEFAULT uuidv7(),
  resource_id           uuid NOT NULL REFERENCES resources(id) ON DELETE RESTRICT,
  booked_for_user_id    uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  booked_by_user_id     uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  title                 text,
  description           text,
  -- B1: per-series appointment visibility for the read-path authorization filter.
  visibility            booking_visibility NOT NULL DEFAULT 'public',
  source                booking_source NOT NULL DEFAULT 'web',  -- m3: native enum
  ical_uid              text UNIQUE NOT NULL,
  sequence              integer NOT NULL DEFAULT 0,
  dtstamp               timestamptz NOT NULL DEFAULT now(),
  recurrence_rule       text,
  recurrence_until      timestamptz,
  -- M3: IANA series time zone (TZID) for DST-correct RRULE re-expansion; validated
  -- against pg_timezone_names so re-materialisation cannot drift local times.
  series_timezone       text NOT NULL,
  -- M8: THISANDFUTURE splits link the new (future) series back to its origin.
  split_from_booking_id uuid REFERENCES bookings(id) ON DELETE SET NULL,
  status                booking_status NOT NULL DEFAULT 'booked',
  cancelled_at          timestamptz,
  cancelled_by          uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at            timestamptz NOT NULL DEFAULT now(),
  updated_at            timestamptz NOT NULL DEFAULT now(),
  -- m6: recurrence_until is a derived materialisation cap; the RRULE is the source
  -- of truth, so it cannot exist without one.
  CONSTRAINT bookings_recurrence_until_check
    CHECK (recurrence_until IS NULL OR recurrence_rule IS NOT NULL),
  -- M2: lets booking_occurrences anchor a composite (booking_id, resource_id) FK
  -- so an occurrence can never drift to a different resource than its parent.
  CONSTRAINT bookings_id_resource_uq UNIQUE (id, resource_id)
);
-- M3: reject unknown time zones at write time. A subquery cannot live in a CHECK,
-- so series_timezone (IANA TZID) is validated by a trigger against pg_timezone_names.
CREATE FUNCTION bookings_validate_series_timezone() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_timezone_names WHERE name = NEW.series_timezone) THEN
    RAISE EXCEPTION 'invalid series_timezone: %', NEW.series_timezone
      USING ERRCODE = 'check_violation';
  END IF;
  RETURN NEW;
END $$;
CREATE TRIGGER bookings_series_timezone_check BEFORE INSERT OR UPDATE ON bookings
  FOR EACH ROW EXECUTE FUNCTION bookings_validate_series_timezone();
CREATE TRIGGER bookings_set_updated_at BEFORE UPDATE ON bookings
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX bookings_for_idx      ON bookings (booked_for_user_id);
CREATE INDEX bookings_resource_idx ON bookings (resource_id);
CREATE INDEX bookings_by_idx       ON bookings (booked_by_user_id);  -- M9: delegate/audit queries

-- M8: recurrence exceptions distinct from a per-row cancel. exception_kind
-- distinguishes a platform EXDATE removal, an attendee cancel, and a moved
-- instance, so re-materialisation reconstructs exceptions deterministically.
CREATE TABLE booking_recurrence_overrides (
  id             uuid PRIMARY KEY DEFAULT uuidv7(),
  booking_id     uuid NOT NULL REFERENCES bookings(id) ON DELETE CASCADE,
  recurrence_id  timestamptz NOT NULL,
  exception_kind text NOT NULL,
  created_at     timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT booking_recurrence_overrides_kind_check
    CHECK (exception_kind IN ('moved','cancelled','exdate')),
  UNIQUE (booking_id, recurrence_id)
);

CREATE TABLE booking_occurrences (
  id                  uuid PRIMARY KEY DEFAULT uuidv7(),
  -- B4: nullable. System blocks (occurrence_kind <> 'booking') have no parent booking.
  booking_id          uuid,
  -- B4: which kind of block this row is.
  occurrence_kind     occurrence_kind NOT NULL DEFAULT 'booking',
  resource_id         uuid NOT NULL REFERENCES resources(id) ON DELETE RESTRICT,
  period              tstzrange NOT NULL,
  recurrence_id       timestamptz,
  is_override         boolean NOT NULL DEFAULT false,
  status              booking_status NOT NULL DEFAULT 'booked',
  check_in_at         timestamptz,
  checked_in_by       uuid REFERENCES users(id) ON DELETE SET NULL,
  checked_out_at      timestamptz,
  auto_released_at    timestamptz,
  cancelled_at        timestamptz,
  cancellation_reason text,
  created_at          timestamptz NOT NULL DEFAULT now(),
  updated_at          timestamptz NOT NULL DEFAULT now(),
  -- B3: pin canonical half-open bounds; the whole no-overlap correctness rests on
  -- ranges being [start, end). Rejects '[]' / '()' bounds outright.
  CONSTRAINT booking_occurrences_period_bounded_check
    CHECK (NOT isempty(period) AND lower(period) IS NOT NULL AND upper(period) IS NOT NULL),
  CONSTRAINT booking_occurrences_period_halfopen_check
    CHECK (period = tstzrange(lower(period), upper(period), '[)')),
  -- B4: occurrence_kind = 'booking' iff there is a parent booking.
  CONSTRAINT booking_occurrences_kind_booking_check
    CHECK ((occurrence_kind = 'booking') = (booking_id IS NOT NULL)),
  -- M2: composite FK so resource_id can never drift from the parent booking. It is
  -- naturally not enforced for system blocks where booking_id IS NULL (MATCH SIMPLE).
  CONSTRAINT booking_occurrences_booking_resource_fk
    FOREIGN KEY (booking_id, resource_id) REFERENCES bookings (id, resource_id) ON DELETE CASCADE,
  -- M4: terminal/active statuses must carry their timestamp; correctness no longer
  -- depends on app discipline alone.
  CONSTRAINT booking_occurrences_checked_in_at_check
    CHECK (status <> 'checked_in'  OR check_in_at      IS NOT NULL),
  CONSTRAINT booking_occurrences_checked_out_at_check
    CHECK (status <> 'checked_out' OR checked_out_at   IS NOT NULL),
  CONSTRAINT booking_occurrences_cancelled_at_check
    CHECK (status <> 'cancelled'   OR cancelled_at     IS NOT NULL),
  CONSTRAINT booking_occurrences_released_at_check
    CHECK (status <> 'released'    OR auto_released_at IS NOT NULL),
  -- B4: the single no-double-booking guarantee. A system block (occurrence_kind
  -- <> 'booking') always blocks; a user booking blocks only while live. So a
  -- blackout/permanent-assignment row and a user booking on the same resource
  -- cannot overlap.
  CONSTRAINT booking_occurrences_no_double_booking
    EXCLUDE USING gist (resource_id WITH =, period WITH &&)
    WHERE (occurrence_kind <> 'booking' OR status IN ('booked','checked_in'))
);
CREATE TRIGGER booking_occurrences_set_updated_at BEFORE UPDATE ON booking_occurrences
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX booking_occurrences_booking_idx ON booking_occurrences (booking_id);
CREATE INDEX booking_occurrences_live_idx    ON booking_occurrences (period)
  WHERE (occurrence_kind <> 'booking' OR status IN ('booked','checked_in'));
-- M1: idempotent materialiser. One occurrence per (booking, instance) for
-- recurring series; one per booking for single bookings (recurrence_id NULL).
CREATE UNIQUE INDEX booking_occurrences_instance_uq
  ON booking_occurrences (booking_id, recurrence_id)
  WHERE booking_id IS NOT NULL AND recurrence_id IS NOT NULL;
CREATE UNIQUE INDEX booking_occurrences_single_uq
  ON booking_occurrences (booking_id)
  WHERE booking_id IS NOT NULL AND recurrence_id IS NULL;
