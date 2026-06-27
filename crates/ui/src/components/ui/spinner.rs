use leptos_icons::Icon;
use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Spinner(#[prop(into, optional)] class: String) -> impl IntoView {
    let merged_class = tw_merge!("size-4 animate-spin", class);

    view! { <Icon icon=icondata::LuLoader attr:class=merged_class attr:role="status" attr:aria-label="Loading" /> }
}

#[component]
pub fn SpinnerCircle(#[prop(into, optional)] class: String) -> impl IntoView {
    let merged_class = tw_merge!("size-4 animate-spin", class);

    view! { <Icon icon=icondata::LuLoaderCircle attr:class=merged_class attr:role="status" attr:aria-label="Loading" /> }
}