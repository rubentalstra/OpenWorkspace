//! Pure validation of a booking request against its resource's rules.
//!
//! Mirrors the `resource_rules` table. Every rule field is nullable; a `None`
//! field means "no constraint" and is skipped. All violations are collected so
//! the caller can report them together.

use chrono::{DateTime, Duration, Utc};

use crate::time_range::TimeRange;

/// The booking the caller wants to create.
#[derive(Clone, Debug)]
pub struct BookingRequest {
    /// The half-open period of the (first) occurrence in UTC.
    pub period: TimeRange,
    /// `true` if the request carries an `RRULE` (recurring series).
    pub is_recurring: bool,
    /// Number of materialised occurrences the series would produce.
    pub occurrence_count: u32,
    /// Latest occurrence end, for horizon checking. Equals `period.end` for a
    /// single booking.
    pub series_end: DateTime<Utc>,
}

/// The subset of `resource_rules` the engine enforces. Each `None` field
/// disables that rule.
#[derive(Clone, Copy, Debug, Default)]
pub struct ResourceRules {
    /// Maximum days in advance a booking may start.
    pub max_advance_days: Option<i64>,
    /// Minimum minutes in advance a booking must start (lead time).
    pub min_advance_minutes: Option<i64>,
    /// Minimum occurrence duration in minutes.
    pub min_duration_minutes: Option<i64>,
    /// Maximum occurrence duration in minutes.
    pub max_duration_minutes: Option<i64>,
    /// Slot granularity in minutes; start offset and duration must be multiples.
    pub slot_granularity_minutes: Option<i64>,
    /// Maximum bookings per user per day.
    pub max_per_user_per_day: Option<u32>,
    /// Maximum active (non-terminal) bookings per user.
    pub max_active_per_user: Option<u32>,
    /// Whether recurring bookings are allowed at all.
    pub allow_recurrence: bool,
    /// Maximum number of occurrences a recurring series may materialise.
    pub max_recurrence_count: Option<u32>,
    /// Maximum days into the future a recurring series may extend.
    pub max_recurrence_horizon_days: Option<i64>,
}

/// A single rule that the request violates.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleViolation {
    /// The period is empty or inverted (`start >= end`).
    InvalidPeriod,
    /// Start is further in the future than `max_advance_days`.
    TooFarInAdvance,
    /// Start is sooner than `min_advance_minutes` from `now`.
    InsufficientLeadTime,
    /// Duration is below `min_duration_minutes`.
    DurationTooShort,
    /// Duration exceeds `max_duration_minutes`.
    DurationTooLong,
    /// Start offset or duration is not a multiple of `slot_granularity_minutes`.
    NotOnSlotBoundary,
    /// Recurrence requested but `allow_recurrence` is false.
    RecurrenceNotAllowed,
    /// Materialised occurrence count exceeds `max_recurrence_count`.
    TooManyOccurrences,
    /// Series end is beyond `max_recurrence_horizon_days` from `now`.
    BeyondRecurrenceHorizon,
    /// The user already has `max_per_user_per_day` bookings for the day.
    PerDayLimitExceeded,
    /// The user already has `max_active_per_user` active bookings.
    ActiveLimitExceeded,
}

/// Validates `req` against `rules`, given the user's current `active_count` and
/// `per_day_count`, at clock `now`.
///
/// # Errors
///
/// Returns `Err(Vec<RuleViolation>)` listing every rule the request breaks; the
/// vector is non-empty on failure.
pub fn validate_booking(
    req: &BookingRequest,
    rules: &ResourceRules,
    active_count: u32,
    per_day_count: u32,
    now: DateTime<Utc>,
) -> Result<(), Vec<RuleViolation>> {
    let mut violations = Vec::new();

    if !req.period.is_valid() {
        // A degenerate period makes the remaining duration/slot checks
        // meaningless, so report it alone.
        return Err(vec![RuleViolation::InvalidPeriod]);
    }

    let start = req.period.start;
    let duration = req.period.end - req.period.start;

    if let Some(days) = rules.max_advance_days
        && start > now + Duration::days(days)
    {
        violations.push(RuleViolation::TooFarInAdvance);
    }

    if let Some(mins) = rules.min_advance_minutes
        && start < now + Duration::minutes(mins)
    {
        violations.push(RuleViolation::InsufficientLeadTime);
    }

    if let Some(mins) = rules.min_duration_minutes
        && duration < Duration::minutes(mins)
    {
        violations.push(RuleViolation::DurationTooShort);
    }

    if let Some(mins) = rules.max_duration_minutes
        && duration > Duration::minutes(mins)
    {
        violations.push(RuleViolation::DurationTooLong);
    }

    // `gran > 0` is a DB CHECK invariant, but guarding it avoids a
    // divide-by-zero on malformed input.
    if let Some(gran) = rules.slot_granularity_minutes
        && gran > 0
    {
        let gran_secs = gran * 60;
        let start_off = start.timestamp() % gran_secs;
        let dur_secs = duration.num_seconds();
        if start_off != 0 || dur_secs % gran_secs != 0 {
            violations.push(RuleViolation::NotOnSlotBoundary);
        }
    }

    if req.is_recurring {
        if !rules.allow_recurrence {
            violations.push(RuleViolation::RecurrenceNotAllowed);
        }
        if let Some(cap) = rules.max_recurrence_count
            && req.occurrence_count > cap
        {
            violations.push(RuleViolation::TooManyOccurrences);
        }
        if let Some(days) = rules.max_recurrence_horizon_days
            && req.series_end > now + Duration::days(days)
        {
            violations.push(RuleViolation::BeyondRecurrenceHorizon);
        }
    }

    if let Some(cap) = rules.max_per_user_per_day
        && per_day_count >= cap
    {
        violations.push(RuleViolation::PerDayLimitExceeded);
    }

    if let Some(cap) = rules.max_active_per_user
        && active_count >= cap
    {
        violations.push(RuleViolation::ActiveLimitExceeded);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(secs, 0).single().expect("valid instant")
    }

    const NOW: i64 = 1_000_000;

    fn req(start: i64, end: i64) -> BookingRequest {
        BookingRequest {
            period: TimeRange::new(at(start), at(end)),
            is_recurring: false,
            occurrence_count: 1,
            series_end: at(end),
        }
    }

    fn ok_check(rules: &ResourceRules, r: &BookingRequest) -> Result<(), Vec<RuleViolation>> {
        validate_booking(r, rules, 0, 0, at(NOW))
    }

    #[test]
    fn no_rules_passes() {
        let rules = ResourceRules {
            allow_recurrence: true,
            ..ResourceRules::default()
        };
        let r = req(NOW + 3600, NOW + 7200);
        assert!(ok_check(&rules, &r).is_ok());
    }

    #[test]
    fn invalid_period_reported_alone() {
        let rules = ResourceRules::default();
        let r = req(NOW + 7200, NOW + 3600); // inverted
        assert_eq!(
            ok_check(&rules, &r),
            Err(vec![RuleViolation::InvalidPeriod])
        );
    }

    #[test]
    fn duration_bounds() {
        let rules = ResourceRules {
            min_duration_minutes: Some(30),
            max_duration_minutes: Some(120),
            ..ResourceRules::default()
        };
        let too_short = req(NOW + 3600, NOW + 3600 + 60 * 10); // 10 min
        assert_eq!(
            ok_check(&rules, &too_short),
            Err(vec![RuleViolation::DurationTooShort])
        );
        let too_long = req(NOW + 3600, NOW + 3600 + 60 * 200); // 200 min
        assert_eq!(
            ok_check(&rules, &too_long),
            Err(vec![RuleViolation::DurationTooLong])
        );
    }

    #[test]
    fn lead_time_and_advance() {
        let rules = ResourceRules {
            min_advance_minutes: Some(60),
            max_advance_days: Some(7),
            ..ResourceRules::default()
        };
        let too_soon = req(NOW + 60, NOW + 3600);
        assert_eq!(
            ok_check(&rules, &too_soon),
            Err(vec![RuleViolation::InsufficientLeadTime])
        );
        let too_far = req(NOW + 60 * 60 * 24 * 30, NOW + 60 * 60 * 24 * 30 + 3600);
        assert_eq!(
            ok_check(&rules, &too_far),
            Err(vec![RuleViolation::TooFarInAdvance])
        );
    }

    #[test]
    fn slot_granularity() {
        let rules = ResourceRules {
            slot_granularity_minutes: Some(30), // 1800s
            ..ResourceRules::default()
        };
        // Start aligned to 1800s, duration 1800s ⇒ ok.
        let aligned = req(1_800_000, 1_800_000 + 1800);
        assert!(validate_booking(&aligned, &rules, 0, 0, at(0)).is_ok());
        // Start off the boundary.
        let misaligned = req(1_800_001, 1_800_001 + 1800);
        assert_eq!(
            validate_booking(&misaligned, &rules, 0, 0, at(0)),
            Err(vec![RuleViolation::NotOnSlotBoundary])
        );
    }

    #[test]
    fn recurrence_gates() {
        let rules = ResourceRules {
            allow_recurrence: false,
            max_recurrence_count: Some(5),
            max_recurrence_horizon_days: Some(30),
            ..ResourceRules::default()
        };
        let mut r = req(NOW + 3600, NOW + 7200);
        r.is_recurring = true;
        r.occurrence_count = 10;
        r.series_end = at(NOW + 60 * 60 * 24 * 90);
        let v = ok_check(&rules, &r).unwrap_err();
        assert!(v.contains(&RuleViolation::RecurrenceNotAllowed));
        assert!(v.contains(&RuleViolation::TooManyOccurrences));
        assert!(v.contains(&RuleViolation::BeyondRecurrenceHorizon));
    }

    #[test]
    fn recurrence_gates_skipped_for_single() {
        let rules = ResourceRules {
            allow_recurrence: false,
            ..ResourceRules::default()
        };
        let r = req(NOW + 3600, NOW + 7200); // not recurring
        assert!(ok_check(&rules, &r).is_ok());
    }

    #[test]
    fn per_user_caps() {
        let rules = ResourceRules {
            max_per_user_per_day: Some(2),
            max_active_per_user: Some(3),
            ..ResourceRules::default()
        };
        let r = req(NOW + 3600, NOW + 7200);
        let v = validate_booking(&r, &rules, 3, 2, at(NOW)).unwrap_err();
        assert!(v.contains(&RuleViolation::PerDayLimitExceeded));
        assert!(v.contains(&RuleViolation::ActiveLimitExceeded));
        // Under the caps ⇒ ok.
        assert!(validate_booking(&r, &rules, 2, 1, at(NOW)).is_ok());
    }
}
