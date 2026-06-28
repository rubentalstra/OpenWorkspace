use crate::cn;
use crate::components::command::{
    Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList,
};
use crate::components::popover::{Popover, PopoverContent, PopoverTrigger};
use leptos::prelude::*;
use leptos_icons::Icon;

/// Combobox — shadcn Base UI `combobox`. A Popover whose content holds a Command:
/// the trigger opens an anchored popup containing a searchable list. Controlled via
/// an external `open` signal or uncontrolled via `default_open`. The selected value
/// is held in `value` (or `default_value`); items report their selection through the
/// command list. This is the thin composition wrapper over [`Popover`]/[`Command`].
#[component]
pub fn Combobox(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    view! {
        <Popover open=open class=class>
            <div data-slot="combobox">{children()}</div>
        </Popover>
    }
}

/// The control that toggles the combobox popup. Renders the current selection (its
/// `children`) plus the trailing chevron, exactly as the Base UI `combobox-trigger`.
#[component]
pub fn ComboboxTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <PopoverTrigger class=Signal::derive(move || {
            cn!("cn-combobox-trigger", class.get())
        })>
            {children()}
            <Icon
                icon=icondata::LuChevronDown
                attr:data-slot="combobox-trigger-icon"
                attr:class="cn-combobox-trigger-icon pointer-events-none"
            />
        </PopoverTrigger>
    }
}

/// The displayed selection text inside the trigger.
#[component]
pub fn ComboboxValue(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <span data-slot="combobox-value" class=move || cn!("", class.get())>
            {children()}
        </span>
    }
}

/// The combobox popup; mounted (and enter-animated) while open. Holds the [`Command`]
/// (input + list). Anchored under the trigger via the popover positioner.
#[component]
pub fn ComboboxContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let children = StoredValue::new(children);
    view! {
        <PopoverContent class=Signal::derive(move || {
            cn!(
                "cn-combobox-content cn-combobox-content-logical group/combobox-content relative w-full origin-(--transform-origin) p-0",
                class.get(),
            )
        })>
            <Command>{children.with_value(|children| children())}</Command>
        </PopoverContent>
    }
}

/// The text input used to filter the combobox list. Wraps [`CommandInput`].
#[component]
pub fn ComboboxInput(
    #[prop(into, optional)] placeholder: Signal<String>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <CommandInput
            placeholder=placeholder
            class=Signal::derive(move || cn!("cn-combobox-input w-auto", class.get()))
        />
    }
}

/// The scrollable region holding the combobox items. Wraps [`CommandList`].
#[component]
pub fn ComboboxList(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <CommandList class=Signal::derive(move || {
            cn!("cn-combobox-list overflow-y-auto overscroll-contain", class.get())
        })>{children()}</CommandList>
    }
}

/// A selectable combobox option. Wraps [`CommandItem`]; renders a trailing check
/// indicator while selected (driven by `selected`).
#[component]
pub fn ComboboxItem(
    #[prop(into)] value: String,
    #[prop(into, optional)] selected: Signal<bool>,
    #[prop(optional)] on_select: Option<Callback<String>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <CommandItem
            value=value
            selected=selected
            on_select=on_select.unwrap_or_else(|| Callback::new(|_: String| {}))
            class=Signal::derive(move || {
                cn!(
                    "cn-combobox-item relative flex w-full cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            })
        >
            {children()}
            <Show when=move || selected.get()>
                <span data-slot="combobox-item-indicator" class="cn-combobox-item-indicator">
                    <Icon
                        icon=icondata::LuCheck
                        attr:class="cn-combobox-item-indicator-icon pointer-events-none"
                    />
                </span>
            </Show>
        </CommandItem>
    }
}

/// A labelled group of combobox items. Wraps [`CommandGroup`].
#[component]
pub fn ComboboxGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <CommandGroup class=Signal::derive(move || {
            cn!("cn-combobox-group", class.get())
        })>{children()}</CommandGroup>
    }
}

/// The label heading for a [`ComboboxGroup`].
#[component]
pub fn ComboboxLabel(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div data-slot="combobox-label" class=move || cn!("cn-combobox-label", class.get())>
            {children()}
        </div>
    }
}

/// Shown when no combobox item matches the current filter. Wraps [`CommandEmpty`].
#[component]
pub fn ComboboxEmpty(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <CommandEmpty class=Signal::derive(move || {
            cn!("cn-combobox-empty", class.get())
        })>{children()}</CommandEmpty>
    }
}

/// A horizontal divider between combobox sections.
#[component]
pub fn ComboboxSeparator(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <div
            role="separator"
            data-slot="combobox-separator"
            class=move || cn!("cn-combobox-separator", class.get())
        ></div>
    }
}
