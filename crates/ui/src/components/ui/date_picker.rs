use crate::utils::date::DateUtils;
use crate::{DatePickerDay, DatePickerState, clx, cn};
use leptos::prelude::*;
use leptos_icons::Icon;
use time::{Date, Month};

clx! {
    /// Calendar surface containing the header and month grid.
    DatePicker, div, "flex flex-col gap-4 p-3 rounded-lg border bg-card text-card-foreground shadow-sm w-fit"
}
clx! {
    /// Previous/next-month navigation button in the [`DatePickerHeader`].
    DatePickerNavButton, button, "inline-flex items-center justify-center p-0 text-sm font-medium transition-colors bg-transparent border rounded-md opacity-50 whitespace-nowrap ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 disabled:cursor-not-allowed border-input hover:bg-accent hover:text-accent-foreground size-7 hover:opacity-100 [&_svg:not([class*='size-'])]:size-4"
}
clx! {
    /// Centered month/year title in the [`DatePickerHeader`].
    DatePickerTitle, span, "text-sm font-medium text-center"
}
clx! {
    /// Header row: previous button, title and next button.
    DatePickerHeader, header, "grid grid-cols-[auto_1fr_auto] items-center pt-1"
}
clx! {
    /// Weekday column heading in the calendar table.
    DatePickerWeekDay, th, "text-muted-foreground rounded-md w-9 font-normal text-[0.8rem]"
}
clx! {
    /// Heading cell for the optional week-number column.
    DatePickerWeekNumberHeader, th, "text-muted-foreground rounded-md w-6 font-normal text-[0.8rem] select-none"
}
clx! {
    /// Week-number cell at the start of a calendar row.
    DatePickerWeekNumberCell, td, "w-6 text-center text-[0.8rem] text-muted-foreground select-none"
}
clx! {
    /// A week row of day cells.
    DatePickerRow, tr, "flex w-full mt-2"
}
clx! {
    /// Vertical stack wrapping a single month's header and table.
    DatePickerMonth, div, "flex flex-col items-center justify-start gap-2 size-full"
}
clx! {
    /// The calendar table holding the weekday headings and day rows.
    DatePickerTable, table, "w-full space-y-1 border-collapse"
}

const DATE_PICKER_CELL_BASE: &str = "inline-flex items-center justify-center text-sm size-9 rounded-md select-none hover:cursor-pointer hover:bg-accent aria-disabled:pointer-events-none aria-disabled:opacity-50 aria-disabled:cursor-not-allowed aria-current:bg-primary aria-current:hover:bg-primary aria-current:text-primary-foreground";

/// A single day cell. Highlights the start/end dates (`aria-current`) and the
/// in-range days between them, and reports clicks via `on_click`.
#[component]
pub fn DatePickerCell(
    day: u8,
    year: i32,
    month: Month,
    disabled: bool,
    start_date: RwSignal<Date>,
    end_date: RwSignal<Date>,
    on_click: impl Fn(u8) + 'static,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let current_date = (day > 0 && !disabled)
        .then(|| Date::from_calendar_date(year, month, day).ok())
        .flatten();

    let is_current =
        move || current_date.is_some_and(|date| date == start_date.get() || date == end_date.get());
    let is_selected =
        move || current_date.is_some_and(|date| date > start_date.get() && date < end_date.get());

    let cell_class = move || {
        let range = if is_selected() {
            "bg-accent rounded-none"
        } else {
            ""
        };
        cn!(DATE_PICKER_CELL_BASE, range, class.get())
    };

    let handle_click = move |_| {
        if !disabled {
            on_click(day);
        }
    };

    view! {
        <td
            data-name="DatePickerCell"
            class=cell_class
            aria-current=move || is_current().to_string()
            aria-disabled=disabled.to_string()
            on:click=handle_click
        >
            {day}
        </td>
    }
}

const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

/// English month name for a calendar title, avoiding a dependency on `Month`'s
/// `Display` formatting.
fn month_name(month: Month) -> &'static str {
    match month {
        Month::January => "January",
        Month::February => "February",
        Month::March => "March",
        Month::April => "April",
        Month::May => "May",
        Month::June => "June",
        Month::July => "July",
        Month::August => "August",
        Month::September => "September",
        Month::October => "October",
        Month::November => "November",
        Month::December => "December",
    }
}

/// First day of the month `offset` months away from `from`, saturating to the
/// 1st so the result is always a valid date.
fn shift_month(from: Date, forward: bool) -> Date {
    let (month, year) = if forward {
        DateUtils::next_month_year(from.month(), from.year())
    } else {
        DateUtils::prev_month_year(from.month(), from.year())
    };
    Date::from_calendar_date(year, month, 1).unwrap_or(from)
}

/// Self-contained, interactive single-month calendar assembled from the
/// date-picker primitives. It tracks the visible month internally, renders the
/// weekday headings and a Monday-first day grid via
/// [`DatePickerState::get_calendar_days`], and writes the chosen day into
/// `start_date`/`end_date`.
///
/// Pass one shared signal as both `start_date` and `end_date` for single-date
/// selection, or two distinct signals with `range=true` to pick an inclusive
/// range (the cells between the ends are highlighted). `on_select` fires with
/// the full [`Date`] each time a day is chosen.
#[component]
pub fn Calendar(
    /// Start of the highlighted selection.
    start_date: RwSignal<Date>,
    /// End of the highlighted selection; equal to `start_date` for a single date.
    end_date: RwSignal<Date>,
    /// When set, the second click of each pair extends the range instead of
    /// replacing the selection.
    #[prop(optional)]
    range: bool,
    /// Fires with the chosen date on every selection.
    #[prop(into, optional)]
    on_select: Option<Callback<Date>>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let displayed = RwSignal::new(start_date.get_untracked());
    let selecting_end = RwSignal::new(false);

    let select_day = move |day: u8, year: i32, month: Month| {
        let Ok(date) = Date::from_calendar_date(year, month, day) else {
            return;
        };
        if range {
            if selecting_end.get_untracked() {
                if date < start_date.get_untracked() {
                    end_date.set(start_date.get_untracked());
                    start_date.set(date);
                } else {
                    end_date.set(date);
                }
                selecting_end.set(false);
            } else {
                start_date.set(date);
                end_date.set(date);
                selecting_end.set(true);
            }
        } else {
            start_date.set(date);
            end_date.set(date);
        }
        if let Some(callback) = on_select {
            callback.run(date);
        }
    };

    let go_prev = move |_| displayed.update(|d| *d = shift_month(*d, false));
    let go_next = move |_| displayed.update(|d| *d = shift_month(*d, true));
    let title = move || {
        let d = displayed.get();
        format!("{} {}", month_name(d.month()), d.year())
    };

    let grid = move || {
        let shown = displayed.get();
        let year = shown.year();
        let month = shown.month();
        let weeks = DatePickerState::get_calendar_days(year, month)
            .chunks(7)
            .map(|week| {
                let cells = week
                    .iter()
                    .copied()
                    .map(|DatePickerDay { day, disabled }| {
                        view! {
                            <DatePickerCell
                                day=day
                                year=year
                                month=month
                                disabled=disabled
                                start_date=start_date
                                end_date=end_date
                                on_click=move |clicked| select_day(clicked, year, month)
                            />
                        }
                    })
                    .collect_view();
                view! { <DatePickerRow>{cells}</DatePickerRow> }
            })
            .collect_view();
        view! { <tbody>{weeks}</tbody> }
    };

    let headings = WEEKDAYS
        .iter()
        .map(|label| view! { <DatePickerWeekDay>{*label}</DatePickerWeekDay> })
        .collect_view();

    view! {
        <DatePicker class=class>
            <DatePickerHeader>
                <DatePickerNavButton
                    attr:r#type="button"
                    attr:aria-label="Previous month"
                    on:click=go_prev
                >
                    <Icon icon=icondata::LuChevronLeft />
                </DatePickerNavButton>
                <DatePickerTitle>{title}</DatePickerTitle>
                <DatePickerNavButton
                    attr:r#type="button"
                    attr:aria-label="Next month"
                    on:click=go_next
                >
                    <Icon icon=icondata::LuChevronRight />
                </DatePickerNavButton>
            </DatePickerHeader>
            <DatePickerMonth>
                <DatePickerTable>
                    <thead>
                        <tr class="flex">{headings}</tr>
                    </thead>
                    {grid}
                </DatePickerTable>
            </DatePickerMonth>
        </DatePicker>
    }
}
