use crate::{clx, cn};
use leptos::prelude::*;
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
