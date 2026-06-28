use crate::{Button, ButtonSize, Textarea, clx, cn};
use leptos::html;
use leptos::prelude::*;

const PROMPT_BASE: &str = "group/input-group border-input dark:bg-input/30 relative flex w-full min-w-0 flex-col overflow-hidden rounded-md border shadow-xs transition-[color,box-shadow] outline-none has-[textarea:focus-visible]:border-ring has-[textarea:focus-visible]:ring-ring/50 has-[textarea:focus-visible]:ring-[3px]";

const PROMPT_TEXTAREA_BASE: &str = "field-sizing-content max-h-48 min-h-[52px] flex-1 resize-none rounded-none border-0 bg-transparent px-3 py-3 text-sm shadow-none outline-none focus-visible:ring-0 dark:bg-transparent placeholder:text-muted-foreground";

const PROMPT_FOOTER_BASE: &str =
    "flex w-full items-center justify-between gap-1 border-t px-2 py-2";

clx! {
    /// Left-aligned cluster of prompt actions (attachments, model picker, …)
    /// inside an [`InputPromptFooter`].
    InputPromptTools, div, "flex items-center gap-1"
}

/// Prompt composer shell — a bordered column holding an
/// [`InputPromptTextarea`] above an [`InputPromptFooter`]. Native attributes
/// forward to the root `<div>`.
#[component]
pub fn InputPrompt(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div data-name="InputPrompt" role="group" class=move || cn!(PROMPT_BASE, class.get())>
            {children()}
        </div>
    }
}

/// Auto-growing prompt textarea. Submits via `on_submit` when Enter is pressed
/// without Shift; every other native attribute and event (e.g. `prop:value` +
/// `on:input` for two-way control) forwards to the underlying `<textarea>`.
#[component]
pub fn InputPromptTextarea(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Textarea>,
    #[prop(optional)] on_submit: Option<Callback<()>>,
) -> impl IntoView {
    let submit = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            if let Some(cb) = on_submit {
                cb.run(());
            }
        }
    };

    view! {
        <Textarea
            node_ref=node_ref
            attr:data-slot="input-group-control"
            attr:rows="1"
            class=Signal::derive(move || cn!(PROMPT_TEXTAREA_BASE, class.get()))
            on:keydown=submit
        />
    }
}

/// Block-end action row of an [`InputPrompt`] — tools on the leading edge, the
/// submit control on the trailing edge.
#[component]
pub fn InputPromptFooter(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-name="InputPromptFooter"
            data-slot="input-group-addon"
            data-align="block-end"
            role="group"
            class=move || cn!(PROMPT_FOOTER_BASE, class.get())
        >
            {children()}
        </div>
    }
}

/// Round submit button for an [`InputPrompt`]. Disable it while the prompt is
/// empty by forwarding `attr:disabled` from a derived signal at the call site.
#[component]
pub fn InputPromptSubmit(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <Button
            size=ButtonSize::Icon
            class=Signal::derive(move || cn!("size-8 rounded-full", class.get()))
            attr:r#type="button"
        >
            {children()}
        </Button>
    }
}
