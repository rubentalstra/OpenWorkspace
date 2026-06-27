//! A half-open UTC time interval `[start, end)`.

use chrono::{DateTime, Utc};

/// A half-open `[start, end)` interval in UTC.
///
/// This mirrors the canonical `tstzrange(lower, upper, '[)')` stored in
/// `booking_occurrences.period`: two ranges that merely *touch*
/// (`a.end == b.start`) do **not** overlap, exactly like the `GiST` exclusion
/// predicate. By construction callers should keep `start < end`; an empty or
/// inverted range simply never overlaps anything.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TimeRange {
    /// Inclusive lower bound.
    pub start: DateTime<Utc>,
    /// Exclusive upper bound.
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// Constructs a range from its bounds. No ordering check is performed; an
    /// inverted range is treated as empty by [`TimeRange::overlaps`].
    #[must_use]
    pub const fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    /// Returns `true` if this range and `other` share any instant under
    /// half-open `[)` semantics.
    ///
    /// `a.start < b.end && b.start < a.end` — touching ranges never overlap.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Returns `true` if `start < end` (a proper, non-empty interval).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.start < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use proptest::prelude::*;

    fn at(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(secs, 0).single().expect("valid instant")
    }

    fn range(start: i64, end: i64) -> TimeRange {
        TimeRange::new(at(start), at(end))
    }

    #[test]
    fn touching_ranges_do_not_overlap() {
        // [0,10) and [10,20) share only the boundary instant.
        let a = range(0, 10);
        let b = range(10, 20);
        assert!(!a.overlaps(&b));
        assert!(!b.overlaps(&a));
    }

    #[test]
    fn one_second_overlap_counts() {
        let a = range(0, 10);
        let b = range(9, 20);
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
    }

    #[test]
    fn reflexive_overlap_for_nonempty() {
        let a = range(0, 10);
        assert!(a.overlaps(&a));
    }

    // Strategy producing a proper [start, end) range.
    prop_compose! {
        fn arb_range()(start in -1_000_000i64..1_000_000, len in 1i64..1_000_000)
            -> TimeRange {
            range(start, start + len)
        }
    }

    proptest! {
        #[test]
        fn overlap_is_reflexive(r in arb_range()) {
            prop_assert!(r.overlaps(&r));
        }

        #[test]
        fn overlap_is_symmetric(a in arb_range(), b in arb_range()) {
            prop_assert_eq!(a.overlaps(&b), b.overlaps(&a));
        }

        #[test]
        fn touching_never_overlaps(start in -1_000_000i64..1_000_000, l1 in 1i64..100_000, l2 in 1i64..100_000) {
            let a = range(start, start + l1);
            let b = range(start + l1, start + l1 + l2);
            prop_assert!(!a.overlaps(&b));
            prop_assert!(!b.overlaps(&a));
        }
    }
}
