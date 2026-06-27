use crate::{clx, cn};
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum CardSize {
    #[default]
    Default,
    Sm,
}

clx! {CardHeader, div, "@container/card-header flex flex-col items-start gap-1.5 px-6 [[data-size=sm]_&]:px-4 [.border-b]:pb-6 sm:grid sm:auto-rows-min sm:grid-rows-[auto_auto] has-data-[name=CardAction]:sm:grid-cols-[1fr_auto]"}
clx! {CardTitle, h2, "leading-none font-semibold"}
clx! {CardContent, div, "px-6 [[data-size=sm]_&]:px-4"}
clx! {CardDescription, p, "text-muted-foreground text-sm"}
clx! {CardFooter, footer, "flex items-center px-6 [[data-size=sm]_&]:px-4 [.border-t]:pt-6", "gap-2"}
clx! {CardAction, div, "self-start sm:col-start-2 sm:row-span-2 sm:row-start-1 sm:justify-self-end"}
clx! {CardList, ul, "flex flex-col gap-4"}
clx! {CardItem, li, "flex items-center [&_svg:not([class*='size-'])]:size-4 [&_svg]:shrink-0"}

/// Surface container. `size` controls vertical rhythm and is exposed to
/// descendants via `data-size` for spacing variants.
#[component]
pub fn Card(
    #[prop(into, optional)] size: Signal<CardSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        let size_class = match size.get() {
            CardSize::Default => "py-6 gap-4",
            CardSize::Sm => "py-4 gap-3",
        };
        cn!(
            "bg-card text-card-foreground flex flex-col rounded-xl border shadow-sm",
            size_class,
            class.get()
        )
    };
    let data_size = move || match size.get() {
        CardSize::Default => "default",
        CardSize::Sm => "sm",
    };

    view! {
        <div data-name="Card" data-size=data_size class=merged>
            {children()}
        </div>
    }
}
