use crate::{Button, ButtonSize, ButtonVariant, cn, use_theme_mode};
use leptos::prelude::*;
use leptos_icons::Icon;

const TOGGLE_BASE: &str = "relative isolate";

const ICON_BASE: &str =
    "absolute size-4 transition-all duration-500 ease-out motion-reduce:transition-none";

/// Dark-mode switch wired to the shared [`crate::ThemeMode`]. Clicking flips the
/// active theme and persists it; `aria-pressed` reflects whether dark mode is on.
/// A sun and a moon glyph cross-fade with pure CSS so the control animates
/// without any JavaScript. Native attributes and events forward to the button.
#[component]
pub fn ThemeToggle(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let theme = use_theme_mode();

    let pressed = move || if theme.get() { "true" } else { "false" };
    let sun_class = move || {
        cn!(
            ICON_BASE,
            if theme.get() {
                "scale-50 rotate-90 opacity-0"
            } else {
                "scale-100 rotate-0 opacity-100"
            }
        )
    };
    let moon_class = move || {
        cn!(
            ICON_BASE,
            if theme.get() {
                "scale-100 rotate-0 opacity-100"
            } else {
                "scale-50 -rotate-90 opacity-0"
            }
        )
    };

    view! {
        <Button
            variant=ButtonVariant::Ghost
            size=ButtonSize::Icon
            class=move || cn!(TOGGLE_BASE, class.get())
            attr:r#type="button"
            attr:aria-label="Toggle dark mode"
            attr:aria-pressed=pressed
            attr:data-name="ThemeToggle"
            on:click=move |_| theme.toggle()
        >
            <Icon icon=icondata::LuSun attr:class=sun_class attr:aria-hidden="true" />
            <Icon icon=icondata::LuMoon attr:class=moon_class attr:aria-hidden="true" />
        </Button>
    }
}
