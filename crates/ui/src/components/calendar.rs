use crate::cn;
use leptos::prelude::*;
use leptos_icons::Icon;
use time::{Date, Month, OffsetDateTime, util};

/// Sunday-based weekday header labels, matching react-day-picker's default
/// (`showOutsideDays`, week starts Sunday).
const WEEKDAY_LABELS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

/// First day of the month containing `date`.
fn first_of_month(date: Date) -> Date {
    Date::from_calendar_date(date.year(), date.month(), 1).unwrap_or(date)
}

/// Today's date (UTC), used for the `today` marker.
fn today() -> Date {
    OffsetDateTime::now_utc().date()
}

/// Step `anchor` (a first-of-month date) one month back or forward, rolling the
/// year over at the December/January boundary. Clamps the day to 1 so the result
/// stays a valid first-of-month.
fn shift_month(anchor: Date, forward: bool) -> Date {
    let (year, month) = if forward {
        let next = anchor.month().next();
        let year = if matches!(next, Month::January) {
            anchor.year() + 1
        } else {
            anchor.year()
        };
        (year, next)
    } else {
        let prev = anchor.month().previous();
        let year = if matches!(prev, Month::December) {
            anchor.year() - 1
        } else {
            anchor.year()
        };
        (year, prev)
    };
    Date::from_calendar_date(year, month, 1).unwrap_or(anchor)
}

/// Build the 6×7 grid of dates for the month containing `anchor`, including the
/// trailing days of the previous month and the leading days of the next month so
/// every cell is filled (the classic fixed-height month view).
fn month_grid(anchor: Date) -> Vec<Date> {
    let first = first_of_month(anchor);
    let lead = first.weekday().number_days_from_sunday();
    let start = first - time::Duration::days(i64::from(lead));
    (0..42)
        .map(|offset| start + time::Duration::days(offset))
        .collect()
}

/// Calendar — shadcn Base UI `calendar`. A pure-Leptos single-month day grid
/// (the upstream is a `react-day-picker` wrapper). Controlled via an external
/// `selected` signal or uncontrolled internally; navigate months with the
/// previous/next chevrons. Selecting a day sets `selected` and fires `on_change`.
#[component]
pub fn Calendar(
    /// The currently selected date, if any. Defaults to an uncontrolled signal.
    #[prop(optional)]
    selected: Option<RwSignal<Option<Date>>>,
    /// The month to display first; defaults to the selected date's month, else today.
    #[prop(optional)]
    default_month: Option<Date>,
    /// Fired with the date a user picks.
    #[prop(optional)]
    on_change: Option<Callback<Date>>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let selected = selected.unwrap_or_else(|| RwSignal::new(None));
    let initial = default_month
        .or_else(|| selected.get_untracked())
        .unwrap_or_else(today);
    let display = RwSignal::new(first_of_month(initial));

    let caption = move || {
        let m = display.get();
        format!("{} {}", month_name(m.month()), m.year())
    };

    view! {
        <div
            data-slot="calendar"
            class=move || {
                cn!(
                    "cn-calendar group/calendar w-fit bg-background [--cell-radius:var(--radius-md)] [--cell-size:--spacing(8)]",
                    class.get(),
                )
            }
        >
            <div class="flex flex-col gap-4">
                <CalendarNav display=display caption=caption />
                <CalendarGrid display=display selected=selected on_change=on_change />
            </div>
        </div>
    }
}

/// The caption row: previous-month chevron, the "Month Year" label, next-month chevron.
#[component]
fn CalendarNav(display: RwSignal<Date>, #[prop(into)] caption: Signal<String>) -> impl IntoView {
    view! {
        <div
            data-slot="calendar-nav"
            class="relative flex h-(--cell-size) w-full items-center justify-between gap-1"
        >
            <button
                type="button"
                data-slot="calendar-previous"
                aria-label="Go to the previous month"
                class="cn-button cn-button-variant-ghost size-(--cell-size) p-0 select-none"
                on:click=move |_| display.update(|d| *d = shift_month(*d, false))
            >
                <Icon icon=icondata::LuChevronLeft attr:class="size-4" />
            </button>
            <div
                data-slot="calendar-caption"
                aria-live="polite"
                class="cn-calendar-caption flex h-(--cell-size) items-center justify-center text-sm font-medium select-none"
            >
                {move || caption.get()}
            </div>
            <button
                type="button"
                data-slot="calendar-next"
                aria-label="Go to the next month"
                class="cn-button cn-button-variant-ghost size-(--cell-size) p-0 select-none"
                on:click=move |_| display.update(|d| *d = shift_month(*d, true))
            >
                <Icon icon=icondata::LuChevronRight attr:class="size-4" />
            </button>
        </div>
    }
}

/// The weekday header row plus the six weeks of day cells for the displayed month.
#[component]
fn CalendarGrid(
    display: RwSignal<Date>,
    selected: RwSignal<Option<Date>>,
    on_change: Option<Callback<Date>>,
) -> impl IntoView {
    let weeks = Memo::new(move |_| {
        let mut rows: Vec<Vec<Date>> = Vec::new();
        for chunk in month_grid(display.get()).chunks(7) {
            rows.push(chunk.to_vec());
        }
        rows
    });
    view! {
        <div data-slot="calendar-month-grid" role="grid" class="w-full">
            <div data-slot="calendar-weekdays" role="row" class="flex">
                {WEEKDAY_LABELS
                    .iter()
                    .map(|label| {
                        view! {
                            <div
                                role="columnheader"
                                data-slot="calendar-weekday"
                                class="flex-1 rounded-(--cell-radius) text-[0.8rem] font-normal text-muted-foreground select-none"
                            >
                                {*label}
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
            <For each=move || weeks.get() key=|week| week.first().copied() let:week>
                <div data-slot="calendar-week" role="row" class="mt-2 flex w-full">
                    {week
                        .into_iter()
                        .map(|date| {
                            view! {
                                <CalendarDay
                                    date=date
                                    display=display
                                    selected=selected
                                    on_change=on_change
                                />
                            }
                        })
                        .collect_view()}
                </div>
            </For>
        </div>
    }
}

/// A single day cell. Carries `data-selected`/`data-today`/`data-outside` so the
/// nova layer can style state; clicking selects the date and fires `on_change`.
#[component]
fn CalendarDay(
    date: Date,
    display: RwSignal<Date>,
    selected: RwSignal<Option<Date>>,
    on_change: Option<Callback<Date>>,
) -> impl IntoView {
    let is_selected = Memo::new(move |_| selected.get() == Some(date));
    let is_today = Memo::new(move |_| today() == date);
    let is_outside = Memo::new(move |_| date.month() != display.get().month());
    let label = date.to_string();
    let pick = move |_| {
        selected.set(Some(date));
        if let Some(cb) = on_change {
            cb.run(date);
        }
    };
    view! {
        <div
            data-slot="calendar-day"
            role="gridcell"
            class="group/day relative aspect-square h-full w-full rounded-(--cell-radius) p-0 text-center select-none"
        >
            <button
                type="button"
                data-slot="calendar-day-button"
                data-day=label
                data-selected=move || is_selected.get().then_some("true")
                data-today=move || is_today.get().then_some("true")
                data-outside=move || is_outside.get().then_some("true")
                aria-selected=move || is_selected.get().to_string()
                class=move || {
                    cn!(
                        "cn-calendar-day-button cn-button cn-button-variant-ghost relative flex aspect-square size-auto w-full min-w-(--cell-size) flex-col gap-1 leading-none font-normal data-[today=true]:bg-muted data-[today=true]:text-foreground data-[selected=true]:bg-primary data-[selected=true]:text-primary-foreground data-[outside=true]:text-muted-foreground data-[outside=true]:opacity-50",
                    )
                }
                on:click=pick
            >
                {date.day().to_string()}
            </button>
        </div>
    }
}

/// English month name for the caption label.
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

/// Number of days in `month` of `year` — thin re-export of `time::util` so callers
/// of this module need not depend on `time` directly.
#[must_use]
pub fn days_in_month(month: Month, year: i32) -> u8 {
    util::days_in_month(month, year)
}

/// Returns the first occurrence of `weekday` on or before `date` (Sunday-based
/// week start), useful for callers laying out their own grids.
#[must_use]
pub fn week_start(date: Date) -> Date {
    let lead = i64::from(date.weekday().number_days_from_sunday());
    date - time::Duration::days(lead)
}
