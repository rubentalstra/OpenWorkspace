use crate::{clx, cn, void};
use leptos::prelude::*;

const BUTTON_GROUP_BASE: &str = "flex flex-wrap justify-center mt-2";

clx! {RadioButtonText, span, "block cursor-pointer bg-transparent text-primary px-3 py-1.5 relative ml-px shadow-[0_0_0_1px_#b5bfd9] tracking-wider text-center transition-colors duration-500"}

void! {RadioButtonInput, input, "radio__button", "focus:outline-0 focus:border-input/60"}
clx! {RadioButtonFieldset, fieldset, BUTTON_GROUP_BASE}
clx! {RadioButtonBar, div, BUTTON_GROUP_BASE, "[&>label:first-child>span]:rounded-l-md [&>label:last-child>span]:rounded-r-md"}

/// Segmented control that groups [`RadioButton`] children into a single
/// horizontal bar with rounded outer edges.
#[component]
pub fn RadioButtonGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <RadioButtonFieldset class=class>
            <RadioButtonBar attr:role="radiogroup">{children()}</RadioButtonBar>
        </RadioButtonFieldset>
    }
}

/// A single option within a [`RadioButtonGroup`]. Forward the radio's
/// `attr:name`, `attr:value`, `attr:checked` and `bind:` at the call site.
#[component]
pub fn RadioButton(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <label data-name="RadioButton" class=move || cn!(class.get())>
            <RadioButtonInput attr:r#type="radio" />
            {children()}
        </label>
    }
}
