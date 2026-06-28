use crate::clx;
use crate::utils::country::Country;
use crate::utils::phone_number::{PhoneFormat, PhoneNumber};
use leptos::prelude::*;
use leptos_icons::Icon;

use super::command::{Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList};
use super::input::Input;
use super::popover::{Popover, PopoverAlign, PopoverContent, PopoverTrigger};

/// Countries surfaced at the top of the picker before the full list.
const COMMON_COUNTRIES: &[Country] = &[
    Country::UnitedStatesOfAmerica,
    Country::UnitedKingdom,
    Country::France,
    Country::Germany,
    Country::Canada,
    Country::Australia,
    Country::Spain,
    Country::Italy,
    Country::Japan,
    Country::China,
    Country::India,
    Country::Brazil,
    Country::Mexico,
];

clx! {
    /// Row wrapper joining the country selector and the phone field.
    InputPhoneWrapper, div, "flex w-full"
}

/// A selectable country row in the picker: flag, name, dial code and a check
/// mark for the current selection.
#[component]
fn CountryItem(country: Country, selected_country: RwSignal<Country>) -> impl IntoView {
    let search_value = format!(
        "{} {} {}",
        country.name(),
        country.alpha2(),
        country.dial_code_formatted()
    );

    view! {
        <CommandItem
            value=search_value
            on_select=Callback::new(move |()| selected_country.set(country))
        >
            <span class="text-base">{country.flag_emoji()}</span>
            <span class="flex-1 truncate">{country.name()}</span>
            <span class="w-12 text-right text-muted-foreground">
                {country.dial_code_formatted()}
            </span>
            <Show when=move || selected_country.get() == country>
                <Icon icon=icondata::LuCheck attr:class="ml-1 size-4" />
            </Show>
        </CommandItem>
    }
}

/// Phone-number field with a searchable country selector. The dial code follows
/// the chosen country; the number is stored as raw digits and shown formatted.
#[component]
pub fn InputPhone(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] value_signal: Option<RwSignal<PhoneNumber>>,
    #[prop(optional)] country_signal: Option<RwSignal<Country>>,
    #[prop(optional)] disabled: bool,
    #[prop(into, optional)] invalid: Signal<bool>,
    #[prop(optional)] on_blur: Option<Callback<()>>,
) -> impl IntoView {
    let value = value_signal.unwrap_or_else(|| RwSignal::new(PhoneNumber::default()));
    let selected_country =
        country_signal.unwrap_or_else(|| RwSignal::new(Country::UnitedStatesOfAmerica));

    view! {
        <InputPhoneWrapper class=class>
            <Popover align=PopoverAlign::Start>
                <PopoverTrigger
                    class="gap-1 px-3 w-auto rounded-r-none border-r-0"
                    attr:disabled=disabled
                    attr:aria-label="Select country"
                >
                    <span class="text-base">{move || selected_country.get().flag_emoji()}</span>
                    <span class="text-xs text-muted-foreground">
                        {move || selected_country.get().dial_code_formatted()}
                    </span>
                    <Icon icon=icondata::LuChevronsUpDown attr:class="ml-1 opacity-50 size-3" />
                </PopoverTrigger>

                <PopoverContent class="p-0 w-[280px]">
                    <Command>
                        <div class="flex gap-2 items-center px-2 border-b">
                            <Icon
                                icon=icondata::LuSearch
                                attr:class="size-4 text-muted-foreground shrink-0"
                            />
                            <CommandInput attr:placeholder="Search country..." />
                        </div>
                        <CommandList class="min-h-0 max-h-[280px]">
                            <CommandEmpty>"No country found."</CommandEmpty>
                            <CommandGroup>
                                {COMMON_COUNTRIES
                                    .iter()
                                    .map(|&country| {
                                        view! { <CountryItem country selected_country /> }
                                    })
                                    .collect_view()}
                            </CommandGroup>
                            <CommandGroup>
                                <div class="px-2 py-1.5 text-xs font-medium text-muted-foreground">
                                    "All countries"
                                </div>
                                {Country::all()
                                    .iter()
                                    .filter(|c| !COMMON_COUNTRIES.contains(c))
                                    .map(|&country| {
                                        view! { <CountryItem country selected_country /> }
                                    })
                                    .collect_view()}
                            </CommandGroup>
                        </CommandList>
                    </Command>
                </PopoverContent>
            </Popover>

            <div class="relative flex-1">
                <Input
                    class="pr-8 w-full rounded-l-none"
                    attr:r#type="tel"
                    attr:inputmode="numeric"
                    attr:placeholder=move || {
                        PhoneFormat::for_country(selected_country.get()).placeholder()
                    }
                    attr:disabled=disabled
                    attr:aria-label="Phone number"
                    attr:aria-invalid=move || invalid.get().to_string()
                    prop:value=move || value.get().format(selected_country.get())
                    on:input=move |ev| {
                        let format = PhoneFormat::for_country(selected_country.get());
                        value.set(PhoneNumber::new(&event_target_value(&ev), format.max_digits));
                    }
                    on:blur=move |_| {
                        if let Some(cb) = on_blur {
                            cb.run(());
                        }
                    }
                />
                <Show when=move || !value.get().is_empty() && !disabled>
                    <button
                        type="button"
                        tabindex="-1"
                        class="absolute right-2 top-1/2 p-0.5 rounded-sm transition-colors -translate-y-1/2 text-muted-foreground hover:text-foreground hover:bg-muted"
                        aria-label="Clear phone number"
                        on:click=move |_| value.set(PhoneNumber::default())
                    >
                        <Icon icon=icondata::LuX attr:class="size-4" />
                    </button>
                </Show>
            </div>
        </InputPhoneWrapper>
    }
}
