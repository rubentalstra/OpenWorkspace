use crate::cn;
use leptos::prelude::*;

const RADIO_GROUP_BASE: &str = "flex flex-col gap-3";
const RADIO_ITEM_BASE: &str = "aspect-square size-4 shrink-0 rounded-full border border-input shadow-xs transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:border-primary";

/// Selection state shared from a [`RadioGroup`] to its [`RadioGroupItem`]s.
#[derive(Clone, Copy)]
pub struct RadioGroupValue(pub RwSignal<String>);

/// Single-selection radio group. Binds the chosen option's value to `value` and
/// exposes it to descendant [`RadioGroupItem`]s through context.
#[component]
pub fn RadioGroup(
    value: RwSignal<String>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    provide_context(RadioGroupValue(value));

    view! {
        <div
            data-name="RadioGroup"
            role="radiogroup"
            class=move || cn!(RADIO_GROUP_BASE, class.get())
        >
            {children()}
        </div>
    }
}

/// One option within a [`RadioGroup`]; selecting it sets the group's value to
/// `value`. Forward `attr:id`, `attr:disabled` and other native attributes at
/// the call site.
#[component]
pub fn RadioGroupItem(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let RadioGroupValue(selected) = expect_context::<RadioGroupValue>();
    let option = StoredValue::new(value);
    let is_checked = Memo::new(move |_| option.with_value(|v| selected.get() == *v));

    view! {
        <button
            data-name="RadioGroupItem"
            type="button"
            role="radio"
            class=move || cn!(RADIO_ITEM_BASE, class.get())
            aria-checked=move || is_checked.get().to_string()
            data-state=move || if is_checked.get() { "checked" } else { "unchecked" }
            on:click=move |_| selected.set(option.get_value())
        >
            <span class="flex items-center justify-center">
                {move || {
                    is_checked
                        .get()
                        .then(|| view! { <span class="size-2.5 rounded-full bg-primary" /> })
                }}
            </span>
        </button>
    }
}
