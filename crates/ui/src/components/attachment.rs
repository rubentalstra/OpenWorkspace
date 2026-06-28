use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::{cn, slot};
use leptos::prelude::*;

const ATTACHMENT_BASE: &str = "cn-attachment group/attachment relative flex max-w-full min-w-0 shrink-0 flex-wrap border bg-card text-card-foreground transition-colors has-[>a,>button]:hover:bg-muted/50 data-[state=error]:border-destructive/30 data-[state=idle]:border-dashed";

/// Upload lifecycle state of an [`Attachment`], surfaced as `data-state`.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AttachmentState {
    /// Empty drop target.
    Idle,
    /// Upload in progress.
    Uploading,
    /// Server-side processing.
    Processing,
    /// Failed.
    Error,
    /// Completed (the default).
    #[default]
    Done,
}

impl AttachmentState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Uploading => "uploading",
            Self::Processing => "processing",
            Self::Error => "error",
            Self::Done => "done",
        }
    }
}

/// Size of an [`Attachment`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AttachmentSize {
    /// The default size.
    #[default]
    Default,
    /// Compact.
    Sm,
    /// Extra compact.
    Xs,
}

impl AttachmentSize {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-attachment-size-default",
            Self::Sm => "cn-attachment-size-sm",
            Self::Xs => "cn-attachment-size-xs",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Xs => "xs",
        }
    }
}

/// Layout of an [`Attachment`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AttachmentOrientation {
    /// Media beside the content (the default).
    #[default]
    Horizontal,
    /// Media above the content.
    Vertical,
}

impl AttachmentOrientation {
    fn class(self) -> &'static str {
        match self {
            Self::Horizontal => "cn-attachment-orientation-horizontal items-center",
            Self::Vertical => "cn-attachment-orientation-vertical flex-col",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// Attachment — shadcn Base UI `attachment`. A file/media attachment card.
#[component]
pub fn Attachment(
    #[prop(default = AttachmentState::Done)] state: AttachmentState,
    #[prop(default = AttachmentSize::Default)] size: AttachmentSize,
    #[prop(default = AttachmentOrientation::Horizontal)] orientation: AttachmentOrientation,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="attachment"
            data-state=state.as_str()
            data-size=size.as_str()
            data-orientation=orientation.as_str()
            class=move || cn!(ATTACHMENT_BASE, size.class(), orientation.class(), class.get())
        >
            {children()}
        </div>
    }
}

const ATTACHMENT_MEDIA_BASE: &str = "cn-attachment-media relative flex aspect-square shrink-0 items-center justify-center overflow-hidden group-data-[state=error]/attachment:bg-destructive/10 group-data-[state=error]/attachment:text-destructive [&_svg]:pointer-events-none";

/// How [`AttachmentMedia`] renders its content.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AttachmentMediaVariant {
    /// A centered icon (the default).
    #[default]
    Icon,
    /// A cover image.
    Image,
}

impl AttachmentMediaVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Icon => "cn-attachment-media-variant-icon",
            Self::Image => {
                "cn-attachment-media-variant-image *:[img]:aspect-square *:[img]:w-full *:[img]:object-cover"
            }
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Icon => "icon",
            Self::Image => "image",
        }
    }
}

/// The leading media (icon or image) of an [`Attachment`].
#[component]
pub fn AttachmentMedia(
    #[prop(default = AttachmentMediaVariant::Icon)] variant: AttachmentMediaVariant,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="attachment-media"
            data-variant=variant.as_str()
            class=move || cn!(ATTACHMENT_MEDIA_BASE, variant.class(), class.get())
        >
            {children()}
        </div>
    }
}

slot! {
    AttachmentContent, div, "attachment-content",
    "cn-attachment-content max-w-full min-w-0 flex-1"
}
slot! {
    AttachmentTitle, span, "attachment-title",
    "cn-attachment-title block max-w-full min-w-0 truncate group-data-[state=processing]/attachment:shimmer group-data-[state=uploading]/attachment:shimmer"
}
slot! {
    AttachmentDescription, span, "attachment-description",
    "cn-attachment-description block max-w-full min-w-0 truncate text-muted-foreground group-data-[state=error]/attachment:text-destructive/80"
}
slot! {
    AttachmentActions, div, "attachment-actions",
    "cn-attachment-actions flex shrink-0 items-center"
}
slot! {
    AttachmentGroup, div, "attachment-group",
    "cn-attachment-group flex min-w-0 scroll-fade-x snap-x snap-mandatory scrollbar-none overflow-x-auto overscroll-x-contain *:data-[slot=attachment]:flex-none *:data-[slot=attachment]:snap-start"
}

/// A small icon button in an [`AttachmentActions`] row (ghost, icon-xs by default).
#[component]
pub fn AttachmentAction(
    #[prop(into, optional, default = ButtonVariant::Ghost)] variant: ButtonVariant,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <Button
            variant=variant
            size=ButtonSize::IconXs
            class=Signal::derive(move || cn!("cn-attachment-action", class.get()))
            attr:data-slot="attachment-action"
        >
            {children()}
        </Button>
    }
}

/// A full-card overlay button (covers the attachment for click/keyboard activation).
#[component]
pub fn AttachmentTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            data-slot="attachment-trigger"
            class=move || {
                cn!("cn-attachment-trigger absolute inset-0 z-10 outline-none", class.get())
            }
        >
            {children()}
        </button>
    }
}
