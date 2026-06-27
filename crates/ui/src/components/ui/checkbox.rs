use crate::cn;
use leptos::html;
use leptos::prelude::*;
use leptos_icons::Icon;

const CHECKBOX_BASE: &str = "group peer border-input dark:bg-input/30 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground dark:data-[state=checked]:bg-primary data-[state=checked]:border-primary focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive inline-flex size-4 shrink-0 items-center justify-center rounded-[4px] border shadow-xs transition-shadow outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50";

/// Styled checkbox control rendered as a `role="checkbox"` button. Drive its
/// state from the call site via `attr:data-state="checked"` (and the matching
/// `attr:aria-checked`); the check mark shows automatically in the checked
/// state. All native attributes, events and bindings forward to the root.
#[component]
pub fn Checkbox(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Button>,
) -> impl IntoView {
    view! {
        <button
            node_ref=node_ref
            data-name="Checkbox"
            type="button"
            role="checkbox"
            class=move || cn!(CHECKBOX_BASE, class.get())
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
