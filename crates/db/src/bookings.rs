//! Booking persistence on top of the I/O-free `domain` engine.
//!
//! This module owns the `sqlx`-mapped row enums (the `domain` crate stays
//! `sqlx`-free) and the transactional write paths:
//!
//! - [`create_booking`] materialises a `bookings` header plus one
//!   `booking_occurrences` row per expanded instant, inside a single
//!   transaction wrapped in a bounded serialization-failure retry.
//! - [`check_in`], [`check_out`], [`cancel`] and [`auto_release`] apply a
//!   [`domain::state_machine::Transition`] to a single occurrence.
//!
//! The no-double-booking `GiST` exclusion constraint (`23P01`) surfaces as
//! [`DbError::Conflict`]; a `40001` serialization failure surfaces as
//! [`DbError::Retryable`] and is retried internally.

use chrono::{DateTime, Utc};
use domain::state_machine::{Stamp, Transition};
use domain::{
    BookingSource, BookingStatus, BookingVisibility, OccurrenceId, OccurrenceKind, TimeRange,
};
use sqlx::postgres::types::PgRange;
use uuid::Uuid;

use crate::{Db, DbError, classify};

/// How many times the whole `begin..commit` is retried when COMMIT raises a
/// `40001` serialization failure.
const MAX_TX_ATTEMPTS: u32 = 3;

// --- Row enums (sqlx-mapped; `domain` owns the plain ones) ---------------

/// Persistence-mapped mirror of [`domain::BookingStatus`] (`booking_status`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "booking_status", rename_all = "snake_case")]
pub enum BookingStatusRow {
    /// Reserved but not yet checked in.
    Booked,
    /// Checked in; the resource is actively occupied.
    CheckedIn,
    /// Checked out; the tail of the period is freed.
    CheckedOut,
    /// Auto-released after the grace window elapsed without check-in.
    Released,
    /// Marked as a no-show.
    NoShow,
    /// Cancelled before becoming active.
    Cancelled,
}

impl From<BookingStatus> for BookingStatusRow {
    fn from(value: BookingStatus) -> Self {
        match value {
            BookingStatus::Booked => Self::Booked,
            BookingStatus::CheckedIn => Self::CheckedIn,
            BookingStatus::CheckedOut => Self::CheckedOut,
            BookingStatus::Released => Self::Released,
            BookingStatus::NoShow => Self::NoShow,
            BookingStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<BookingStatusRow> for BookingStatus {
    fn from(value: BookingStatusRow) -> Self {
        match value {
            BookingStatusRow::Booked => Self::Booked,
            BookingStatusRow::CheckedIn => Self::CheckedIn,
            BookingStatusRow::CheckedOut => Self::CheckedOut,
            BookingStatusRow::Released => Self::Released,
            BookingStatusRow::NoShow => Self::NoShow,
            BookingStatusRow::Cancelled => Self::Cancelled,
        }
    }
}

/// Persistence-mapped mirror of [`domain::OccurrenceKind`] (`occurrence_kind`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "occurrence_kind", rename_all = "snake_case")]
pub enum OccurrenceKindRow {
    /// A user booking (has a parent `bookings` row).
    Booking,
    /// A materialised permanent assignment block.
    PermanentAssignment,
    /// A materialised blackout block.
    Blackout,
}

impl From<OccurrenceKind> for OccurrenceKindRow {
    fn from(value: OccurrenceKind) -> Self {
        match value {
            OccurrenceKind::Booking => Self::Booking,
            OccurrenceKind::PermanentAssignment => Self::PermanentAssignment,
            OccurrenceKind::Blackout => Self::Blackout,
        }
    }
}

impl From<OccurrenceKindRow> for OccurrenceKind {
    fn from(value: OccurrenceKindRow) -> Self {
        match value {
            OccurrenceKindRow::Booking => Self::Booking,
            OccurrenceKindRow::PermanentAssignment => Self::PermanentAssignment,
            OccurrenceKindRow::Blackout => Self::Blackout,
        }
    }
}

/// Persistence-mapped mirror of [`domain::BookingVisibility`]
/// (`booking_visibility`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "booking_visibility", rename_all = "snake_case")]
pub enum BookingVisibilityRow {
    /// Visible to everyone.
    Public,
    /// Visible within the owning organization.
    OrgVisible,
    /// Visible only to the booker / bookee.
    Private,
}

impl From<BookingVisibility> for BookingVisibilityRow {
    fn from(value: BookingVisibility) -> Self {
        match value {
            BookingVisibility::Public => Self::Public,
            BookingVisibility::OrgVisible => Self::OrgVisible,
            BookingVisibility::Private => Self::Private,
        }
    }
}

impl From<BookingVisibilityRow> for BookingVisibility {
    fn from(value: BookingVisibilityRow) -> Self {
        match value {
            BookingVisibilityRow::Public => Self::Public,
            BookingVisibilityRow::OrgVisible => Self::OrgVisible,
            BookingVisibilityRow::Private => Self::Private,
        }
    }
}

/// Persistence-mapped mirror of [`domain::BookingSource`] (`booking_source`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "booking_source", rename_all = "snake_case")]
pub enum BookingSourceRow {
    /// Created via the web UI.
    Web,
    /// Created via the public API.
    Api,
    /// Created by a bulk import.
    Import,
    /// Created by a delegate on behalf of a principal.
    Delegate,
}

impl From<BookingSource> for BookingSourceRow {
    fn from(value: BookingSource) -> Self {
        match value {
            BookingSource::Web => Self::Web,
            BookingSource::Api => Self::Api,
            BookingSource::Import => Self::Import,
            BookingSource::Delegate => Self::Delegate,
        }
    }
}

impl From<BookingSourceRow> for BookingSource {
    fn from(value: BookingSourceRow) -> Self {
        match value {
            BookingSourceRow::Web => Self::Web,
            BookingSourceRow::Api => Self::Api,
            BookingSourceRow::Import => Self::Import,
            BookingSourceRow::Delegate => Self::Delegate,
        }
    }
}

// --- Request + row structs ----------------------------------------------

/// The `bookings` header to insert. Field shapes mirror the columns the write
/// path populates; the recurrence fields are stored verbatim so the worker can
/// deterministically re-materialise the series later.
#[derive(Clone, Debug)]
pub struct NewBooking {
    /// Resource being booked.
    pub resource_id: Uuid,
    /// User the booking is for.
    pub booked_for_user_id: Uuid,
    /// User who created the booking (the delegate, if any).
    pub booked_by_user_id: Uuid,
    /// Optional human title.
    pub title: Option<String>,
    /// Optional human description.
    pub description: Option<String>,
    /// Per-series read-path visibility.
    pub visibility: BookingVisibility,
    /// Channel the booking was created through.
    pub source: BookingSource,
    /// Globally unique iCalendar UID for the series.
    pub ical_uid: String,
    /// `RRULE` text (`None`/empty for a single booking).
    pub recurrence_rule: Option<String>,
    /// Inclusive materialisation cap for the series.
    pub recurrence_until: Option<DateTime<Utc>>,
    /// IANA TZID the series is anchored in (validated by a DB trigger).
    pub series_timezone: String,
}

/// The inserted `bookings` header, with row-mapped enums converted back to the
/// plain `domain` enums at the boundary.
#[derive(Clone, Debug)]
pub struct Booking {
    /// Primary key.
    pub id: Uuid,
    /// Resource being booked.
    pub resource_id: Uuid,
    /// User the booking is for.
    pub booked_for_user_id: Uuid,
    /// User who created the booking.
    pub booked_by_user_id: Uuid,
    /// Per-series read-path visibility.
    pub visibility: BookingVisibility,
    /// Channel the booking was created through.
    pub source: BookingSource,
    /// iCalendar UID for the series.
    pub ical_uid: String,
    /// Header lifecycle status.
    pub status: BookingStatus,
}

/// A booking plus the occurrence ids materialised for it (one per expanded
/// instant, in input order).
#[derive(Clone, Debug)]
pub struct CreatedBooking {
    /// The inserted header.
    pub booking: Booking,
    /// Ids of the materialised `booking_occurrences`, in input order.
    pub occurrence_ids: Vec<OccurrenceId>,
}

// --- create_booking ------------------------------------------------------

/// Persists a booking header plus one occurrence per `expanded` instant, all in
/// one transaction.
///
/// `expanded` is the output of [`domain::recurrence::expand_series`] — a single
/// element for a non-recurring booking, or the materialised set for a series.
/// Each occurrence's `period` is stored as a half-open `[start, end)`
/// `tstzrange`, and `recurrence_id` is the UTC start instant (for a single
/// booking, where `expanded.len() == 1`, `recurrence_id` is left `NULL` so the
/// single-occurrence unique index applies).
///
/// The whole `begin..commit` is retried up to [`MAX_TX_ATTEMPTS`] times on a
/// serialization failure ([`DbError::Retryable`], a `40001` that surfaces at
/// COMMIT). An overlap with a live occurrence aborts the series and maps to
/// [`DbError::Conflict`].
///
/// This function enforces only *structural* validity of the supplied periods
/// (non-empty expansion, every range `start < end`). Resource-rule validation
/// (capacity, ownership, blackout windows, etc. via
/// [`domain::validation::validate_booking`]) is the **service layer's**
/// responsibility (P14) and is intentionally not performed here.
///
/// # Errors
///
/// - [`DbError::EmptyExpansion`] — `expanded` is empty (would orphan a header).
/// - [`DbError::InvalidPeriod`] — a range is empty or inverted (`start >= end`).
/// - [`DbError::Conflict`] — an occurrence overlaps a live one (`23P01`).
/// - [`DbError::Retryable`] — exhausted retries on a serialization failure.
/// - [`DbError::Sqlx`] — any other database error.
pub async fn create_booking(
    pool: &Db,
    new: &NewBooking,
    expanded: &[TimeRange],
) -> Result<CreatedBooking, DbError> {
    // Validate structurally BEFORE BEGIN so no header / ical_uid is consumed for
    // an unpersistable request.
    if expanded.is_empty() {
        return Err(DbError::EmptyExpansion);
    }
    if expanded.iter().any(|r| !r.is_valid()) {
        return Err(DbError::InvalidPeriod);
    }

    let single = expanded.len() == 1;
    let mut attempt = 0;
    loop {
        attempt += 1;
        match try_create_booking(pool, new, expanded, single).await {
            Err(DbError::Retryable) if attempt < MAX_TX_ATTEMPTS => {}
            other => return other,
        }
    }
}

/// One transactional attempt of [`create_booking`].
async fn try_create_booking(
    pool: &Db,
    new: &NewBooking,
    expanded: &[TimeRange],
    single: bool,
) -> Result<CreatedBooking, DbError> {
    let visibility = BookingVisibilityRow::from(new.visibility);
    let source = BookingSourceRow::from(new.source);

    let mut tx = pool.begin().await.map_err(classify)?;

    let header = sqlx::query_as!(
        BookingHeaderRow,
        r#"
        INSERT INTO bookings (
            resource_id, booked_for_user_id, booked_by_user_id,
            title, description, visibility, source, ical_uid,
            recurrence_rule, recurrence_until, series_timezone
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id,
            resource_id,
            booked_for_user_id,
            booked_by_user_id,
            visibility AS "visibility: BookingVisibilityRow",
            source     AS "source: BookingSourceRow",
            ical_uid,
            status     AS "status: BookingStatusRow"
        "#,
        new.resource_id,
        new.booked_for_user_id,
        new.booked_by_user_id,
        new.title,
        new.description,
        visibility as _,
        source as _,
        new.ical_uid,
        new.recurrence_rule,
        new.recurrence_until,
        new.series_timezone,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(classify)?;

    let mut occurrence_ids = Vec::with_capacity(expanded.len());
    for range in expanded {
        let period = PgRange::from(range.start..range.end);
        // A single booking stores recurrence_id = NULL (single-occurrence
        // unique index); a series stores the UTC start instant per instance.
        let recurrence_id = if single { None } else { Some(range.start) };
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO booking_occurrences (
                booking_id, occurrence_kind, resource_id, period,
                recurrence_id, status
            )
            VALUES ($1, 'booking', $2, $3, $4, 'booked')
            RETURNING id
            "#,
            header.id,
            header.resource_id,
            period,
            recurrence_id,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(classify)?;
        occurrence_ids.push(OccurrenceId::new(id));
    }

    tx.commit().await.map_err(classify)?;

    Ok(CreatedBooking {
        booking: header.into(),
        occurrence_ids,
    })
}

/// `query_as!` target for the `bookings` RETURNING projection.
struct BookingHeaderRow {
    id: Uuid,
    resource_id: Uuid,
    booked_for_user_id: Uuid,
    booked_by_user_id: Uuid,
    visibility: BookingVisibilityRow,
    source: BookingSourceRow,
    ical_uid: String,
    status: BookingStatusRow,
}

impl From<BookingHeaderRow> for Booking {
    fn from(row: BookingHeaderRow) -> Self {
        Self {
            id: row.id,
            resource_id: row.resource_id,
            booked_for_user_id: row.booked_for_user_id,
            booked_by_user_id: row.booked_by_user_id,
            visibility: row.visibility.into(),
            source: row.source.into(),
            ical_uid: row.ical_uid,
            status: row.status.into(),
        }
    }
}

// --- transition persistence ---------------------------------------------

/// Applies a [`domain::state_machine::Transition`] to a single occurrence:
/// writes `status`, the timestamp the schema's status↔timestamp `CHECK`
/// requires, and (on checkout only) shrinks `period` to free the tail.
///
/// The new period is set with `tstzrange(lower(period), $now, '[)')` so the
/// stored lower bound is preserved exactly and the upper bound is the checkout
/// instant carried by the transition.
///
/// `expected_from` is the status the caller computed the transition against; the
/// UPDATE only matches a row still in that status (optimistic concurrency). A
/// row that has moved on (a concurrent transition, or a stale read) matches zero
/// rows and yields [`DbError::StaleState`] rather than silently clobbering it.
///
/// Note: unlike [`create_booking`], this path does not retry `40001`
/// serialization failures internally — a [`DbError::Retryable`] is the caller's
/// concern to re-read and retry.
///
/// # Errors
///
/// - [`DbError::StaleState`] — the occurrence is no longer in `expected_from`.
/// - [`DbError::Conflict`] — should not occur for a status-shrinking update,
///   but a `23P01` is still mapped defensively.
/// - [`DbError::Retryable`] / [`DbError::Sqlx`] — serialization or other DB
///   error.
pub async fn apply_transition(
    pool: &Db,
    occurrence_id: OccurrenceId,
    expected_from: BookingStatus,
    transition: &Transition,
) -> Result<(), DbError> {
    let id = occurrence_id.as_uuid();
    let status = BookingStatusRow::from(transition.new_status);
    let expected = BookingStatusRow::from(expected_from);

    // The shrunk upper bound only matters for checkout; for every other stamp
    // the period is left untouched.
    let new_end: Option<DateTime<Utc>> = match (transition.new_status, transition.new_period) {
        (BookingStatus::CheckedOut, Some(period)) => Some(period.end),
        _ => None,
    };

    let cols = StampColumns::from(transition.stamp);

    let result = sqlx::query!(
        r#"
        UPDATE booking_occurrences
        SET status           = $2,
            check_in_at      = COALESCE($3, check_in_at),
            checked_out_at   = COALESCE($4, checked_out_at),
            cancelled_at     = COALESCE($5, cancelled_at),
            auto_released_at = COALESCE($6, auto_released_at),
            period = CASE
                WHEN $7::timestamptz IS NULL THEN period
                ELSE tstzrange(lower(period), $7::timestamptz, '[)')
            END
        WHERE id = $1 AND status = $8
        "#,
        id,
        status as _,
        cols.check_in_at,
        cols.checked_out_at,
        cols.cancelled_at,
        cols.auto_released_at,
        new_end,
        expected as _,
    )
    .execute(pool)
    .await
    .map_err(classify)?;

    if result.rows_affected() == 0 {
        return Err(DbError::StaleState);
    }

    Ok(())
}

/// The four nullable timestamp columns the transition UPDATE sets; exactly one
/// is `Some` (or none, for the no-show stamp). Mirrors the schema's
/// status↔timestamp `CHECK`s.
#[derive(Clone, Copy, Default)]
#[expect(
    clippy::struct_field_names,
    reason = "fields mirror the booking_occurrences timestamp column names verbatim"
)]
struct StampColumns {
    check_in_at: Option<DateTime<Utc>>,
    checked_out_at: Option<DateTime<Utc>>,
    cancelled_at: Option<DateTime<Utc>>,
    auto_released_at: Option<DateTime<Utc>>,
}

impl From<Stamp> for StampColumns {
    fn from(stamp: Stamp) -> Self {
        match stamp {
            Stamp::CheckedInAt(at) => Self {
                check_in_at: Some(at),
                ..Self::default()
            },
            Stamp::CheckedOutAt(at) => Self {
                checked_out_at: Some(at),
                ..Self::default()
            },
            Stamp::CancelledAt(at) => Self {
                cancelled_at: Some(at),
                ..Self::default()
            },
            Stamp::AutoReleasedAt(at) => Self {
                auto_released_at: Some(at),
                ..Self::default()
            },
            Stamp::None => Self::default(),
        }
    }
}

/// Persists a check-in transition for `occurrence_id`. Thin wrapper over
/// [`apply_transition`] — the caller computes the [`Transition`] via
/// [`domain::state_machine::transition`] with the injected clock.
///
/// # Errors
///
/// See [`apply_transition`].
pub async fn check_in(
    pool: &Db,
    occurrence_id: OccurrenceId,
    transition: &Transition,
) -> Result<(), DbError> {
    apply_transition(pool, occurrence_id, BookingStatus::Booked, transition).await
}

/// Persists a check-out transition (shrinks `period` to free the tail).
///
/// # Errors
///
/// See [`apply_transition`].
pub async fn check_out(
    pool: &Db,
    occurrence_id: OccurrenceId,
    transition: &Transition,
) -> Result<(), DbError> {
    apply_transition(pool, occurrence_id, BookingStatus::CheckedIn, transition).await
}

/// Persists a cancellation transition.
///
/// # Errors
///
/// See [`apply_transition`].
pub async fn cancel(
    pool: &Db,
    occurrence_id: OccurrenceId,
    transition: &Transition,
) -> Result<(), DbError> {
    apply_transition(pool, occurrence_id, BookingStatus::Booked, transition).await
}

/// Persists an auto-release transition.
///
/// # Errors
///
/// See [`apply_transition`].
pub async fn auto_release(
    pool: &Db,
    occurrence_id: OccurrenceId,
    transition: &Transition,
) -> Result<(), DbError> {
    apply_transition(pool, occurrence_id, BookingStatus::Booked, transition).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone as _};
    use domain::state_machine::{
        BookingEvent, OccurrenceContext, TransitionPolicy, transition as compute_transition,
    };

    /// Seed the minimal FK chain (org → floor → desk → two users) and return
    /// `(resource_id, booked_for, booked_by)`.
    async fn seed(pool: &Db) -> (Uuid, Uuid, Uuid) {
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
        let resource: Uuid = sqlx::query_scalar(
            "INSERT INTO resources (location_id, kind, name) VALUES ($1, 'desk', 'Desk 1') RETURNING id",
        )
        .bind(location)
        .fetch_one(pool)
        .await
        .unwrap();
        let user: Uuid = sqlx::query_scalar(
            "INSERT INTO users (email, display_name, webauthn_user_handle) \
             VALUES ($1, 'Booker', $2) RETURNING id",
        )
        .bind(format!("{tag}@example.test"))
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .fetch_one(pool)
        .await
        .unwrap();
        (resource, user, user)
    }

    fn at(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap()
    }

    fn new_booking(resource: Uuid, for_user: Uuid, by_user: Uuid) -> NewBooking {
        NewBooking {
            resource_id: resource,
            booked_for_user_id: for_user,
            booked_by_user_id: by_user,
            title: Some("Test".to_owned()),
            description: None,
            visibility: BookingVisibility::Public,
            source: BookingSource::Web,
            ical_uid: Uuid::new_v4().to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            series_timezone: "Europe/Amsterdam".to_owned(),
        }
    }

    fn range(start: DateTime<Utc>, end: DateTime<Utc>) -> TimeRange {
        TimeRange::new(start, end)
    }

    #[sqlx::test]
    async fn overlapping_live_occurrence_is_conflict(pool: Db) {
        let (resource, for_user, by_user) = seed(&pool).await;
        create_booking(
            &pool,
            &new_booking(resource, for_user, by_user),
            &[range(at(2026, 7, 1, 9), at(2026, 7, 1, 11))],
        )
        .await
        .unwrap();

        let err = create_booking(
            &pool,
            &new_booking(resource, for_user, by_user),
            &[range(at(2026, 7, 1, 10), at(2026, 7, 1, 12))],
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::Conflict), "got {err:?}");
    }

    #[sqlx::test]
    async fn adjacent_half_open_occurrences_allowed(pool: Db) {
        let (resource, for_user, by_user) = seed(&pool).await;
        create_booking(
            &pool,
            &new_booking(resource, for_user, by_user),
            &[range(at(2026, 7, 1, 9), at(2026, 7, 1, 10))],
        )
        .await
        .unwrap();
        // [9,10) and [10,11) touch but do not overlap.
        create_booking(
            &pool,
            &new_booking(resource, for_user, by_user),
            &[range(at(2026, 7, 1, 10), at(2026, 7, 1, 11))],
        )
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn recurring_create_materialises_all_occurrences(pool: Db) {
        let (resource, for_user, by_user) = seed(&pool).await;
        let mut spec = new_booking(resource, for_user, by_user);
        spec.recurrence_rule = Some("FREQ=DAILY;COUNT=3".to_owned());
        // Three non-overlapping daily instances.
        let expanded = vec![
            range(at(2026, 7, 1, 9), at(2026, 7, 1, 10)),
            range(at(2026, 7, 2, 9), at(2026, 7, 2, 10)),
            range(at(2026, 7, 3, 9), at(2026, 7, 3, 10)),
        ];
        let created = create_booking(&pool, &spec, &expanded).await.unwrap();
        assert_eq!(created.occurrence_ids.len(), 3);

        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM booking_occurrences WHERE booking_id = $1")
                .bind(created.booking.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 3);
    }

    #[sqlx::test]
    async fn early_checkout_frees_the_tail(pool: Db) {
        let (resource, for_user, by_user) = seed(&pool).await;
        let period = range(at(2026, 7, 1, 9), at(2026, 7, 1, 17));
        let created = create_booking(&pool, &new_booking(resource, for_user, by_user), &[period])
            .await
            .unwrap();
        let occ = created.occurrence_ids[0];

        let policy = TransitionPolicy {
            check_in_window: Duration::hours(1),
            grace: Duration::minutes(15),
            cancellation_deadline: None,
        };
        let ctx = OccurrenceContext { period };

        // Check in at 09:00, then check out at 12:00 — the tail [12:00,17:00) frees.
        let ci = compute_transition(
            BookingStatus::Booked,
            BookingEvent::CheckIn,
            ctx,
            policy,
            at(2026, 7, 1, 9),
        )
        .unwrap();
        check_in(&pool, occ, &ci).await.unwrap();

        let co = compute_transition(
            BookingStatus::CheckedIn,
            BookingEvent::CheckOut,
            ctx,
            policy,
            at(2026, 7, 1, 12),
        )
        .unwrap();
        check_out(&pool, occ, &co).await.unwrap();

        // A fresh booking in the freed window [13:00,16:00) now succeeds.
        create_booking(
            &pool,
            &new_booking(resource, for_user, by_user),
            &[range(at(2026, 7, 1, 13), at(2026, 7, 1, 16))],
        )
        .await
        .unwrap();
    }

    fn at_min(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .unwrap()
    }

    #[sqlx::test]
    async fn checkout_before_start_preserves_full_period(pool: Db) {
        // End-to-end M1 guard: check in BEFORE start (within the window) and
        // check out BEFORE start. The clamp must keep the full period
        // [09:00,17:00) so the tstzrange stays non-empty and the UPDATE
        // succeeds against the period CHECK.
        let (resource, for_user, by_user) = seed(&pool).await;
        let period = range(at(2026, 7, 1, 9), at(2026, 7, 1, 17));
        let created = create_booking(&pool, &new_booking(resource, for_user, by_user), &[period])
            .await
            .unwrap();
        let occ = created.occurrence_ids[0];

        let policy = TransitionPolicy {
            check_in_window: Duration::hours(1), // check-in opens at 08:00
            grace: Duration::minutes(15),
            cancellation_deadline: None,
        };
        let ctx = OccurrenceContext { period };

        // Check in at 08:30 (before the 09:00 start, inside the window).
        let ci = compute_transition(
            BookingStatus::Booked,
            BookingEvent::CheckIn,
            ctx,
            policy,
            at_min(2026, 7, 1, 8, 30),
        )
        .unwrap();
        check_in(&pool, occ, &ci).await.unwrap();

        // Check out at 08:45, still before start. Must succeed (clamp ⇒ full
        // period preserved).
        let co = compute_transition(
            BookingStatus::CheckedIn,
            BookingEvent::CheckOut,
            ctx,
            policy,
            at_min(2026, 7, 1, 8, 45),
        )
        .unwrap();
        check_out(&pool, occ, &co).await.unwrap();

        // The stored period is unchanged: full [09:00,17:00).
        let stored: PgRange<DateTime<Utc>> =
            sqlx::query_scalar("SELECT period FROM booking_occurrences WHERE id = $1")
                .bind(occ.as_uuid())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored, PgRange::from(period.start..period.end));
    }

    #[sqlx::test]
    async fn stale_status_yields_stale_state(pool: Db) {
        // Minor-c guard: applying a transition whose expected from-status no
        // longer matches the row yields StaleState, not a silent clobber.
        let (resource, for_user, by_user) = seed(&pool).await;
        let period = range(at(2026, 7, 1, 9), at(2026, 7, 1, 11));
        let created = create_booking(&pool, &new_booking(resource, for_user, by_user), &[period])
            .await
            .unwrap();
        let occ = created.occurrence_ids[0];

        let policy = TransitionPolicy {
            check_in_window: Duration::hours(1),
            grace: Duration::minutes(15),
            cancellation_deadline: None,
        };
        let ctx = OccurrenceContext { period };

        // Cancel the booking (Booked → Cancelled).
        let cx = compute_transition(
            BookingStatus::Booked,
            BookingEvent::Cancel,
            ctx,
            policy,
            at(2026, 6, 30, 9),
        )
        .unwrap();
        cancel(&pool, occ, &cx).await.unwrap();

        // A check-in computed against the now-stale Booked status must not
        // overwrite the cancelled row.
        let ci = compute_transition(
            BookingStatus::Booked,
            BookingEvent::CheckIn,
            ctx,
            policy,
            at(2026, 7, 1, 9),
        )
        .unwrap();
        let err = check_in(&pool, occ, &ci).await.unwrap_err();
        assert!(matches!(err, DbError::StaleState), "got {err:?}");
    }

    #[sqlx::test]
    async fn empty_expansion_is_rejected(pool: Db) {
        // M4 guard: zero occurrences must be rejected before any header write.
        let (resource, for_user, by_user) = seed(&pool).await;
        let err = create_booking(&pool, &new_booking(resource, for_user, by_user), &[])
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::EmptyExpansion), "got {err:?}");
        // No header was written.
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM bookings")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[sqlx::test]
    async fn inverted_period_is_rejected(pool: Db) {
        // M5 guard: an inverted (start > end) range is rejected structurally
        // before BEGIN, with no orphan header.
        let (resource, for_user, by_user) = seed(&pool).await;
        let inverted = range(at(2026, 7, 1, 11), at(2026, 7, 1, 9));
        let err = create_booking(
            &pool,
            &new_booking(resource, for_user, by_user),
            &[inverted],
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::InvalidPeriod), "got {err:?}");
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM bookings")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[sqlx::test]
    async fn cancelled_occurrence_no_longer_blocks(pool: Db) {
        let (resource, for_user, by_user) = seed(&pool).await;
        let period = range(at(2026, 7, 1, 9), at(2026, 7, 1, 11));
        let created = create_booking(&pool, &new_booking(resource, for_user, by_user), &[period])
            .await
            .unwrap();
        let occ = created.occurrence_ids[0];

        let policy = TransitionPolicy {
            check_in_window: Duration::hours(1),
            grace: Duration::minutes(15),
            cancellation_deadline: None,
        };
        let ctx = OccurrenceContext { period };
        let cx = compute_transition(
            BookingStatus::Booked,
            BookingEvent::Cancel,
            ctx,
            policy,
            at(2026, 6, 30, 9),
        )
        .unwrap();
        cancel(&pool, occ, &cx).await.unwrap();

        // The same window is now bookable again.
        create_booking(
            &pool,
            &new_booking(resource, for_user, by_user),
            &[range(at(2026, 7, 1, 9), at(2026, 7, 1, 11))],
        )
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn released_occurrence_no_longer_blocks(pool: Db) {
        let (resource, for_user, by_user) = seed(&pool).await;
        let period = range(at(2026, 7, 1, 9), at(2026, 7, 1, 11));
        let created = create_booking(&pool, &new_booking(resource, for_user, by_user), &[period])
            .await
            .unwrap();
        let occ = created.occurrence_ids[0];

        let policy = TransitionPolicy {
            check_in_window: Duration::hours(1),
            grace: Duration::minutes(15),
            cancellation_deadline: None,
        };
        let ctx = OccurrenceContext { period };
        // Auto-release after grace (start 09:00 + 15m → 09:15).
        let rx = compute_transition(
            BookingStatus::Booked,
            BookingEvent::AutoRelease,
            ctx,
            policy,
            at(2026, 7, 1, 10),
        )
        .unwrap();
        auto_release(&pool, occ, &rx).await.unwrap();

        create_booking(
            &pool,
            &new_booking(resource, for_user, by_user),
            &[range(at(2026, 7, 1, 9), at(2026, 7, 1, 11))],
        )
        .await
        .unwrap();
    }
}
