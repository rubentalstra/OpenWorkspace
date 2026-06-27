use crate::cn;
use leptos::prelude::*;

const LABEL_BASE: &str = "flex items-center gap-2 text-sm leading-none font-medium select-none peer-disabled:cursor-not-allowed peer-disabled:opacity-50 group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50";

/// Form label. Associate it with a control through `r#for`, or wrap the control
/// as a child.
#[component]
pub fn Label(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] r#for: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <label data-name="Label" r#for=r#for class=move || cn!(LABEL_BASE, class.get())>
            {children()}
        </label>
    }
}
