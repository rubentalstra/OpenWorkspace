//! Chat & maps building blocks: bubbles, messages, attachments, markers, scroller.

use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{
    Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription,
    AttachmentMedia, AttachmentTitle, Bubble, BubbleAlign, BubbleContent, BubbleGroup, Marker,
    MarkerContent, MarkerIcon, Message, MessageAvatar, MessageContent, MessageGroup, MessageHeader,
    MessageScroller, MessageScrollerButton, MessageScrollerContent, MessageScrollerItem,
    MessageScrollerViewport,
};

use super::{Demo, PageShell};

/// Chat & maps building blocks: bubbles, messages, attachments, markers, scroller.
#[component]
pub fn ChatPage() -> impl IntoView {
    view! {
        <PageShell title="Chat" subtitle="Messaging and map building blocks.">
            <Demo title="Bubbles">
                <BubbleGroup class="w-full max-w-sm gap-2">
                    <Bubble align=BubbleAlign::Start>
                        <BubbleContent class="bg-muted rounded-2xl px-3 py-2 text-sm">
                            "Is desk A-12 free tomorrow?"
                        </BubbleContent>
                    </Bubble>
                    <Bubble align=BubbleAlign::End>
                        <BubbleContent class="bg-primary text-primary-foreground ml-auto rounded-2xl px-3 py-2 text-sm">
                            "Yep — booked it for you 👍"
                        </BubbleContent>
                    </Bubble>
                </BubbleGroup>
            </Demo>
            <Demo title="Message">
                <MessageGroup class="w-full max-w-sm gap-3">
                    <Message>
                        <MessageAvatar class="size-8 text-xs">"OV"</MessageAvatar>
                        <MessageContent class="gap-1">
                            <MessageHeader class="gap-2 text-sm">
                                <span class="font-medium">"Olivia"</span>
                                <span class="text-muted-foreground text-xs">"9:41"</span>
                            </MessageHeader>
                            <div class="bg-muted w-fit rounded-2xl px-3 py-2 text-sm">
                                "Heading to floor 2 now."
                            </div>
                        </MessageContent>
                    </Message>
                </MessageGroup>
            </Demo>
            <Demo title="Attachment">
                <Attachment class="w-72 gap-2 rounded-lg p-2">
                    <AttachmentMedia class="size-10 rounded-md">
                        <Icon icon=icondata::LuFileText attr:class="size-5" />
                    </AttachmentMedia>
                    <AttachmentContent class="self-center">
                        <AttachmentTitle>"floorplan.pdf"</AttachmentTitle>
                        <AttachmentDescription>"2.4 MB · PDF"</AttachmentDescription>
                    </AttachmentContent>
                    <AttachmentActions class="self-center">
                        <AttachmentAction>
                            <Icon icon=icondata::LuX attr:class="size-4" />
                        </AttachmentAction>
                    </AttachmentActions>
                </Attachment>
            </Demo>
            <Demo title="Marker">
                <div class="flex w-full flex-col gap-2 text-sm">
                    <Marker class="gap-2">
                        <MarkerIcon>
                            <Icon icon=icondata::LuMapPin attr:class="size-4" />
                        </MarkerIcon>
                        <MarkerContent>"Building A · Floor 2"</MarkerContent>
                    </Marker>
                    <Marker class="gap-2">
                        <MarkerIcon>
                            <Icon icon=icondata::LuMapPin attr:class="size-4" />
                        </MarkerIcon>
                        <MarkerContent>"Building B · Floor 1"</MarkerContent>
                    </Marker>
                </div>
            </Demo>
            <Demo title="Message scroller">
                <MessageScroller class="h-48 w-full rounded-md border">
                    <MessageScrollerViewport class="p-3">
                        <MessageScrollerContent class="gap-2">
                            {(1..=15)
                                .map(|n| {
                                    view! {
                                        <MessageScrollerItem>
                                            <div class="bg-muted rounded-md p-2 text-sm">
                                                {format!("Message {n}")}
                                            </div>
                                        </MessageScrollerItem>
                                    }
                                })
                                .collect_view()}
                        </MessageScrollerContent>
                    </MessageScrollerViewport>
                    <MessageScrollerButton />
                </MessageScroller>
            </Demo>
        </PageShell>
    }
}
