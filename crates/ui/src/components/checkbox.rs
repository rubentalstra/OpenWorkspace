use crate::cn;
use leptos::prelude::*;
use leptos_icons::Icon;

/// Checkbox — shadcn Base UI `checkbox`. Controlled: read `checked`, react to
/// `on_change`. Shows a Lucide check while checked; state is exposed as
/// `data-checked` + `aria-checked` for the nova layer.
#[component]
pub fn Checkbox(
    #[prop(into, optional)] checked: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let toggle = move |_| {
        if let Some(cb) = on_change {
            cb.run(!checked.get_untracked());
        }
    };
    view! {
        <button
            type="button"
            role="checkbox"
            data-slot="checkbox"
            aria-checked=move || checked.get().to_string()
            data-checked=move || checked.get().to_string()
            class=move || {
                cn!(
                    "cn-checkbox peer relative shrink-0 outline-none after:absolute after:-inset-x-3 after:-inset-y-2 disabled:cursor-not-allowed disabled:opacity-50",
                    class.get(),
                )
            }
            on:click=toggle
        >
            <Show when=move || checked.get()>
                <span
                    data-slot="checkbox-indicator"
                    class="cn-checkbox-indicator grid place-content-center text-current transition-none"
                >
                    <Icon icon=icondata::LuCheck />
                </span>
            </Show>
        </button>
    }
}
