use leptos::prelude::*;
use time::macros::date;
use time::{Date, Month};
use ui::{
    Calendar, DatePicker, DatePickerCell, DatePickerDualState, DatePickerHeader, DatePickerMonth,
    DatePickerRow, DatePickerState, DatePickerTable, DatePickerTitle, DatePickerWeekDay,
};

use super::{Demo, Page, Section};

/// Single and range date pickers built on the calendar primitives.
#[component]
pub fn DatesPage() -> impl IntoView {
    view! {
        <Page
            title="Dates"
            subtitle="Single and range date pickers built on the calendar primitives."
        >
            <SingleSection />
            <RangeSection />
            <PrimitivesSection />
            <StateSection />
        </Page>
    }
}

/// English month name for read-only labels, mirroring the kit's own mapping.
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

/// `2026-06-15` rendered as `15 June 2026`.
fn format_date(d: Date) -> String {
    format!("{} {} {}", d.day(), month_name(d.month()), d.year())
}

#[component]
fn SingleSection() -> impl IntoView {
    let day = RwSignal::new(date!(2026 - 06 - 15));
    let day_end = RwSignal::new(date!(2026 - 06 - 15));

    let picked = RwSignal::new(date!(2026 - 06 - 15));
    let picked_start = RwSignal::new(date!(2026 - 06 - 15));
    let picked_end = RwSignal::new(date!(2026 - 06 - 15));
    let on_select = Callback::new(move |d: Date| picked.set(d));

    view! {
        <Section
            title="Single date"
            description="One shared signal drives both ends, so each click replaces the selection."
        >
            <Demo col=true label="Pick a day">
                <Calendar start_date=day end_date=day_end />
                <span class="text-sm text-muted-foreground">
                    "Selected: " {move || format_date(day.get())}
                </span>
            </Demo>
            <Demo col=true label="With on_select callback">
                <Calendar start_date=picked_start end_date=picked_end on_select=on_select />
                <span class="text-sm text-muted-foreground">
                    "Last picked: " {move || format_date(picked.get())}
                </span>
            </Demo>
        </Section>
    }
}

#[component]
fn RangeSection() -> impl IntoView {
    let start = RwSignal::new(date!(2026 - 06 - 10));
    let end = RwSignal::new(date!(2026 - 06 - 20));

    let last = RwSignal::new(date!(2026 - 06 - 20));
    let on_select = Callback::new(move |d: Date| last.set(d));

    let nights = move || {
        let s = start.get();
        let e = end.get();
        (e - s).whole_days().max(0)
    };

    view! {
        <Section
            title="Range"
            description="With range=true the second click of each pair extends the selection; days between the ends are highlighted."
        >
            <Demo col=true label="Pick a range">
                <Calendar start_date=start end_date=end range=true on_select=on_select />
                <div class="flex flex-col gap-1 text-sm text-muted-foreground">
                    <span>
                        {move || format_date(start.get())} " \u{2192} "
                        {move || format_date(end.get())}
                    </span>
                    <span>{move || format!("{} night(s)", nights())}</span>
                    <span>"Last click: " {move || format_date(last.get())}</span>
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn PrimitivesSection() -> impl IntoView {
    let year = 2026;
    let month = Month::June;

    let start_date = RwSignal::new(date!(2026 - 06 - 12));
    let end_date = RwSignal::new(date!(2026 - 06 - 18));

    let headings = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
        .into_iter()
        .map(|label| view! { <DatePickerWeekDay>{label}</DatePickerWeekDay> })
        .collect_view();

    let rows = DatePickerState::get_calendar_days(year, month)
        .chunks(7)
        .map(|week| {
            let cells = week
                .iter()
                .copied()
                .map(|cell| {
                    view! {
                        <DatePickerCell
                            day=cell.day
                            year=year
                            month=month
                            disabled=cell.disabled
                            start_date=start_date
                            end_date=end_date
                            on_click=|_| {}
                        />
                    }
                })
                .collect_view();
            view! { <DatePickerRow>{cells}</DatePickerRow> }
        })
        .collect_view();

    view! {
        <Section
            title="Calendar primitives"
            description="The clx-styled building blocks behind Calendar, here wired by hand into a read-only month grid with a fixed highlighted range."
        >
            <Demo label="Hand-assembled grid">
                <DatePicker>
                    <DatePickerHeader>
                        <span class="text-sm font-medium" />
                        <DatePickerTitle>
                            {format!("{} {}", month_name(month), year)}
                        </DatePickerTitle>
                        <span class="text-sm font-medium" />
                    </DatePickerHeader>
                    <DatePickerMonth>
                        <DatePickerTable>
                            <thead>
                                <tr class="flex">{headings}</tr>
                            </thead>
                            <tbody>{rows}</tbody>
                        </DatePickerTable>
                    </DatePickerMonth>
                </DatePicker>
            </Demo>
        </Section>
    }
}

#[component]
fn StateSection() -> impl IntoView {
    let single = DatePickerState::new(date!(2026 - 06 - 10), date!(2026 - 06 - 20));
    let dual = DatePickerDualState::new(date!(2026 - 07 - 03), date!(2026 - 07 - 09));

    let (primary_month, primary_year) = DatePickerDualState::get_display_month(dual.start_date, 0);
    let (next_month, next_year) = DatePickerDualState::get_display_month(dual.start_date, 1);

    let probe = date!(2026 - 07 - 03);
    let is_edge = dual.is_start_or_end_date(probe);
    let grid_len = DatePickerDualState::calculate_calendar_data(primary_year, primary_month).len();

    view! {
        <Section
            title="State helpers"
            description="DatePickerState and DatePickerDualState are plain value types; these are their derived values rendered as text."
        >
            <Demo col=true label="DatePickerState">
                <span class="text-sm text-muted-foreground">
                    "start: " {format_date(single.start_date)} " \u{2022} end: "
                    {format_date(single.end_date)}
                </span>
            </Demo>
            <Demo col=true label="DatePickerDualState">
                <div class="flex flex-col gap-1 text-sm text-muted-foreground">
                    <span>
                        "range: " {format_date(dual.start_date)} " \u{2192} "
                        {format_date(dual.end_date)}
                    </span>
                    <span>
                        "primary pane: " {month_name(primary_month)} " " {primary_year.to_string()}
                    </span>
                    <span>"next pane: " {month_name(next_month)} " " {next_year.to_string()}</span>
                    <span>
                        {format!("{} is an edge of the range: {}", format_date(probe), is_edge)}
                    </span>
                    <span>{format!("primary grid cells: {grid_len}")}</span>
                </div>
            </Demo>
        </Section>
    }
}
