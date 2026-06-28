//! Stateless date helpers for the date pickers, built on the `time` crate.

use time::{Date, Month};

/// Date utilities shared by the date-picker components.
pub(crate) struct DateUtils;

impl DateUtils {
    /// Parses an ISO `YYYY-MM-DD` value from a URL query parameter, returning
    /// `None` for absent or malformed input.
    pub(crate) fn parse_from_url(value: Option<String>) -> Option<Date> {
        let value = value?;
        let mut parts = value.split('-');
        let year: i32 = parts.next()?.parse().ok()?;
        let month: u8 = parts.next()?.parse().ok()?;
        let day: u8 = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()
    }

    /// The month and year immediately before `month`/`year`, wrapping January
    /// back to the previous December.
    pub(crate) fn prev_month_year(month: Month, year: i32) -> (Month, i32) {
        match month {
            Month::January => (Month::December, year - 1),
            other => (other.previous(), year),
        }
    }

    /// The month and year immediately after `month`/`year`, wrapping December
    /// forward to the next January.
    pub(crate) fn next_month_year(month: Month, year: i32) -> (Month, i32) {
        match month {
            Month::December => (Month::January, year + 1),
            other => (other.next(), year),
        }
    }
}
