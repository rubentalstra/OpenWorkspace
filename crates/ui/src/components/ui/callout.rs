use crate::cn;
use leptos::prelude::*;

const CALLOUT_BASE: &str = "relative w-full rounded-xl border px-4 py-3 text-sm md:-mx-1 [&_code]:bg-black/5 [&_code]:rounded [&_code]:px-1 [&_code]:py-0.5 dark:[&_code]:bg-white/10";

/// Tone of a [`Callout`], setting its border and surface colors.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CalloutVariant {
    #[default]
    Default,
    Info,
    Warning,
}

/// Notice box for inline messages. `variant` sets the tone; an optional `title`
/// renders above the body. Native attributes forward to the root element.
#[component]
pub fn Callout(
    #[prop(into, optional)] variant: Signal<CalloutVariant>,
    #[prop(into, optional)] title: Signal<String>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        let tone = match variant.get() {
            CalloutVariant::Default => "border-border bg-surface text-surface-foreground",
            CalloutVariant::Info => {
                "border-info bg-info-light text-foreground dark:bg-info-dark/20 dark:border-info/50"
            }
            CalloutVariant::Warning => {
                "border-warning bg-warning-light text-foreground dark:bg-warning-dark/20 dark:border-warning/50"
            }
        };
        cn!(CALLOUT_BASE, tone, class.get())
    };

    view! {
        <div data-name="Callout" class=merged>
            {move || {
                let title = title.get();
                (!title.is_empty())
                    .then(|| view! { <p class="mb-1 font-medium leading-none">{title}</p> })
            }}
            <p class="text-sm leading-relaxed text-muted-foreground">{children()}</p>
        </div>
    }
}
