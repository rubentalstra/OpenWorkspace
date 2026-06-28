use crate::cn;
use leptos::prelude::*;
use leptos_icons::Icon;

/// Spinner — shadcn Base UI `spinner`. A spinning Lucide loader (`loader-circle`,
/// shadcn's `Loader2Icon`).
#[component]
pub fn Spinner(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <Icon
            icon=icondata::LuLoaderCircle
            attr:data-slot="spinner"
            attr:role="status"
            attr:aria-label="Loading"
            attr:class=move || cn!("size-4 animate-spin", class.get())
        />
    }
}
