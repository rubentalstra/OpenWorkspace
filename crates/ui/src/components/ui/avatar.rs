use crate::{clx, cn, void};
use leptos::prelude::*;

clx! {AvatarFallback, div, "absolute inset-0 flex size-full items-center justify-center rounded-full bg-muted text-sm text-muted-foreground group-data-[size=sm]/avatar:text-xs"}
clx! {AvatarGroup, div, "group/avatar-group flex -space-x-2 *:data-[name=Avatar]:ring-2 *:data-[name=Avatar]:ring-background"}
clx! {AvatarGroupCount, div, "relative flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-sm text-muted-foreground ring-2 ring-background group-has-data-[size=lg]/avatar-group:size-10 group-has-data-[size=sm]/avatar-group:size-6 [&>svg]:size-4 group-has-data-[size=lg]/avatar-group:[&>svg]:size-5 group-has-data-[size=sm]/avatar-group:[&>svg]:size-3"}
clx! {AvatarBadge, span, "absolute right-0 bottom-0 z-10 inline-flex items-center justify-center rounded-full bg-primary text-primary-foreground ring-2 ring-background select-none group-data-[size=sm]/avatar:size-2 group-data-[size=sm]/avatar:[&>svg]:hidden group-data-[size=default]/avatar:size-2.5 group-data-[size=default]/avatar:[&>svg]:size-2 group-data-[size=lg]/avatar:size-3 group-data-[size=lg]/avatar:[&>svg]:size-2"}
void! {AvatarImage, img, "absolute inset-0 z-10 aspect-square size-full rounded-full object-cover"}

/// Avatar container. `size` sets the diameter and is exposed to descendants via
/// `data-size` so fallback, badge and count styling can react to it.
#[component]
pub fn Avatar(
    #[prop(into, optional)] size: Signal<AvatarSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        let size_class = match size.get() {
            AvatarSize::Sm => "size-6",
            AvatarSize::Default => "size-8",
            AvatarSize::Lg => "size-10",
        };
        cn!(
            "group/avatar relative flex shrink-0 overflow-hidden rounded-full select-none after:absolute after:inset-0 after:rounded-full after:border after:border-border after:mix-blend-darken dark:after:mix-blend-lighten",
            size_class,
            class.get()
        )
    };
    let data_size = move || match size.get() {
        AvatarSize::Sm => "sm",
        AvatarSize::Default => "default",
        AvatarSize::Lg => "lg",
    };

    view! {
        <div data-name="Avatar" data-size=data_size class=merged>
            {children()}
        </div>
    }
}

/// Diameter of an [`Avatar`] and its descendants.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarSize {
    Sm,
    #[default]
    Default,
    Lg,
}
