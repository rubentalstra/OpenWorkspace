use crate::cn;
use crate::components::calendar::Calendar;
use crate::components::popover::{Popover, PopoverContent, PopoverTrigger};
use leptos::prelude::*;
use leptos_icons::Icon;
use time::Date;

/// `DatePicker` — shadcn Base UI `date-picker` (the "Date Picker Simple"
/// composition). A [`Popover`] whose trigger is an outline button showing the
/// selected date (or a placeholder), and whose content hosts a single-select
/// [`Calendar`]. Picking a day writes `selected`, fires `on_change`, and closes.
#[component]
pub fn DatePicker(
    /// Externally-owned selected date; `None` while unset.
    #[prop(optional)]
    selected: Option<RwSignal<Option<Date>>>,
    /// Externally-owned open state of the popover.
    #[prop(optional)]
    open: Option<RwSignal<bool>>,
    /// Initial open state when uncontrolled.
    #[prop(default = false)]
    default_open: bool,
    /// Fired with the newly-picked date.
    #[prop(optional)]
    on_change: Option<Callback<Date>>,
    /// Text shown on the trigger while no date is selected.
    #[prop(into, optional, default = String::from("Pick a date"))]
    placeholder: String,
    /// Extra classes merged onto the trigger button.
    #[prop(into, optional)]
    class: Signal<String>,
) -> impl IntoView {
    let selected = selected.unwrap_or_else(|| RwSignal::new(None));
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));

    let on_pick = Callback::new(move |picked: Date| {
        selected.set(Some(picked));
        if let Some(cb) = on_change {
            cb.run(picked);
        }
        open.set(false);
    });

    let label = move || {
        selected.get().map_or_else(
            || placeholder.clone(),
            |d| format!("{}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day()),
        )
    };

    view! {
        <Popover open=open>
            <PopoverTrigger class=Signal::derive(move || {
                cn!(
                    "cn-button cn-button-variant-outline cn-button-size-default group/button inline-flex w-full shrink-0 items-center justify-start gap-2 px-2.5 font-normal whitespace-nowrap transition-all outline-none select-none [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            })>
                <Icon icon=icondata::LuCalendar attr:data-icon="inline-start" attr:class="size-4" />
                <span data-slot="date-picker-value">{label}</span>
            </PopoverTrigger>
            <PopoverContent class="w-auto p-0">
                <Calendar selected=selected on_change=on_pick />
            </PopoverContent>
        </Popover>
    }
}
