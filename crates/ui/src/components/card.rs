use crate::{cn, slot};
use leptos::prelude::*;

/// Card density, surfaced as `data-size` for the nova layer to react to.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum CardSize {
    #[default]
    Default,
    Sm,
}

impl CardSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
        }
    }
}

/// Card — shadcn Base UI `card` container.
#[component]
pub fn Card(
    #[prop(into, optional)] size: Signal<CardSize>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            data-slot="card"
            data-size=move || size.get().as_str()
            class=move || cn!("cn-card group/card flex flex-col", class.get())
        >
            {children.map(|children| children())}
        </div>
    }
}

slot! {
    /// Card header grid; reacts to `CardAction`/`CardDescription` siblings.
    CardHeader, div, "card-header",
    "cn-card-header group/card-header @container/card-header grid auto-rows-min items-start has-data-[slot=card-action]:grid-cols-[1fr_auto] has-data-[slot=card-description]:grid-rows-[auto_auto]"
}
slot! { CardTitle, div, "card-title", "cn-card-title cn-font-heading" }
slot! { CardDescription, div, "card-description", "cn-card-description" }
slot! {
    CardAction, div, "card-action",
    "cn-card-action col-start-2 row-span-2 row-start-1 self-start justify-self-end"
}
slot! { CardContent, div, "card-content", "cn-card-content" }
slot! { CardFooter, div, "card-footer", "cn-card-footer flex items-center" }
