//! PostgreSQL access and migrations (sqlx), behind a thin facade.

mod bookings;
mod credentials;

pub use bookings::{
    Booking, BookingSourceRow, BookingStatusRow, BookingVisibilityRow, CreatedBooking, NewBooking,
    OccurrenceKindRow, apply_transition, auto_release, cancel, check_in, check_out, create_booking,
};
pub use credentials::{
    CredentialRow, UserStatusRow, change_password, insert_bootstrap_admin, instance_admin_exists,
    load_credential_by_email, load_credential_by_id, touch_last_login, update_password_hash,
};

use secrecy::{ExposeSecret, SecretString};
use sqlx::error::ErrorKind;
use sqlx::postgres::PgPoolOptions;

/// Connection pool handle. Clone-cheap (`Arc` inside); share into app state and the worker.
pub type Db = sqlx::PgPool;

/// Embedded, reversible migrations. Path is relative to this crate's root.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Database facade errors.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// A booking exclusion-constraint violation (Postgres `23P01`). The HTTP layer
    /// maps this to 409 Conflict — the single chokepoint for no-double-booking.
    #[error("resource is already booked for that time")]
    Conflict,
    /// A serialization failure (`40001`); the operation may be retried.
    #[error("transaction conflict; retry")]
    Retryable,
    /// `create_booking` was called with an empty expansion (zero occurrences),
    /// which would persist an orphan header. Rejected before any write.
    #[error("cannot create a booking with zero occurrences")]
    EmptyExpansion,
    /// A structurally invalid occurrence period (empty or inverted, i.e. not
    /// `start < end`) was supplied. Rejected before any write.
    #[error("occurrence period is empty or inverted")]
    InvalidPeriod,
    /// A transition was applied to an occurrence whose status no longer matches
    /// the expected from-status (a lost update / stale read). The caller should
    /// re-read and recompute the transition.
    #[error("occurrence is in an unexpected state; re-read and retry")]
    StaleState,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Connect to PostgreSQL and build a pool.
pub async fn connect(url: &SecretString, max_connections: u32) -> Result<Db, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(url.expose_secret())
        .await?;
    Ok(pool)
}

/// Apply all pending migrations. Advisory-locked, so concurrent web replicas
/// starting together cannot race; idempotent.
pub async fn run_migrations(db: &Db) -> Result<(), DbError> {
    MIGRATOR.run(db).await?;
    Ok(())
}

/// Readiness check: `Ok(())` if the database answers a trivial query.
pub async fn ping(db: &Db) -> Result<(), DbError> {
    sqlx::query("SELECT 1").execute(db).await?;
    Ok(())
}

/// Classify a raw sqlx error into a typed [`DbError`], mapping the cases the
/// application reacts to: an exclusion-constraint violation (the no-double-booking
/// guarantee) becomes [`DbError::Conflict`]; a serialization failure becomes
/// [`DbError::Retryable`]. Used at query call sites.
#[must_use]
pub fn classify(err: sqlx::Error) -> DbError {
    if let Some(db_err) = err.as_database_error() {
        if matches!(db_err.kind(), ErrorKind::ExclusionViolation) {
            return DbError::Conflict;
        }
        match db_err.code().as_deref() {
            // Serialization failure: retryable.
            Some("40001") => return DbError::Retryable,
            // check_violation — e.g. an empty/inverted period reaching the
            // booking_occurrences_period_bounded_check. Defense in depth: the
            // write paths reject malformed periods structurally before BEGIN.
            Some("23514") => return DbError::InvalidPeriod,
            // unique_violation — e.g. a duplicate (booking_id, recurrence_id) on
            // booking_occurrences_instance_uq. Defense in depth: expand_series
            // dedups identical UTC start instants before persistence.
            Some("23505") => return DbError::Conflict,
            _ => {}
        }
    }
    DbError::Sqlx(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone as _, Utc};
    use proptest::prelude::*;
    use sqlx::postgres::types::PgRange;
    use uuid::Uuid;

    #[sqlx::test(migrations = false)]
    async fn migrations_apply_with_extensions_then_revert(pool: Db) -> Result<(), DbError> {
        run_migrations(&pool).await?;

        let extensions: Vec<String> = sqlx::query_scalar("SELECT extname FROM pg_extension")
            .fetch_all(&pool)
            .await?;
        for required in ["btree_gist", "pg_trgm", "citext"] {
            assert!(
                extensions.iter().any(|e| e == required),
                "missing extension {required}"
            );
        }

        // Roll all migrations back down to version 0.
        MIGRATOR.undo(&pool, 0).await?;
        Ok(())
    }

    /// The `session_expiry_index` migration must create `idx_session_expiry_date`
    /// on up, and its down must drop it (the reaper's `expiry_date < now()` would
    /// otherwise full-scan).
    #[sqlx::test(migrations = false)]
    async fn session_expiry_index_up_creates_and_down_drops(pool: Db) -> Result<(), DbError> {
        run_migrations(&pool).await?;
        let exists_after_up: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM pg_indexes \
             WHERE schemaname = 'tower_sessions' AND tablename = 'session' \
             AND indexname = 'idx_session_expiry_date')",
        )
        .fetch_one(&pool)
        .await?;
        assert!(exists_after_up, "up must create idx_session_expiry_date");

        // Revert just the index migration; the session table itself remains.
        MIGRATOR.undo(&pool, 20_260_627_073_831).await?;
        let exists_after_down: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM pg_indexes \
             WHERE schemaname = 'tower_sessions' AND tablename = 'session' \
             AND indexname = 'idx_session_expiry_date')",
        )
        .fetch_one(&pool)
        .await?;
        assert!(!exists_after_down, "down must drop idx_session_expiry_date");
        Ok(())
    }

    /// Seed the minimal FK chain (org → floor → desk) and return the resource id.
    async fn seed_resource(pool: &Db) -> Uuid {
        let tag = Uuid::new_v4().simple().to_string();
        let org: Uuid = sqlx::query_scalar(
            "INSERT INTO organizations (name, slug) VALUES ($1, $1) RETURNING id",
        )
        .bind(&tag)
        .fetch_one(pool)
        .await
        .unwrap();
        let location: Uuid = sqlx::query_scalar(
            "INSERT INTO locations (kind, name, path, depth, organization_id) \
             VALUES ('floor', 'Floor 1', '/f1', 0, $1) RETURNING id",
        )
        .bind(org)
        .fetch_one(pool)
        .await
        .unwrap();
        sqlx::query_scalar(
            "INSERT INTO resources (location_id, kind, name) VALUES ($1, 'desk', 'Desk 1') RETURNING id",
        )
        .bind(location)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    /// A half-open `[start, end)` period on 2026-07-01 between the given hours.
    fn period(start_hour: u32, end_hour: u32) -> PgRange<DateTime<Utc>> {
        let start = Utc.with_ymd_and_hms(2026, 7, 1, start_hour, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 7, 1, end_hour, 0, 0).unwrap();
        PgRange::from(start..end)
    }

    /// Insert a system block (a `blackout` occurrence with no parent booking) — exercises
    /// the no-double-booking exclusion constraint and the unified-block guarantee.
    async fn insert_block(
        pool: &Db,
        resource: Uuid,
        period: PgRange<DateTime<Utc>>,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO booking_occurrences (occurrence_kind, resource_id, period, status) \
             VALUES ('blackout', $1, $2, 'booked')",
        )
        .bind(resource)
        .bind(period)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(classify)
    }

    #[sqlx::test]
    async fn overlapping_periods_rejected_as_conflict(pool: Db) {
        let resource = seed_resource(&pool).await;
        insert_block(&pool, resource, period(9, 11)).await.unwrap();
        let err = insert_block(&pool, resource, period(10, 12))
            .await
            .unwrap_err();
        assert!(
            matches!(err, DbError::Conflict),
            "expected Conflict, got {err:?}"
        );
    }

    #[sqlx::test]
    async fn adjacent_periods_allowed(pool: Db) {
        let resource = seed_resource(&pool).await;
        insert_block(&pool, resource, period(9, 10)).await.unwrap();
        // Half-open [9,10) and [10,11) touch but do not overlap.
        insert_block(&pool, resource, period(10, 11)).await.unwrap();
    }

    #[sqlx::test]
    async fn different_resources_do_not_conflict(pool: Db) {
        let a = seed_resource(&pool).await;
        let b = seed_resource(&pool).await;
        insert_block(&pool, a, period(9, 11)).await.unwrap();
        insert_block(&pool, b, period(9, 11)).await.unwrap();
    }

    /// The pure half-open overlap predicate the exclusion constraint enforces.
    fn periods_overlap(a: (i64, i64), b: (i64, i64)) -> bool {
        a.0 < b.1 && b.0 < a.1
    }

    proptest! {
        #[test]
        fn overlap_predicate_is_halfopen_and_symmetric(
            a0 in 0i64..1_000, len_a in 1i64..500, b0 in 0i64..1_000, len_b in 1i64..500,
        ) {
            let a = (a0, a0 + len_a);
            let b = (b0, b0 + len_b);
            prop_assert_eq!(periods_overlap(a, b), periods_overlap(b, a));
            // Touching ranges [x,y)+[y,z) never overlap (half-open).
            prop_assert!(!periods_overlap(a, (a.1, a.1 + 5)));
        }
    }
}
