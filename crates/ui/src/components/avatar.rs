use crate::{cn, slot};
use leptos::prelude::*;

/// Avatar size, surfaced as `data-size` for the nova layer.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AvatarSize {
    #[default]
    Default,
    Sm,
    Lg,
}

impl AvatarSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Lg => "lg",
        }
    }
}

#[derive(Clone, Copy)]
struct AvatarCtx {
    loaded: RwSignal<bool>,
}

/// Avatar — shadcn Base UI `avatar`. Shows `AvatarFallback` until the
/// `AvatarImage` finishes loading (mirroring the Base UI image-load state).
#[component]
pub fn Avatar(
    #[prop(into, optional)] size: Signal<AvatarSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = AvatarCtx {
        loaded: RwSignal::new(false),
    };
    provide_context(ctx);
    view! {
        <span
            data-slot="avatar"
            data-size=move || size.get().as_str()
            class=move || {
                cn!(
                    "cn-avatar group/avatar relative flex shrink-0 select-none after:absolute after:inset-0 after:border after:border-border after:mix-blend-darken dark:after:mix-blend-lighten",
                    class.get(),
                )
            }
        >
            {children()}
        </span>
    }
}

/// The avatar image; hidden until it loads (then the fallback hides).
#[component]
pub fn AvatarImage(
    #[prop(into)] src: String,
    #[prop(into, optional)] alt: String,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let ctx = expect_context::<AvatarCtx>();
    view! {
        <img
            data-slot="avatar-image"
            src=src
            alt=alt
            on:load=move |_| ctx.loaded.set(true)
            on:error=move |_| ctx.loaded.set(false)
            class:hidden=move || !ctx.loaded.get()
            class=move || cn!("cn-avatar-image aspect-square size-full object-cover", class.get())
        />
    }
}

/// Shown while the image is absent or still loading.
#[component]
pub fn AvatarFallback(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AvatarCtx>();
    view! {
        <span
            data-slot="avatar-fallback"
            class:hidden=move || ctx.loaded.get()
            class=move || {
                cn!(
                    "cn-avatar-fallback flex size-full items-center justify-center text-sm group-data-[size=sm]/avatar:text-xs",
                    class.get(),
                )
            }
        >
            {children()}
        </span>
    }
}

slot! {
    /// Overlaps a cluster of avatars (e.g. attendees).
    AvatarGroup, div, "avatar-group",
    "cn-avatar-group group/avatar-group flex -space-x-2 *:data-[slot=avatar]:ring-2 *:data-[slot=avatar]:ring-background"
}
slot! {
    /// A small status dot/badge anchored to an avatar's corner.
    AvatarBadge, span, "avatar-badge",
    "cn-avatar-badge absolute right-0 bottom-0 z-10 inline-flex items-center justify-center rounded-full bg-blend-color ring-2 select-none group-data-[size=sm]/avatar:size-2 group-data-[size=sm]/avatar:[&>svg]:hidden group-data-[size=default]/avatar:size-2.5 group-data-[size=default]/avatar:[&>svg]:size-2 group-data-[size=lg]/avatar:size-3 group-data-[size=lg]/avatar:[&>svg]:size-2"
}
slot! {
    /// The "+N" overflow tile in an `AvatarGroup`.
    AvatarGroupCount, div, "avatar-group-count",
    "cn-avatar-group-count relative flex shrink-0 items-center justify-center ring-2 ring-background"
}
