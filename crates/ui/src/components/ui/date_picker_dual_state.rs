use leptos::prelude::*;
use time::{Date, Month};

use crate::utils::date::DateUtils;
use crate::utils::query::{Query, QueryUtils};

/// Selected start/end dates for the dual-month range picker.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct DatePickerDualState {
    pub start_date: Date,
    pub end_date: Date,
}

impl DatePickerDualState {
    /// Builds a state from explicit dates.
    pub fn new(start_date: Date, end_date: Date) -> Self {
        Self {
            start_date,
            end_date,
        }
    }

    /// Seeds the range from the URL `start_date`/`end_date` parameters (falling
    /// back to a fixed default range). Returns the state signal and a setup
    /// closure that wires an effect keeping it in sync with the URL.
    pub fn from_url_or_default() -> (RwSignal<Self>, impl Fn() + Clone) {
        let start_query = QueryUtils::extract(Query::START_DATE.to_string());
        let end_query = QueryUtils::extract(Query::END_DATE.to_string());

        let initial = Memo::new(move |_| {
            let fallback_start = Date::from_calendar_date(2025, Month::May, 5).unwrap_or(Date::MIN);
            let fallback_end = Date::from_calendar_date(2025, Month::May, 14).unwrap_or(Date::MIN);
            let start_date = DateUtils::parse_from_url(start_query.get()).unwrap_or(fallback_start);
            let end_date = DateUtils::parse_from_url(end_query.get()).unwrap_or(fallback_end);
            Self::new(start_date, end_date)
        });

        let state = RwSignal::new(initial.get());
        let setup_url_sync = move || {
            Effect::new(move |_| state.set(initial.get()));
        };

        (state, setup_url_sync)
    }

    /// Extends the range toward `day`: a click on or before the current start
    /// moves the start, otherwise it moves the end.
    pub fn handle_day_selection(&mut self, day: u8, month: Month, year: i32) {
        if day == 0 {
            return;
        }
        let Ok(new_date) = Date::from_calendar_date(year, month, day) else {
            return;
        };
        if new_date <= self.start_date {
            self.start_date = new_date;
        } else {
            self.end_date = new_date;
        }
    }

    /// The month/year to show for a pane: `0` is the primary month, anything
    /// else is the following month.
    pub fn get_display_month(display_date: Date, month_offset: i32) -> (Month, i32) {
        if month_offset == 0 {
            (display_date.month(), display_date.year())
        } else {
            DateUtils::next_month_year(display_date.month(), display_date.year())
        }
    }

    /// Whether `date` is exactly the start or end of the range.
    pub fn is_start_or_end_date(&self, date: Date) -> bool {
        date == self.start_date || date == self.end_date
    }

    /// The padded calendar grid for `month`/`year` (Sunday-first); each entry is
    /// `(day, month, year, is_selected, is_adjacent_month)`.
    pub fn calculate_calendar_data(year: i32, month: Month) -> Vec<(u8, Month, i32, bool, bool)> {
        let Ok(first_day) = Date::from_calendar_date(year, month, 1) else {
            return Vec::new();
        };
        let first_weekday = first_day.weekday().number_from_sunday() - 1;

        let (prev_month, prev_year) = DateUtils::prev_month_year(month, year);
        let (next_month, next_year) = DateUtils::next_month_year(month, year);
        let days_in_prev_month = prev_month.length(prev_year);
        let days_in_month = month.length(year);

        let mut days = Vec::new();
        for offset in 0..first_weekday {
            let day = days_in_prev_month - first_weekday + offset + 1;
            days.push((day, prev_month, prev_year, false, true));
        }
        for day in 1..=days_in_month {
            days.push((day, month, year, false, false));
        }
        let trailing = u8::try_from((7 - days.len() % 7) % 7).unwrap_or(0);
        for day in 1..=trailing {
            days.push((day, next_month, next_year, false, true));
        }

        days
    }
}
