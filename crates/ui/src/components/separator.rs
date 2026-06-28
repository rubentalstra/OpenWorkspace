use crate::cn;
use leptos::prelude::*;

/// Separator orientation, emitted as `data-orientation` for the
/// `data-horizontal:`/`data-vertical:` nova variants.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl SeparatorOrientation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// Separator — shadcn Base UI `separator`. A decorative rule whose thickness
/// follows `orientation`.
#[component]
pub fn Separator(
    #[prop(into, optional)] orientation: Signal<SeparatorOrientation>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <div
            data-slot="separator"
            role="separator"
            aria-orientation=move || orientation.get().as_str()
            data-orientation=move || orientation.get().as_str()
            class=move || {
                cn!(
                    "shrink-0 bg-border data-horizontal:h-px data-horizontal:w-full data-vertical:w-px data-vertical:self-stretch",
                    class.get(),
                )
            }
        />
    }
}
