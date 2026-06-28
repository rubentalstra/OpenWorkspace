use crate::{clx, cn, use_random_id_for, void};
use leptos::context::Provider;
use leptos::prelude::*;

const BUTTON_GROUP_BASE: &str = "flex flex-wrap justify-center mt-2";

clx! {RadioButtonText, span, "block cursor-pointer bg-transparent text-primary px-3 py-1.5 relative ml-px shadow-[0_0_0_1px_#b5bfd9] tracking-wider text-center transition-colors duration-500"}

void! {RadioButtonInput, input, "radio__button", "focus:outline-0 focus:border-input/60"}
clx! {RadioButtonFieldset, fieldset, BUTTON_GROUP_BASE}
clx! {RadioButtonBar, div, BUTTON_GROUP_BASE, "[&>label:first-child>span]:rounded-l-md [&>label:last-child>span]:rounded-r-md"}

/// Selected value and shared input `name` handed from [`RadioButtonGroup`] to
/// its [`RadioButton`] children through context.
#[derive(Clone, Copy)]
struct RadioButtonGroupContext {
    value: RwSignal<String>,
    name: StoredValue<String>,
}

/// Segmented control that groups [`RadioButton`] children into a single
/// horizontal bar with rounded outer edges. Owns the selected `value`; clicking
/// an option writes its value into the shared signal so the choice is readable
/// and controllable from the call site.
#[component]
pub fn RadioButtonGroup(
    /// The selected option's value; drive it to read or control the choice.
    value: RwSignal<String>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = RadioButtonGroupContext {
        value,
        name: StoredValue::new(use_random_id_for("radio")),
    };

    view! {
        <Provider value=ctx>
            <RadioButtonFieldset class=class>
                <RadioButtonBar attr:role="radiogroup">{children()}</RadioButtonBar>
            </RadioButtonFieldset>
        </Provider>
    }
}

/// A single option within a [`RadioButtonGroup`]. It is selected when its
/// `value` matches the group's; clicking it sets the group value. The options of
/// one group share a native `name`, so arrow-key navigation works.
#[component]
pub fn RadioButton(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = use_context::<RadioButtonGroupContext>();
    let checked_value = value.clone();
    let checked = move || ctx.is_some_and(|c| c.value.get() == checked_value);
    let select_value = value;
    let select = move |_| {
        if let Some(c) = ctx {
            c.value.set(select_value.clone());
        }
    };
    let name = move || ctx.map(|c| c.name.get_value());

    view! {
        <label data-name="RadioButton" class=move || cn!(class.get())>
            <RadioButtonInput
                attr:r#type="radio"
                attr:name=name
                prop:checked=checked
                on:change=select
            />
            {children()}
        </label>
    }
}
