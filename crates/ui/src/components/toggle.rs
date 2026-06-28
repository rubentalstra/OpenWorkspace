use crate::cn;
use leptos::prelude::*;

/// Toggle visual style.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ToggleVariant {
    #[default]
    Default,
    Outline,
}

impl ToggleVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-toggle-variant-default",
            Self::Outline => "cn-toggle-variant-outline",
        }
    }
}

/// Toggle size.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ToggleSize {
    #[default]
    Default,
    Sm,
    Lg,
}

impl ToggleSize {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-toggle-size-default",
            Self::Sm => "cn-toggle-size-sm",
            Self::Lg => "cn-toggle-size-lg",
        }
    }
}

const TOGGLE_BASE: &str = "cn-toggle group/toggle inline-flex items-center justify-center whitespace-nowrap outline-none hover:bg-muted focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0";

/// Toggle — shadcn Base UI `toggle`. A two-state button; controlled via `pressed`
/// + `on_change`. Pressed state is exposed as `aria-pressed` + `data-state`.
#[component]
pub fn Toggle(
    #[prop(into, optional)] variant: Signal<ToggleVariant>,
    #[prop(into, optional)] size: Signal<ToggleSize>,
    #[prop(into, optional)] pressed: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let toggle = move |_| {
        if let Some(cb) = on_change {
            cb.run(!pressed.get_untracked());
        }
    };
    view! {
        <button
            type="button"
            data-slot="toggle"
            aria-pressed=move || pressed.get().to_string()
            data-state=move || if pressed.get() { "on" } else { "off" }
            class=move || cn!(TOGGLE_BASE, variant.get().class(), size.get().class(), class.get())
            on:click=toggle
        >
            {children.map(|children| children())}
        </button>
    }
}
