use crate::{cn, slot};
use leptos::prelude::*;

/// Alert tone.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
}

impl AlertVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-alert-variant-default",
            Self::Destructive => "cn-alert-variant-destructive",
        }
    }
}

/// Alert — shadcn Base UI `alert`. A callout banner.
#[component]
pub fn Alert(
    #[prop(into, optional)] variant: Signal<AlertVariant>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="alert"
            role="alert"
            class=move || {
                cn!("cn-alert group/alert relative w-full", variant.get().class(), class.get())
            }
        >
            {children()}
        </div>
    }
}

slot! {
    AlertTitle, div, "alert-title",
    "cn-alert-title [&_a]:underline [&_a]:underline-offset-3 [&_a]:hover:text-foreground"
}
slot! {
    AlertDescription, div, "alert-description",
    "cn-alert-description [&_a]:underline [&_a]:underline-offset-3 [&_a]:hover:text-foreground"
}
slot! { AlertAction, div, "alert-action", "cn-alert-action" }
