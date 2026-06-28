use leptos::prelude::*;
use time::{Date, Month};

use crate::utils::date::DateUtils;
use crate::utils::query::{Query, QueryUtils};

/// A day cell in the single-month calendar grid.
#[derive(Debug, Clone, Copy)]
pub struct DatePickerDay {
    pub day: u8,
    pub disabled: bool,
}

/// Selected start/end dates for a [`DatePicker`](super::DatePicker).
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct DatePickerState {
    pub start_date: Date,
    pub end_date: Date,
}

impl DatePickerState {
    /// Builds a state from explicit dates.
    pub fn new(start_date: Date, end_date: Date) -> Self {
        Self {
            start_date,
            end_date,
        }
    }

    /// Seeds the state from the `start_date`/`end_date` URL parameters, falling
    /// back to the given defaults and clearing the URL when the range is invalid;
    /// keeps the signal in sync as those parameters change.
    pub fn from_url_or_default(default_start: Date, default_end: Date) -> RwSignal<Self> {
        let start_query = QueryUtils::extract(Query::START_DATE.to_string());
        let end_query = QueryUtils::extract(Query::END_DATE.to_string());
        let state = RwSignal::new(Self::new(default_start, default_end));

        Effect::new(move |_| {
            let parsed_start = DateUtils::parse_from_url(start_query.get());
            let parsed_end = DateUtils::parse_from_url(end_query.get());

            let invalid = matches!((parsed_start, parsed_end), (Some(s), Some(e)) if s > e);
            let (start_date, end_date) = if invalid {
                QueryUtils::update_dates_url(None, None);
                (default_start, default_end)
            } else {
                (
                    parsed_start.unwrap_or(default_start),
                    parsed_end.unwrap_or(default_end),
                )
            };

            state.set(Self::new(start_date, end_date));
        });

        state
    }

    /// The calendar grid for `month`/`year`, padded with disabled leading and
    /// trailing days so it fills whole weeks (Monday-first).
    pub fn get_calendar_days(year: i32, month: Month) -> Vec<DatePickerDay> {
        let Ok(first_day) = Date::from_calendar_date(year, month, 1) else {
            return Vec::new();
        };
        let first_weekday = first_day.weekday().number_from_monday() - 1;

        let (prev_month, prev_year) = DateUtils::prev_month_year(month, year);
        let days_in_prev_month = prev_month.length(prev_year);
        let days_in_month = month.length(year);

        let mut days = Vec::new();
        for offset in 0..first_weekday {
            let day = days_in_prev_month - first_weekday + offset + 1;
            days.push(DatePickerDay {
                day,
                disabled: true,
            });
        }
        for day in 1..=days_in_month {
            days.push(DatePickerDay {
                day,
                disabled: false,
            });
        }
        let trailing = u8::try_from((7 - days.len() % 7) % 7).unwrap_or(0);
        for day in 1..=trailing {
            days.push(DatePickerDay {
                day,
                disabled: true,
            });
        }

        days
    }
}
