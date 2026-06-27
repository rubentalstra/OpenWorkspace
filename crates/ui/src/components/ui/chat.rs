use crate::cn;
use leptos::html;
use leptos::prelude::*;

crate::clx! {
    /// Outer chat surface: a bordered, full-width vertical stack that frames the
    /// header, body and footer.
    ChatCard, div, "flex flex-col w-full rounded-lg border"
}
crate::clx! {
    /// Sticky title row above the message list.
    ChatHeader, header, "flex items-center border-b"
}
crate::clx! {
    /// Vertically-spaced list of message rows.
    ChatMessageList, div, "space-y-4"
}
crate::clx! {
    /// Inbound message row, left-aligned and width-capped.
    ChatMessageReceived, div, "flex max-w-[85%]"
}
crate::clx! {
    /// Outbound message row, right-aligned via `ml-auto` and width-capped.
    ChatMessageSent, div, "flex ml-auto max-w-[85%]"
}
crate::clx! {
    /// Round avatar slot beside a message.
    ChatMessageAvatar, span, "flex shrink-0 overflow-hidden rounded-full"
}
crate::clx! {
    /// Speech bubble wrapping the message text.
    ChatMessageBubble, div, "py-2 px-3 text-sm rounded-lg"
}
crate::clx! {
    /// Message body text inside a bubble.
    ChatMessageContent, p, "leading-normal wrap-break-word"
}
crate::clx! {
    /// Right-aligned timestamp under a message.
    ChatMessageTime, p, "mt-1 text-xs text-right"
}
crate::clx! {
    /// Composer row beneath the message list.
    ChatFooter, footer, "flex items-center border-t"
}

/// Scrollable chat body. On mount it scrolls its own element to the bottom so
/// the latest message is visible; re-trigger by re-mounting the body (e.g. a
/// keyed `<For>`) when the conversation changes.
#[component]
pub fn ChatBody(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    Effect::new(move |_| {
        if let Some(el) = node_ref.get() {
            el.set_scroll_top(el.scroll_height());
        }
    });

    view! {
        <div
            node_ref=node_ref
            data-name="ChatBody"
            class=move || cn!("overflow-hidden flex-1", class.get())
        >
            {children()}
        </div>
    }
}
