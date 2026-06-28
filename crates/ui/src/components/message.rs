use crate::{cn, slot};
use leptos::prelude::*;

slot! { MessageGroup, div, "message-group", "cn-message-group flex min-w-0 flex-col" }

/// Sender side of a [`Message`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum MessageAlign {
    /// Incoming (start).
    #[default]
    Start,
    /// Outgoing (end); reverses the row direction.
    End,
}

impl MessageAlign {
    fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// Message — shadcn Base UI `message`. A chat row (avatar + content), reversed when
/// outgoing.
#[component]
pub fn Message(
    #[prop(default = MessageAlign::Start)] align: MessageAlign,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="message"
            data-align=align.as_str()
            class=move || {
                cn!(
                    "cn-message group/message relative flex w-full min-w-0 data-[align=end]:flex-row-reverse",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

slot! {
    MessageAvatar, div, "message-avatar",
    "cn-message-avatar flex w-fit shrink-0 items-center justify-center self-end overflow-hidden rounded-full bg-muted"
}
slot! {
    MessageContent, div, "message-content",
    "cn-message-content flex w-full min-w-0 flex-col wrap-break-word"
}
slot! {
    MessageHeader, div, "message-header",
    "cn-message-header flex max-w-full min-w-0 items-center"
}
slot! {
    MessageFooter, div, "message-footer",
    "cn-message-footer flex max-w-full min-w-0 items-center group-data-[align=end]/message:justify-end"
}
