use crate::cn;
use leptos::prelude::*;
use leptos_icons::Icon;

const SPINNER_BASE: &str = "size-4 animate-spin";

/// Spinning loader icon for indeterminate progress.
#[component]
pub fn Spinner(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <Icon
            icon=icondata::LuLoader
            attr:data-name="Spinner"
            attr:role="status"
            attr:aria-label="Loading"
            attr:class=move || cn!(SPINNER_BASE, class.get())
        />
    }
}

/// Spinning circular loader icon for indeterminate progress.
#[component]
pub fn SpinnerCircle(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <Icon
            icon=icondata::LuLoaderCircle
            attr:data-name="SpinnerCircle"
            attr:role="status"
            attr:aria-label="Loading"
            attr:class=move || cn!(SPINNER_BASE, class.get())
        />
    }
}
