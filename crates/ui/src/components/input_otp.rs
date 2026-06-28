use crate::cn;
use leptos::prelude::*;
use leptos_icons::Icon;

#[derive(Clone, Copy)]
struct InputOtpCtx {
    value: RwSignal<String>,
    max_length: usize,
    focused: RwSignal<bool>,
}

/// One-time-code input — shadcn Base UI `input-otp`. Renders a single transparent
/// `<input>` overlaid on `max_length` slot boxes; each [`InputOtpSlot`] shows one
/// character of `value` and a blinking caret on the active index. Controlled: read
/// `value`, react to `on_change`. The input only keeps the first `max_length`
/// characters, so an [`InputOtpSlot`] index past the end stays empty.
#[component]
pub fn InputOtp(
    /// Number of code characters (and slot boxes) this field holds.
    #[prop(default = 6)]
    max_length: usize,
    /// Controlled value; defaults to a fresh empty signal when omitted.
    #[prop(optional)]
    value: Option<RwSignal<String>>,
    /// Fired with the trimmed value whenever the input changes.
    #[prop(optional)]
    on_change: Option<Callback<String>>,
    /// Extra classes for the overlaid `<input>`.
    #[prop(into, optional)]
    class: Signal<String>,
    /// Extra classes for the slot container.
    #[prop(into, optional)]
    container_class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let value = value.unwrap_or_else(|| RwSignal::new(String::new()));
    let focused = RwSignal::new(false);
    provide_context(InputOtpCtx {
        value,
        max_length,
        focused,
    });

    let on_input = move |ev: leptos::ev::Event| {
        let raw = event_target_value(&ev);
        let next: String = raw.chars().take(max_length).collect();
        value.set(next.clone());
        if let Some(cb) = on_change {
            cb.run(next);
        }
    };

    view! {
        <div
            data-slot="input-otp"
            class=move || {
                cn!(
                    "cn-input-otp flex items-center has-disabled:opacity-50 relative",
                    container_class.get(),
                )
            }
        >
            {children()}
            <input
                data-input-otp="true"
                inputmode="numeric"
                autocomplete="one-time-code"
                spellcheck="false"
                maxlength=max_length.to_string()
                prop:value=move || value.get()
                class=move || {
                    cn!(
                        "cn-input-otp-input absolute inset-0 h-full w-full bg-transparent text-transparent caret-transparent opacity-0 outline-none disabled:cursor-not-allowed",
                        class.get(),
                    )
                }
                on:input=on_input
                on:focus=move |_| focused.set(true)
                on:blur=move |_| focused.set(false)
            />
        </div>
    }
}

/// Groups a run of [`InputOtpSlot`]s (and any separators) inside an [`InputOtp`].
#[component]
pub fn InputOtpGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="input-otp-group"
            class=move || cn!("cn-input-otp-group flex items-center", class.get())
        >
            {children()}
        </div>
    }
}

/// A single character box. `index` selects which character of the parent
/// [`InputOtp`] value it renders; the active index draws a blinking caret.
#[component]
pub fn InputOtpSlot(
    /// Zero-based position within the parent [`InputOtp`] value.
    index: usize,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let ctx = expect_context::<InputOtpCtx>();
    let char_at = Memo::new(move |_| ctx.value.get().chars().nth(index));
    let filled = Memo::new(move |_| ctx.value.with(|v| v.chars().count()));
    let active = Memo::new(move |_| {
        let len = filled.get();
        let cursor = len.min(ctx.max_length.saturating_sub(1));
        ctx.focused.get() && index == cursor
    });
    let has_fake_caret = Memo::new(move |_| active.get() && char_at.get().is_none());

    view! {
        <div
            data-slot="input-otp-slot"
            data-active=move || active.get().to_string()
            class=move || {
                cn!(
                    "cn-input-otp-slot relative flex items-center justify-center data-[active=true]:z-10",
                    class.get(),
                )
            }
        >
            {move || char_at.get()}
            <Show when=move || has_fake_caret.get()>
                <div class="cn-input-otp-caret pointer-events-none absolute inset-0 flex items-center justify-center">
                    <div class="cn-input-otp-caret-line" />
                </div>
            </Show>
        </div>
    }
}

/// A visual divider (a Lucide minus) placed between [`InputOtpGroup`]s.
#[component]
pub fn InputOtpSeparator() -> impl IntoView {
    view! {
        <div
            data-slot="input-otp-separator"
            role="separator"
            class="cn-input-otp-separator flex items-center"
        >
            <Icon icon=icondata::LuMinus />
        </div>
    }
}
