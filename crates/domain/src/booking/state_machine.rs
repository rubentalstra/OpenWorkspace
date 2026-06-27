//! The pure occurrence state machine.
//!
//! [`transition`] is total and clock-injected: it never panics, never reads a
//! clock, and returns either a [`Transition`] (the status change plus the
//! timestamp the DB's status↔timestamp `CHECK`s require, plus an optional
//! shrunk period) or a typed [`TransitionError`].

use chrono::{DateTime, Duration, Utc};

use crate::enums::BookingStatus;
use crate::time_range::TimeRange;

/// An action applied to an occurrence.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BookingEvent {
    /// A user checks in to claim the resource.
    CheckIn,
    /// A user checks out, freeing the remaining tail of the period.
    CheckOut,
    /// A user cancels before the booking becomes active.
    Cancel,
    /// The worker auto-releases an un-checked-in booking after the grace window.
    AutoRelease,
    /// An operator marks the booking as a no-show.
    MarkNoShow,
}

/// The occurrence's current scheduling facts, independent of policy/clock.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OccurrenceContext {
    /// The occurrence's scheduled half-open period in UTC.
    pub period: TimeRange,
}

/// Time-based gates governing legal transitions. A `None` field disables that
/// gate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TransitionPolicy {
    /// How long before `period.start` check-in is permitted. Check-in earlier
    /// than `start - check_in_window` is rejected.
    pub check_in_window: Duration,
    /// Grace after `period.start` before an un-checked-in booking may be
    /// auto-released. Auto-release earlier than `start + grace` is rejected.
    pub grace: Duration,
    /// How long before `period.start` a cancellation is still allowed.
    /// `Some(d)` ⇒ cancelling after `start - d` is rejected. `None` ⇒ no
    /// deadline (cancel any time while `Booked`).
    pub cancellation_deadline: Option<Duration>,
}

/// Which timestamp column a transition stamps, and with what value. Mirrors the
/// schema's status↔timestamp `CHECK` constraints: each terminal/active status
/// carries exactly the stamp it requires.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stamp {
    /// `check_in_at` ⇐ value (status `checked_in`).
    CheckedInAt(DateTime<Utc>),
    /// `checked_out_at` ⇐ value (status `checked_out`).
    CheckedOutAt(DateTime<Utc>),
    /// `cancelled_at` ⇐ value (status `cancelled`).
    CancelledAt(DateTime<Utc>),
    /// `auto_released_at` ⇐ value (status `released`).
    AutoReleasedAt(DateTime<Utc>),
    /// No timestamp required (status `no_show`).
    None,
}

/// The result of a legal transition: the new status, the stamp the DB must
/// write, and an optional shrunk period (only checkout truncates).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Transition {
    /// Status to persist.
    pub new_status: BookingStatus,
    /// Timestamp the status↔timestamp `CHECK` requires for `new_status`.
    pub stamp: Stamp,
    /// New period to persist, if the transition shrinks it (checkout only).
    pub new_period: Option<TimeRange>,
}

/// Why a transition was rejected.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransitionError {
    /// The `(state, event)` pair is not a legal edge.
    IllegalTransition {
        /// The state the occurrence was in.
        from: BookingStatus,
        /// The event that was attempted.
        event: BookingEvent,
    },
    /// Check-in attempted before `period.start - check_in_window`.
    TooEarlyToCheckIn,
    /// Auto-release attempted before `period.start + grace`.
    GraceNotElapsed,
    /// Cancellation attempted after the cancellation deadline.
    CancellationDeadlinePassed,
}

impl core::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IllegalTransition { from, event } => {
                write!(f, "illegal transition from {from:?} on {event:?}")
            }
            Self::TooEarlyToCheckIn => write!(f, "check-in not yet open"),
            Self::GraceNotElapsed => write!(f, "grace window has not elapsed"),
            Self::CancellationDeadlinePassed => write!(f, "cancellation deadline passed"),
        }
    }
}

impl core::error::Error for TransitionError {}

/// Computes the legal transition for `(state, event)` given the occurrence
/// context, policy gates and the injected clock `now`.
///
/// Total: every input yields `Ok(Transition)` or a typed `Err`; it never
/// panics. Terminal statuses accept no event.
///
/// # Errors
///
/// Returns [`TransitionError`] for an illegal edge or a failed time gate.
pub fn transition(
    state: BookingStatus,
    event: BookingEvent,
    occ: OccurrenceContext,
    policy: TransitionPolicy,
    now: DateTime<Utc>,
) -> Result<Transition, TransitionError> {
    use BookingEvent::{AutoRelease, Cancel, CheckIn, CheckOut, MarkNoShow};
    use BookingStatus::{Booked, CheckedIn};

    let illegal = Err(TransitionError::IllegalTransition { from: state, event });

    match (state, event) {
        // Booked → CheckedIn: rejected before the check-in window opens.
        (Booked, CheckIn) => {
            let opens_at = occ.period.start - policy.check_in_window;
            if now < opens_at {
                return Err(TransitionError::TooEarlyToCheckIn);
            }
            Ok(Transition {
                new_status: CheckedIn,
                stamp: Stamp::CheckedInAt(now),
                new_period: None,
            })
        }

        // Booked → Released (auto): valid only once the grace window elapses.
        (Booked, AutoRelease) => {
            let releasable_at = occ.period.start + policy.grace;
            if now < releasable_at {
                return Err(TransitionError::GraceNotElapsed);
            }
            Ok(Transition {
                new_status: BookingStatus::Released,
                stamp: Stamp::AutoReleasedAt(now),
                new_period: None,
            })
        }

        // Booked → Cancelled: enforce the cancellation deadline if set.
        (Booked, Cancel) => {
            if let Some(deadline) = policy.cancellation_deadline {
                let cutoff = occ.period.start - deadline;
                if now > cutoff {
                    return Err(TransitionError::CancellationDeadlinePassed);
                }
            }
            Ok(Transition {
                new_status: BookingStatus::Cancelled,
                stamp: Stamp::CancelledAt(now),
                new_period: None,
            })
        }

        // Booked → NoShow: no time gate, no stamp required by the schema.
        (Booked, MarkNoShow) => Ok(Transition {
            new_status: BookingStatus::NoShow,
            stamp: Stamp::None,
            new_period: None,
        }),

        // CheckedIn → CheckedOut: truncate the tail to free the resource.
        // A checkout at or after `end` is a no-op shrink (period unchanged).
        //
        // Check-in is legal before `start`, and CheckOut has no time gate, so a
        // checkout can occur at or before `start`. The truncated end is clamped
        // into `[start, end]` so it can never precede `start` (which would yield
        // the inverted/empty `tstzrange` the DB rejects with 22000/23514). When
        // the checkout lands at or before `start`, the clamp collapses the
        // truncation to a zero tail: the booking never became active, so the
        // whole period `[start, end)` is preserved unchanged (nothing freed
        // early), keeping the stored range non-empty and half-open valid.
        (CheckedIn, CheckOut) => {
            let clamped = now.clamp(occ.period.start, occ.period.end);
            // `clamped == start` means `now <= start`: keep the full period so
            // the range stays non-empty (`start < end`).
            let new_end = if clamped <= occ.period.start {
                occ.period.end
            } else {
                clamped
            };
            Ok(Transition {
                new_status: BookingStatus::CheckedOut,
                stamp: Stamp::CheckedOutAt(now),
                new_period: Some(TimeRange::new(occ.period.start, new_end)),
            })
        }

        // Everything else (incl. all events on terminal states) is illegal.
        _ => illegal,
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

    // Period [1000, 2000) in epoch seconds.
    fn ctx() -> OccurrenceContext {
        OccurrenceContext {
            period: TimeRange::new(at(1000), at(2000)),
        }
    }

    fn policy() -> TransitionPolicy {
        TransitionPolicy {
            check_in_window: Duration::seconds(100), // check-in opens at 900
            grace: Duration::seconds(100),           // releasable at 1100
            cancellation_deadline: Some(Duration::seconds(200)), // cutoff at 800
        }
    }

    // --- check-in window gate ---

    #[test]
    fn check_in_rejected_before_window_opens() {
        let err = transition(
            BookingStatus::Booked,
            BookingEvent::CheckIn,
            ctx(),
            policy(),
            at(899),
        )
        .unwrap_err();
        assert_eq!(err, TransitionError::TooEarlyToCheckIn);
    }

    #[test]
    fn check_in_allowed_exactly_at_window_open() {
        let t = transition(
            BookingStatus::Booked,
            BookingEvent::CheckIn,
            ctx(),
            policy(),
            at(900),
        )
        .unwrap();
        assert_eq!(t.new_status, BookingStatus::CheckedIn);
        assert_eq!(t.stamp, Stamp::CheckedInAt(at(900)));
        assert!(t.new_period.is_none());
    }

    // --- grace / auto-release gate ---

    #[test]
    fn auto_release_rejected_before_grace() {
        let err = transition(
            BookingStatus::Booked,
            BookingEvent::AutoRelease,
            ctx(),
            policy(),
            at(1099),
        )
        .unwrap_err();
        assert_eq!(err, TransitionError::GraceNotElapsed);
    }

    #[test]
    fn auto_release_allowed_at_grace_boundary() {
        let t = transition(
            BookingStatus::Booked,
            BookingEvent::AutoRelease,
            ctx(),
            policy(),
            at(1100),
        )
        .unwrap();
        assert_eq!(t.new_status, BookingStatus::Released);
        assert_eq!(t.stamp, Stamp::AutoReleasedAt(at(1100)));
    }

    // --- cancellation deadline gate ---

    #[test]
    fn cancel_allowed_before_deadline() {
        let t = transition(
            BookingStatus::Booked,
            BookingEvent::Cancel,
            ctx(),
            policy(),
            at(800),
        )
        .unwrap();
        assert_eq!(t.new_status, BookingStatus::Cancelled);
        assert_eq!(t.stamp, Stamp::CancelledAt(at(800)));
    }

    #[test]
    fn cancel_rejected_after_deadline() {
        let err = transition(
            BookingStatus::Booked,
            BookingEvent::Cancel,
            ctx(),
            policy(),
            at(801),
        )
        .unwrap_err();
        assert_eq!(err, TransitionError::CancellationDeadlinePassed);
    }

    #[test]
    fn cancel_with_no_deadline_always_allowed() {
        let mut p = policy();
        p.cancellation_deadline = None;
        let t = transition(
            BookingStatus::Booked,
            BookingEvent::Cancel,
            ctx(),
            p,
            at(1999),
        )
        .unwrap();
        assert_eq!(t.new_status, BookingStatus::Cancelled);
    }

    #[test]
    fn no_show_carries_no_stamp() {
        let t = transition(
            BookingStatus::Booked,
            BookingEvent::MarkNoShow,
            ctx(),
            policy(),
            at(2500),
        )
        .unwrap();
        assert_eq!(t.new_status, BookingStatus::NoShow);
        assert_eq!(t.stamp, Stamp::None);
    }

    // --- checkout truncation ---

    #[test]
    fn checkout_truncates_tail() {
        let t = transition(
            BookingStatus::CheckedIn,
            BookingEvent::CheckOut,
            ctx(),
            policy(),
            at(1500),
        )
        .unwrap();
        assert_eq!(t.new_status, BookingStatus::CheckedOut);
        assert_eq!(t.stamp, Stamp::CheckedOutAt(at(1500)));
        let period = t.new_period.expect("checkout shrinks the period");
        assert_eq!(period.start, at(1000));
        assert_eq!(period.end, at(1500));
    }

    #[test]
    fn checkout_before_start_clamps_to_start() {
        // Check-in is legal before start; checkout has no gate, so checking out
        // before start must NOT produce an inverted/empty range. The clamp keeps
        // the whole period `[start, end)` (nothing freed early, since the booking
        // never became active), and the range stays non-empty (start < end).
        let t = transition(
            BookingStatus::CheckedIn,
            BookingEvent::CheckOut,
            ctx(), // period [1000, 2000)
            policy(),
            at(950), // now < start (1000)
        )
        .unwrap();
        assert_eq!(t.new_status, BookingStatus::CheckedOut);
        let period = t.new_period.expect("checkout shrinks the period");
        // new_period == the full original period [1000, 2000).
        assert_eq!(period.start, at(1000));
        assert_eq!(period.end, at(2000));
        assert!(period.start < period.end, "period must be non-empty");
    }

    #[test]
    fn checkout_after_end_is_noop_shrink() {
        // now >= end: the period is unchanged (never grown past `end`).
        let t = transition(
            BookingStatus::CheckedIn,
            BookingEvent::CheckOut,
            ctx(),
            policy(),
            at(2500),
        )
        .unwrap();
        let period = t.new_period.expect("period present");
        assert_eq!(period.end, at(2000));
    }

    // --- illegal edges ---

    #[test]
    fn checkout_from_booked_is_illegal() {
        let err = transition(
            BookingStatus::Booked,
            BookingEvent::CheckOut,
            ctx(),
            policy(),
            at(1500),
        )
        .unwrap_err();
        assert_eq!(
            err,
            TransitionError::IllegalTransition {
                from: BookingStatus::Booked,
                event: BookingEvent::CheckOut,
            }
        );
    }

    #[test]
    fn terminal_states_accept_nothing() {
        let terminals = [
            BookingStatus::CheckedOut,
            BookingStatus::Released,
            BookingStatus::NoShow,
            BookingStatus::Cancelled,
        ];
        let events = [
            BookingEvent::CheckIn,
            BookingEvent::CheckOut,
            BookingEvent::Cancel,
            BookingEvent::AutoRelease,
            BookingEvent::MarkNoShow,
        ];
        for state in terminals {
            assert!(state.is_terminal());
            for event in events {
                let r = transition(state, event, ctx(), policy(), at(1500));
                assert!(matches!(r, Err(TransitionError::IllegalTransition { .. })));
            }
        }
    }

    // --- proptest: totality + stamp/status agreement ---

    fn arb_status() -> impl Strategy<Value = BookingStatus> {
        prop_oneof![
            Just(BookingStatus::Booked),
            Just(BookingStatus::CheckedIn),
            Just(BookingStatus::CheckedOut),
            Just(BookingStatus::Released),
            Just(BookingStatus::NoShow),
            Just(BookingStatus::Cancelled),
        ]
    }

    fn arb_event() -> impl Strategy<Value = BookingEvent> {
        prop_oneof![
            Just(BookingEvent::CheckIn),
            Just(BookingEvent::CheckOut),
            Just(BookingEvent::Cancel),
            Just(BookingEvent::AutoRelease),
            Just(BookingEvent::MarkNoShow),
        ]
    }

    /// The stamp must correspond to the status (mirroring the DB CHECKs).
    fn stamp_matches(status: BookingStatus, stamp: Stamp) -> bool {
        match status {
            BookingStatus::CheckedIn => matches!(stamp, Stamp::CheckedInAt(_)),
            BookingStatus::CheckedOut => matches!(stamp, Stamp::CheckedOutAt(_)),
            BookingStatus::Cancelled => matches!(stamp, Stamp::CancelledAt(_)),
            BookingStatus::Released => matches!(stamp, Stamp::AutoReleasedAt(_)),
            BookingStatus::NoShow => matches!(stamp, Stamp::None),
            BookingStatus::Booked => false, // never a transition target here
        }
    }

    proptest! {
        #[test]
        fn transition_is_total_and_consistent(
            status in arb_status(),
            event in arb_event(),
            start in -10_000i64..10_000,
            len in 1i64..10_000,
            win in 0i64..5_000,
            grace in 0i64..5_000,
            deadline in proptest::option::of(0i64..5_000),
            now in -20_000i64..20_000,
        ) {
            let occ = OccurrenceContext {
                period: TimeRange::new(at(start), at(start + len)),
            };
            let pol = TransitionPolicy {
                check_in_window: Duration::seconds(win),
                grace: Duration::seconds(grace),
                cancellation_deadline: deadline.map(Duration::seconds),
            };
            // Never panics; yields a legal Transition or a typed error. Errors
            // need no further checks — only successes carry invariants.
            if let Ok(t) = transition(status, event, occ, pol, at(now)) {
                // Target is never Booked, and stamp agrees with status.
                prop_assert_ne!(t.new_status, BookingStatus::Booked);
                prop_assert!(stamp_matches(t.new_status, t.stamp));
                // Only checkout shrinks the period, and it never grows it.
                if let Some(p) = t.new_period {
                    prop_assert_eq!(t.new_status, BookingStatus::CheckedOut);
                    prop_assert!(p.end <= occ.period.end);
                    prop_assert_eq!(p.start, occ.period.start);
                    // The shrunk period must stay non-empty (start < end): the DB
                    // CHECK rejects empty/inverted tstzranges. A checkout at or
                    // before start must never yield such a range.
                    prop_assert!(p.start < p.end);
                }
                // Terminal source states can never produce a transition.
                prop_assert!(!status.is_terminal());
            }
        }
    }
}
