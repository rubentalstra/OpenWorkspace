use crate::cn;
use leptos::html;
use leptos::prelude::*;
use leptos_icons::Icon;

const CHECKBOX_BASE: &str = "group peer border-input dark:bg-input/30 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground dark:data-[state=checked]:bg-primary data-[state=checked]:border-primary focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive inline-flex size-4 shrink-0 items-center justify-center rounded-[4px] border shadow-xs transition-shadow outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50";

/// Styled checkbox rendered as a `role="checkbox"` button. Controlled via
/// `checked`; `on_checked_change` fires with the toggled value when clicked, and
/// the check mark shows automatically in the checked state. Native attributes,
/// events and bindings forward to the root.
#[component]
pub fn Checkbox(
    #[prop(into, optional)] checked: Signal<bool>,
    #[prop(optional)] on_checked_change: Option<Callback<bool>>,
    #[prop(into, optional)] aria_label: Option<String>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Button>,
) -> impl IntoView {
    let on_click = move |_| {
        if let Some(cb) = on_checked_change {
            cb.run(!checked.get_untracked());
        }
    };

    view! {
        <button
            node_ref=node_ref
            data-name="Checkbox"
            type="button"
            role="checkbox"
            data-state=move || if checked.get() { "checked" } else { "unchecked" }
            aria-checked=move || checked.get().to_string()
            aria-label=aria_label
            class=move || cn!(CHECKBOX_BASE, class.get())
            on:click=on_click
        >
            <span
                data-name="CheckboxIndicator"
                class="flex items-center justify-center text-current transition-none not-group-data-[state=checked]:hidden"
            >
                <Icon icon=icondata::LuCheck attr:class="size-3.5" />
            </span>
        </button>
    }
}
