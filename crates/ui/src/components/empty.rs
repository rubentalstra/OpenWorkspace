use crate::{cn, slot};
use leptos::prelude::*;

slot! {
    /// Empty — shadcn Base UI `empty`. An empty-state container.
    Empty, div, "empty",
    "cn-empty flex w-full min-w-0 flex-1 flex-col items-center justify-center text-center text-balance"
}
slot! {
    EmptyHeader, div, "empty-header", "cn-empty-header flex max-w-sm flex-col items-center"
}

/// Empty-state media kind.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum EmptyMediaVariant {
    #[default]
    Default,
    Icon,
}

impl EmptyMediaVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Icon => "icon",
        }
    }
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-empty-media-default",
            Self::Icon => "cn-empty-media-icon",
        }
    }
}

/// The empty-state illustration/icon slot.
#[component]
pub fn EmptyMedia(
    #[prop(into, optional)] variant: Signal<EmptyMediaVariant>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="empty-icon"
            data-variant=move || variant.get().as_str()
            class=move || {
                cn!(
                    "cn-empty-media flex shrink-0 items-center justify-center [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    variant.get().class(),
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

slot! { EmptyTitle, div, "empty-title", "cn-empty-title cn-font-heading" }
slot! {
    EmptyDescription, div, "empty-description",
    "cn-empty-description text-muted-foreground [&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-primary"
}
slot! {
    EmptyContent, div, "empty-content",
    "cn-empty-content flex w-full max-w-sm min-w-0 flex-col items-center text-balance"
}
