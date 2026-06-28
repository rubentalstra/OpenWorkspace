use crate::cn;
use leptos::prelude::*;

/// Switch track size, surfaced as `data-size` for the nova layer.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SwitchSize {
    #[default]
    Default,
    Sm,
}

impl SwitchSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
        }
    }
}

/// Switch — shadcn Base UI `switch`. Controlled: read `checked`, react to
/// `on_change`. State is exposed as `data-checked`/`data-unchecked` (on both the
/// root and thumb) plus `aria-checked`, as the nova layer expects.
#[component]
pub fn Switch(
    #[prop(into, optional)] checked: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
    #[prop(into, optional)] size: Signal<SwitchSize>,
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
            role="switch"
            data-slot="switch"
            data-size=move || size.get().as_str()
            aria-checked=move || checked.get().to_string()
            data-checked=move || checked.get().to_string()
            data-unchecked=move || (!checked.get()).to_string()
            class=move || {
                cn!(
                    "cn-switch peer group/switch relative inline-flex items-center transition-all outline-none after:absolute after:-inset-x-3 after:-inset-y-2 data-disabled:cursor-not-allowed data-disabled:opacity-50",
                    class.get(),
                )
            }
            on:click=toggle
        >
            <span
                data-slot="switch-thumb"
                data-checked=move || checked.get().to_string()
                data-unchecked=move || (!checked.get()).to_string()
                class="cn-switch-thumb pointer-events-none block ring-0 transition-transform"
            />
        </button>
    }
}
