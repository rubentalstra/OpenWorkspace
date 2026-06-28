use crate::cn;
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct RadioGroupCtx {
    value: RwSignal<String>,
    on_change: StoredValue<Option<Callback<String>>>,
}

/// RadioGroup — shadcn Base UI `radio-group`. Controlled via an external `value`
/// signal or uncontrolled via `default_value`; `on_change` fires on selection.
#[component]
pub fn RadioGroup(
    #[prop(optional)] value: Option<RwSignal<String>>,
    #[prop(into, optional)] default_value: String,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let value = value.unwrap_or_else(|| RwSignal::new(default_value));
    provide_context(RadioGroupCtx {
        value,
        on_change: StoredValue::new(on_change),
    });
    view! {
        <div
            role="radiogroup"
            data-slot="radio-group"
            class=move || cn!("cn-radio-group w-full", class.get())
        >
            {children()}
        </div>
    }
}

/// A single radio option, identified by its `value`.
#[component]
pub fn RadioGroupItem(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let ctx = expect_context::<RadioGroupCtx>();
    let for_memo = value.clone();
    let checked = Memo::new(move |_| ctx.value.get() == for_memo);
    view! {
        <button
            type="button"
            role="radio"
            data-slot="radio-group-item"
            aria-checked=move || checked.get().to_string()
            data-checked=move || checked.get().to_string()
            class=move || {
                cn!(
                    "cn-radio-group-item group/radio-group-item peer relative aspect-square shrink-0 border outline-none after:absolute after:-inset-x-3 after:-inset-y-2 disabled:cursor-not-allowed disabled:opacity-50",
                    class.get(),
                )
            }
            on:click=move |_| {
                ctx.value.set(value.clone());
                if let Some(cb) = ctx.on_change.get_value() {
                    cb.run(value.clone());
                }
            }
        >
            <span
                data-slot="radio-group-indicator"
                class="cn-radio-group-indicator"
                class:hidden=move || !checked.get()
            >
                <span class="cn-radio-group-indicator-icon" />
            </span>
        </button>
    }
}
