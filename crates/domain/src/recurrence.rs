//! DST-correct recurrence expansion (`rrule` + `chrono-tz`).
//!
//! # Strategy
//!
//! `rrule` resolves wall-clock times to instants internally with policies we do
//! not control (it drops both DST gaps *and* ambiguous times via
//! `LocalResult::single`, then a fallback shifts gaps forward). To apply the
//! project's explicit policy — **gap → skip, ambiguous → earliest** — we run
//! the rule in a *naive* frame: DTSTART, EXDATEs, RDATEs and the `until` bound
//! are all anchored as if their local wall-clock components were UTC. Every
//! instant `rrule` then emits carries, in its `naive_utc()`, the *intended
//! local wall-clock* time. We resolve each of those through `chrono-tz`
//! ourselves with the documented policy.
//!
//! Occurrence end is always **UTC start + duration**; a local end is never
//! re-resolved (which would silently lengthen/shorten across a DST boundary).

use chrono::{DateTime, Duration, MappedLocalTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz as ChronoTz;
use rrule::{RRuleSet, Tz as RruleTz};

use crate::time_range::TimeRange;

/// Input describing a (possibly recurring) booking series to expand.
#[derive(Clone, Debug)]
pub struct SeriesSpec {
    /// First instance start, expressed in `series_timezone` local wall-clock
    /// time (its UTC offset is ignored; only the naive components matter).
    pub dtstart_local: NaiveDateTime,
    /// IANA TZID (e.g. `"Europe/Amsterdam"`). Validated here.
    pub series_timezone: String,
    /// The `RRULE` line (without `DTSTART`), e.g. `"RRULE:FREQ=WEEKLY;COUNT=10"`
    /// or `"FREQ=WEEKLY;COUNT=10"`. `None`/empty ⇒ a single occurrence.
    pub recurrence_rule: Option<String>,
    /// Duration of each occurrence. `end = start_utc + duration`.
    pub duration: Duration,
    /// Optional materialisation cap (inclusive), a real UTC instant.
    pub recurrence_until: Option<DateTime<Utc>>,
    /// Exception dates to remove, expressed in `series_timezone` local
    /// wall-clock time. Matched against generated instants by exact instant.
    pub exdates_local: Vec<NaiveDateTime>,
    /// Extra dates to add, expressed in `series_timezone` local wall-clock time.
    pub rdates_local: Vec<NaiveDateTime>,
    /// Hard upper bound on instances. A series that genuinely produces *more*
    /// than `max_count` instances is reported as [`ExpandError::Truncated`]; a
    /// series whose natural length is `<= max_count` (including `== max_count`)
    /// returns `Ok`.
    pub max_count: u16,
}

/// Why a series could not be expanded.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExpandError {
    /// `series_timezone` is not a recognised IANA TZID.
    UnknownTimezone(String),
    /// `DTSTART`'s local wall-clock time falls in a DST gap (does not exist).
    GapTime(NaiveDateTime),
    /// The `RRULE` string failed to parse or validate.
    InvalidRule(String),
    /// `max_count` is zero, so nothing could be produced.
    EmptyLimit,
    /// Expansion hit `max_count` before the rule was exhausted. Carries the
    /// instances produced up to the cap (a materialisation limit, never the
    /// natural end of the series).
    Truncated(Vec<TimeRange>),
}

impl core::fmt::Display for ExpandError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownTimezone(tz) => write!(f, "unknown series timezone: {tz}"),
            Self::GapTime(dt) => write!(f, "DTSTART {dt} falls in a DST gap"),
            Self::InvalidRule(msg) => write!(f, "invalid recurrence rule: {msg}"),
            Self::EmptyLimit => write!(f, "max_count is zero"),
            Self::Truncated(items) => {
                write!(f, "series truncated at {} instances", items.len())
            }
        }
    }
}

impl core::error::Error for ExpandError {}

/// Anchors a naive local wall-clock time into the *naive frame* used for
/// `rrule`: the components are reinterpreted as UTC. The returned value's
/// `naive_utc()` therefore equals the intended local wall-clock.
fn naive_as_utc(dt: NaiveDateTime) -> DateTime<RruleTz> {
    Utc.from_utc_datetime(&dt).with_timezone(&RruleTz::UTC)
}

/// Resolves an intended local wall-clock time (carried in a generated instant's
/// `naive_utc()`) into a real UTC instant under the project policy.
///
/// `Single` ⇒ that instant; `Ambiguous` ⇒ earliest; `None` (gap) ⇒ skipped.
fn resolve_local(tz: ChronoTz, local: NaiveDateTime) -> Option<DateTime<Utc>> {
    match tz.from_local_datetime(&local) {
        MappedLocalTime::Single(dt) => Some(dt.with_timezone(&Utc)),
        MappedLocalTime::Ambiguous(earliest, _latest) => Some(earliest.with_timezone(&Utc)),
        MappedLocalTime::None => None,
    }
}

/// Expands a [`SeriesSpec`] into its concrete UTC occurrence ranges.
///
/// The returned `Vec`'s length is the authoritative occurrence count: it may be
/// **less** than the rule's nominal `COUNT` because DST-gap instances are
/// skipped (RFC 5545 §3.3.10) and duplicate UTC start instants are deduped.
/// Callers must derive `occurrence_count` from the returned `Vec`, never from
/// the `RRULE` COUNT.
///
/// # Errors
///
/// Returns [`ExpandError`] when the timezone or rule is invalid, when DTSTART
/// lands in a DST gap, when `max_count == 0`, or — as
/// [`ExpandError::Truncated`] — when the rule genuinely produces more than
/// `max_count` instances (a materialisation limit, never the natural end of the
/// series; a series whose natural length equals `max_count` returns `Ok`).
pub fn expand_series(spec: &SeriesSpec) -> Result<Vec<TimeRange>, ExpandError> {
    if spec.max_count == 0 {
        return Err(ExpandError::EmptyLimit);
    }

    let tz: ChronoTz = spec
        .series_timezone
        .parse()
        .map_err(|_| ExpandError::UnknownTimezone(spec.series_timezone.clone()))?;

    // DTSTART must exist in the timezone (gap ⇒ hard error, per policy).
    if matches!(
        tz.from_local_datetime(&spec.dtstart_local),
        MappedLocalTime::None
    ) {
        return Err(ExpandError::GapTime(spec.dtstart_local));
    }

    let dtstart = naive_as_utc(spec.dtstart_local);
    let mut set = RRuleSet::new(dtstart);

    let mut rdates: Vec<DateTime<RruleTz>> = spec
        .rdates_local
        .iter()
        .copied()
        .map(naive_as_utc)
        .collect();

    match spec.recurrence_rule.as_deref().map(str::trim) {
        Some(rule) if !rule.is_empty() => {
            set = set
                .set_from_string(rule)
                .map_err(|e| ExpandError::InvalidRule(e.to_string()))?;
        }
        // No rule: a single occurrence at DTSTART. An empty `RRuleSet` yields
        // nothing, so add DTSTART as an explicit RDATE.
        _ => rdates.push(dtstart),
    }

    if !spec.exdates_local.is_empty() {
        set = set.set_exdates(
            spec.exdates_local
                .iter()
                .copied()
                .map(naive_as_utc)
                .collect(),
        );
    }
    if !rdates.is_empty() {
        set = set.set_rdates(rdates);
    }

    // `recurrence_until` is a real UTC instant; the rule runs in the naive
    // frame, so bound it by the *local wall-clock* it denotes (DST-correct).
    if let Some(until) = spec.recurrence_until {
        let until_local = until.with_timezone(&tz).naive_local();
        set = set.before(naive_as_utc(until_local));
    }

    // Do NOT trust rrule's `limited` flag: `collect_with_error` sets it whenever
    // the output length *equals* the requested limit, even on natural
    // exhaustion (COUNT/UNTIL ending at exactly `max_count`). Probe one past the
    // cap and treat the series as truncated only when the rule genuinely yields
    // strictly more than `max_count` instances.
    let probe = spec.max_count.saturating_add(1);
    let result = set.all(probe);
    let truncated = result.dates.len() > usize::from(spec.max_count);

    let mut out = Vec::with_capacity(result.dates.len().min(usize::from(spec.max_count)));
    let mut seen_starts: Vec<DateTime<Utc>> = Vec::new();
    for instant in &result.dates {
        // Keep at most `max_count` instances regardless of the probe overshoot.
        if out.len() >= usize::from(spec.max_count) {
            break;
        }
        // The intended local wall-clock is this instant's naive datetime.
        let local = instant.naive_utc();
        let Some(start_utc) = resolve_local(tz, local) else {
            // DST gap ⇒ skip this instance; the series continues.
            continue;
        };
        // Dedup identical UTC start instants (e.g. an RDATE duplicating a
        // generated instant, or two wall-clock times collapsing to the same UTC
        // instant). A series stores `recurrence_id = start`, so duplicates would
        // collide on `booking_occurrences_instance_uq`.
        if seen_starts.contains(&start_utc) {
            continue;
        }
        seen_starts.push(start_utc);
        let end_utc = start_utc + spec.duration;
        out.push(TimeRange::new(start_utc, end_utc));
    }

    if truncated {
        return Err(ExpandError::Truncated(out));
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, NaiveDate, Timelike};
    use proptest::prelude::*;

    const AMS: &str = "Europe/Amsterdam";

    fn naive(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, mo, d)
            .expect("valid date")
            .and_hms_opt(h, mi, 0)
            .expect("valid time")
    }

    fn base(rule: &str, dtstart: NaiveDateTime) -> SeriesSpec {
        SeriesSpec {
            dtstart_local: dtstart,
            series_timezone: AMS.to_owned(),
            recurrence_rule: Some(rule.to_owned()),
            duration: Duration::hours(1),
            recurrence_until: None,
            exdates_local: Vec::new(),
            rdates_local: Vec::new(),
            max_count: 1000,
        }
    }

    #[test]
    fn unknown_timezone_errors() {
        let mut spec = base("RRULE:FREQ=DAILY;COUNT=2", naive(2026, 1, 1, 9, 0));
        spec.series_timezone = "Mars/Olympus".to_owned();
        assert!(matches!(
            expand_series(&spec),
            Err(ExpandError::UnknownTimezone(_))
        ));
    }

    #[test]
    fn zero_max_count_errors() {
        let mut spec = base("RRULE:FREQ=DAILY;COUNT=2", naive(2026, 1, 1, 9, 0));
        spec.max_count = 0;
        assert_eq!(expand_series(&spec), Err(ExpandError::EmptyLimit));
    }

    #[test]
    fn single_occurrence_when_no_rule() {
        let mut spec = base("", naive(2026, 1, 1, 9, 0));
        spec.recurrence_rule = None;
        let out = expand_series(&spec).expect("ok");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].end, out[0].start + Duration::hours(1));
    }

    #[test]
    fn single_occurrence_max_count_one() {
        // A non-recurring booking with max_count == 1 must return Ok(len 1),
        // not Truncated. rrule reports len==limit as `limited`, so the old code
        // wrongly rejected the single most common path.
        let mut spec = base("", naive(2026, 1, 1, 9, 0));
        spec.recurrence_rule = None;
        spec.max_count = 1;
        let out = expand_series(&spec).expect("single occurrence is Ok at cap 1");
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn count_equal_to_cap_is_ok() {
        // COUNT=10 with max_count=10: the series ends naturally at exactly the
        // cap, so it must be Ok(len 10), not Truncated.
        let mut spec = base("RRULE:FREQ=DAILY;COUNT=10", naive(2026, 1, 1, 9, 0));
        spec.max_count = 10;
        let out = expand_series(&spec).expect("count == cap is Ok");
        assert_eq!(out.len(), 10);
    }

    #[test]
    fn until_yielding_exactly_cap_is_ok() {
        let tz: ChronoTz = AMS.parse().expect("tz");
        // until = 2026-01-04 09:00 local ⇒ Jan 1,2,3,4 (before() inclusive) = 4.
        let until = resolve_local(tz, naive(2026, 1, 4, 9, 0)).expect("resolves");
        let mut spec = base("RRULE:FREQ=DAILY", naive(2026, 1, 1, 9, 0));
        spec.recurrence_until = Some(until);
        spec.max_count = 4; // the UNTIL-bounded count equals the cap exactly.
        let out = expand_series(&spec).expect("UNTIL count == cap is Ok");
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn duplicate_rdate_instant_is_deduped() {
        // An RDATE that duplicates a generated instant must not yield two ranges
        // with the same UTC start (which would collide on instance_uq when
        // persisted as a series).
        let mut spec = base("RRULE:FREQ=DAILY;COUNT=3", naive(2026, 1, 1, 9, 0));
        // Jan 2 09:00 is already generated by the rule; adding it as an RDATE
        // must not create a duplicate.
        spec.rdates_local = vec![naive(2026, 1, 2, 9, 0)];
        let out = expand_series(&spec).expect("ok");
        assert_eq!(out.len(), 3, "duplicate RDATE instant must be deduped");
        // All start instants are distinct.
        let mut starts: Vec<_> = out.iter().map(|r| r.start).collect();
        starts.sort();
        starts.dedup();
        assert_eq!(starts.len(), out.len());
    }

    #[test]
    fn ambiguous_dtstart_takes_earliest() {
        // DTSTART at 2026-10-25 02:30 Amsterdam is ambiguous (fall-back). Policy
        // resolves it to the earliest UTC offset; the first instant must equal
        // resolve_local's earliest.
        let tz: ChronoTz = AMS.parse().expect("tz");
        let spec = base("RRULE:FREQ=DAILY;COUNT=1", naive(2026, 10, 25, 2, 30));
        let out = expand_series(&spec).expect("ambiguous DTSTART expands");
        assert_eq!(out.len(), 1);
        let earliest = resolve_local(tz, naive(2026, 10, 25, 2, 30)).expect("resolves");
        assert_eq!(out[0].start, earliest);
    }

    #[test]
    fn count_is_respected() {
        let spec = base("RRULE:FREQ=DAILY;COUNT=10", naive(2026, 1, 1, 9, 0));
        let out = expand_series(&spec).expect("ok");
        assert_eq!(out.len(), 10);
    }

    #[test]
    fn exdate_removes_exact_instant() {
        let mut spec = base("RRULE:FREQ=DAILY;COUNT=5", naive(2026, 1, 1, 9, 0));
        spec.exdates_local = vec![naive(2026, 1, 3, 9, 0)];
        let out = expand_series(&spec).expect("ok");
        assert_eq!(out.len(), 4);
        // The Jan-3 09:00 local instant must be absent.
        let tz: ChronoTz = AMS.parse().expect("tz");
        let removed = resolve_local(tz, naive(2026, 1, 3, 9, 0)).expect("resolves");
        assert!(out.iter().all(|r| r.start != removed));
    }

    #[test]
    fn rdate_adds_instant() {
        let mut spec = base("RRULE:FREQ=DAILY;COUNT=3", naive(2026, 1, 1, 9, 0));
        spec.rdates_local = vec![naive(2026, 2, 1, 9, 0)];
        let out = expand_series(&spec).expect("ok");
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn recurrence_until_clamps() {
        let tz: ChronoTz = AMS.parse().expect("tz");
        // until = 2026-01-04 09:00 local, as a real UTC instant.
        let until = resolve_local(tz, naive(2026, 1, 4, 9, 0)).expect("resolves");
        let mut spec = base("RRULE:FREQ=DAILY", naive(2026, 1, 1, 9, 0));
        spec.recurrence_until = Some(until);
        let out = expand_series(&spec).expect("ok");
        // Jan 1,2,3,4 — `before` is inclusive of the bound.
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn limited_maps_to_truncated() {
        let mut spec = base("RRULE:FREQ=DAILY", naive(2026, 1, 1, 9, 0));
        spec.max_count = 3;
        let err = expand_series(&spec).expect_err("unbounded rule must truncate");
        let items = truncated_items(&err).expect("expected Truncated");
        assert_eq!(items.len(), 3);
    }

    /// Extracts the items from a [`ExpandError::Truncated`], or `None` otherwise.
    /// Lets tests assert truncation without a `panic!`/`unreachable!`.
    fn truncated_items(err: &ExpandError) -> Option<&[TimeRange]> {
        match err {
            ExpandError::Truncated(items) => Some(items),
            _ => None,
        }
    }

    #[test]
    fn dtstart_in_gap_errors() {
        // 2026-03-29 02:30 does not exist in Amsterdam (spring forward).
        let spec = base("RRULE:FREQ=DAILY;COUNT=1", naive(2026, 3, 29, 2, 30));
        assert!(matches!(expand_series(&spec), Err(ExpandError::GapTime(_))));
    }

    // --- DST round-trips ---

    #[test]
    fn dst_spring_forward_round_trip() {
        // Weekly Sunday 09:00 spanning the 2026-03-29 spring-forward.
        let spec = base("RRULE:FREQ=WEEKLY;COUNT=4", naive(2026, 3, 15, 9, 0));
        let out = expand_series(&spec).expect("ok");
        assert_eq!(out.len(), 4);
        let tz: ChronoTz = AMS.parse().expect("tz");
        for r in &out {
            let local = r.start.with_timezone(&tz);
            assert_eq!(local.hour(), 9, "each instant is 09:00 local");
            assert_eq!(local.minute(), 0);
        }
        // The pre-DST and post-DST instants differ by the offset change.
        assert!(out[1].start.with_timezone(&tz).date_naive().month() == 3);
    }

    #[test]
    fn dst_fall_back_round_trip_count_preserved() {
        // Weekly Sunday 09:00 spanning the 2026-10-25 fall-back. 09:00 is not the
        // ambiguous hour, but the offset changes; count and local time hold.
        let spec = base("RRULE:FREQ=WEEKLY;COUNT=4", naive(2026, 10, 11, 9, 0));
        let out = expand_series(&spec).expect("ok");
        assert_eq!(out.len(), 4);
        let tz: ChronoTz = AMS.parse().expect("tz");
        for r in &out {
            let local = r.start.with_timezone(&tz);
            assert_eq!(local.hour(), 9);
            assert_eq!(local.minute(), 0);
        }
    }

    #[test]
    fn dst_gap_instance_is_skipped_series_continues() {
        // Daily 02:30 across the 2026-03-29 gap. DTSTART itself is fine (Mar 27),
        // but the Mar-29 02:30 instance lands in the gap and is skipped.
        let spec = base("RRULE:FREQ=DAILY;COUNT=4", naive(2026, 3, 27, 2, 30));
        let out = expand_series(&spec).expect("ok");
        // Mar 27, 28, [29 skipped], 30 ⇒ 3 instances, series continued past gap.
        assert_eq!(out.len(), 3);
        let tz: ChronoTz = AMS.parse().expect("tz");
        let days: Vec<u32> = out
            .iter()
            .map(|r| r.start.with_timezone(&tz).day())
            .collect();
        assert_eq!(days, vec![27, 28, 30]);
    }

    #[test]
    fn ambiguous_instance_takes_earliest() {
        // Daily 02:30 across the 2026-10-25 fall-back. The Oct-25 02:30 instance
        // is ambiguous; policy takes the earliest (pre-fall-back, CEST = +02:00).
        let spec = base("RRULE:FREQ=DAILY;COUNT=4", naive(2026, 10, 24, 2, 30));
        let out = expand_series(&spec).expect("ok");
        // No instance is dropped on fall-back; all 4 present.
        assert_eq!(out.len(), 4);
        let tz: ChronoTz = AMS.parse().expect("tz");
        // The Oct-25 instant resolves to the earliest (CEST) offset.
        let oct25 = out
            .iter()
            .find(|r| r.start.with_timezone(&tz).day() == 25)
            .expect("oct 25 present");
        let earliest = resolve_local(tz, naive(2026, 10, 25, 2, 30)).expect("resolves");
        assert_eq!(oct25.start, earliest);
    }

    // --- proptest properties ---

    proptest! {
        #[test]
        fn count_never_exceeds_max(count in 1u16..200, cap in 1u16..150) {
            let mut spec = base(&format!("RRULE:FREQ=DAILY;COUNT={count}"), naive(2026, 1, 1, 9, 0));
            spec.max_count = cap;
            match expand_series(&spec) {
                Ok(out) => {
                    prop_assert!(out.len() <= usize::from(cap));
                    prop_assert!(out.len() <= usize::from(count));
                    // A series that ended naturally (count <= cap) is Ok — even
                    // when count == cap exactly (no spurious Truncated).
                    prop_assert!(count <= cap);
                }
                Err(ExpandError::Truncated(out)) => {
                    // Truncated fires only when the rule genuinely produces more
                    // than the cap (strictly count > cap).
                    prop_assert_eq!(out.len(), usize::from(cap));
                    prop_assert!(count > cap);
                }
                Err(e) => prop_assert!(false, "unexpected error: {e}"),
            }
        }

        #[test]
        fn every_occurrence_has_positive_duration(count in 1u16..50, mins in 1i64..600) {
            let mut spec = base(&format!("RRULE:FREQ=DAILY;COUNT={count}"), naive(2026, 1, 1, 9, 0));
            spec.duration = Duration::minutes(mins);
            let out = match expand_series(&spec) {
                Ok(o) | Err(ExpandError::Truncated(o)) => o,
                Err(e) => return Err(TestCaseError::fail(format!("unexpected error: {e}"))),
            };
            for r in &out {
                prop_assert!(r.is_valid());
                prop_assert_eq!(r.end - r.start, Duration::minutes(mins));
            }
        }

        #[test]
        fn exdate_wins_over_rrule(count in 2u16..20, skip in 0u16..2) {
            // Remove the (skip+1)-th daily instance; it must not appear.
            let day = 1 + i64::from(skip) + 1; // 1-indexed day-of-month target
            let mut spec = base(&format!("RRULE:FREQ=DAILY;COUNT={count}"), naive(2026, 1, 1, 9, 0));
            let day_u = u32::try_from(day).map_err(|e| TestCaseError::fail(e.to_string()))?;
            let target = naive(2026, 1, day_u, 9, 0);
            spec.exdates_local = vec![target];
            let items = match expand_series(&spec) {
                Ok(o) | Err(ExpandError::Truncated(o)) => o,
                Err(e) => return Err(TestCaseError::fail(format!("unexpected error: {e}"))),
            };
            let tz: ChronoTz = AMS.parse().expect("tz");
            let removed = resolve_local(tz, target).expect("resolves");
            prop_assert!(items.iter().all(|r| r.start != removed));
        }

        #[test]
        fn results_are_strictly_increasing(count in 1u16..60) {
            let spec = base(&format!("RRULE:FREQ=DAILY;COUNT={count}"), naive(2026, 1, 1, 9, 0));
            let out = expand_series(&spec).expect("ok");
            for w in out.windows(2) {
                prop_assert!(w[0].start < w[1].start);
            }
        }
    }
}
