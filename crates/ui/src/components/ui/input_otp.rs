use crate::{cn, use_input_otp, use_random_id};
use leptos::prelude::*;
use leptos_icons::Icon;

/// One-time-code field: renders the visible slots plus a hidden input that holds
/// the value. Mirroring digits into the slots, the caret and focus handling are
/// wired on the client. `value` seeds the initial code; `on_change` fires with
/// the full code on every edit so the entered value is observable from Rust.
#[component]
pub fn InputOTP(
    children: Children,
    max_length: u32,
    #[prop(optional)] disabled: bool,
    #[prop(into, optional)] value: String,
    /// Fires with the current code whenever the user edits it.
    #[prop(optional)]
    on_change: Option<Callback<String>>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    use_input_otp();
    let container_id = format!("otp_{}", use_random_id());

    let on_input = move |ev: leptos::ev::Event| {
        if let Some(cb) = on_change {
            cb.run(event_target_value(&ev));
        }
    };

    view! {
        <div
            data-slot="input-otp"
            data-otp-root=""
            id=container_id
            class=move || {
                cn!("relative flex items-center gap-2 has-[:disabled]:opacity-50", class.get())
            }
        >
            {children()}
            // A `display:none` input cannot be focused or typed into, which
            // would make the whole field inert. Instead overlay a transparent,
            // focusable input across the slots: clicks focus it, keystrokes land
            // in it, and the client controller mirrors its value into the slots
            // and drives the active-slot caret from its selection.
            <input
                data-otp-input=""
                type="text"
                inputmode="numeric"
                autocomplete="one-time-code"
                maxlength=max_length.to_string()
                disabled=disabled
                prop:value=value
                class="absolute inset-0 h-full w-full cursor-default opacity-0 outline-none disabled:cursor-not-allowed"
                on:input=on_input
            />
        </div>
    }
}

/// Groups a run of [`InputOTPSlot`]s with joined edges.
#[component]
pub fn InputOTPGroup(
    children: Children,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <div data-slot="input-otp-group" class=move || cn!("flex items-center", class.get())>
            {children()}
        </div>
    }
}

/// A single character slot at `index`; the active slot shows a blinking caret.
#[component]
pub fn InputOTPSlot(
    index: u32,
    #[prop(optional)] aria_invalid: bool,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <div
            data-slot="input-otp-slot"
            data-otp-slot=""
            data-otp-index=index.to_string()
            data-active="false"
            class=move || {
                cn!(
                    "relative flex h-9 w-9 cursor-text items-center justify-center border-y border-r border-input text-sm shadow-xs transition-all outline-none first:rounded-l-md first:border-l last:rounded-r-md data-[active=true]:z-10 data-[active=true]:border-ring data-[active=true]:ring-[3px] data-[active=true]:ring-ring/50 aria-invalid:border-destructive data-[active=true]:aria-invalid:ring-destructive/20 dark:bg-input/30",
                    class.get(),
                )
            }
            attr:aria-invalid=aria_invalid.then_some("true")
        >
            <span data-otp-char=""></span>
            <div
                data-otp-caret=""
                class="flex absolute inset-0 justify-center items-center pointer-events-none"
                style="display: none"
            >
                <div class="w-px h-4 duration-1000 animate-caret-blink bg-foreground"></div>
            </div>
        </div>
    }
}

/// Visual separator between OTP groups.
#[component]
pub fn InputOTPSeparator(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <div
            data-slot="input-otp-separator"
            role="separator"
            class=move || cn!("flex items-center justify-center text-muted-foreground", class.get())
        >
            <Icon icon=icondata::LuMinus attr:class="size-4" />
        </div>
    }
}
