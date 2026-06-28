use crate::cn;
use crate::components::toggle::{ToggleSize, ToggleVariant};
use leptos::prelude::*;

const TG_BASE: &str = "cn-toggle group/toggle inline-flex items-center justify-center whitespace-nowrap outline-none hover:bg-muted focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 cn-toggle-group-item shrink-0 focus:z-10 focus-visible:z-10";

#[derive(Clone, Copy)]
struct ToggleGroupCtx {
    variant: ToggleVariant,
    size: ToggleSize,
    value: RwSignal<Vec<String>>,
    on_change: StoredValue<Option<Callback<Vec<String>>>>,
}

/// ToggleGroup — shadcn Base UI `toggle-group`. A set of toggles sharing a
/// multi-select value (`Vec<String>` of pressed item values).
#[component]
pub fn ToggleGroup(
    #[prop(optional)] variant: ToggleVariant,
    #[prop(optional)] size: ToggleSize,
    #[prop(optional)] value: Option<RwSignal<Vec<String>>>,
    #[prop(optional)] on_change: Option<Callback<Vec<String>>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let value = value.unwrap_or_else(|| RwSignal::new(Vec::new()));
    provide_context(ToggleGroupCtx {
        variant,
        size,
        value,
        on_change: StoredValue::new(on_change),
    });
    view! {
        <div
            role="group"
            data-slot="toggle-group"
            data-orientation="horizontal"
            class=move || {
                cn!(
                    "cn-toggle-group group/toggle-group flex w-fit flex-row items-center gap-2 data-vertical:flex-col data-vertical:items-stretch",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// A toggle within a `ToggleGroup`, identified by `value`.
#[component]
pub fn ToggleGroupItem(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ToggleGroupCtx>();
    let for_memo = value.clone();
    let pressed = Memo::new(move |_| ctx.value.get().iter().any(|item| item == &for_memo));
    let variant_class = ctx.variant.class();
    let size_class = ctx.size.class();
    view! {
        <button
            type="button"
            data-slot="toggle-group-item"
            aria-pressed=move || pressed.get().to_string()
            data-state=move || if pressed.get() { "on" } else { "off" }
            class=move || cn!(TG_BASE, variant_class, size_class, class.get())
            on:click=move |_| {
                ctx.value
                    .update(|vals| {
                        if let Some(index) = vals.iter().position(|item| item == &value) {
                            vals.remove(index);
                        } else {
                            vals.push(value.clone());
                        }
                    });
                if let Some(cb) = ctx.on_change.get_value() {
                    cb.run(ctx.value.get_untracked());
                }
            }
        >
            {children()}
        </button>
    }
}
