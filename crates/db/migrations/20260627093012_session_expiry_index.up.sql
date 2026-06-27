-- Index supporting the expired-session reaper's `expiry_date < now()` sweep and
-- the per-load `expiry_date > now()` filter. Without it those scans would walk
-- the whole `tower_sessions.session` table.

CREATE INDEX idx_session_expiry_date
  ON tower_sessions.session (expiry_date);
