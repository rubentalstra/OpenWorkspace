use crate::components::button::{Button, ButtonVariant};
use crate::{cn, slot};
use leptos::prelude::*;

// shadcn's input-group focus styling targets [data-slot=input-group-control], so
// the controls inline the input/textarea base rather than wrapping Input/Textarea.
const INPUT_BASE: &str = "cn-input w-full min-w-0 outline-none file:inline-flex file:border-0 file:bg-transparent file:text-foreground placeholder:text-muted-foreground disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50";
const TEXTAREA_BASE: &str = "cn-textarea flex field-sizing-content min-h-16 w-full outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50";

/// InputGroup — shadcn Base UI `input-group`. Wraps an input with leading/trailing
/// addons (icons, buttons, text).
#[component]
pub fn InputGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="input-group"
            role="group"
            class=move || {
                cn!(
                    "group/input-group cn-input-group relative flex w-full min-w-0 items-center outline-none has-[>textarea]:h-auto",
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// Where an addon sits relative to the control.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum InputGroupAddonAlign {
    #[default]
    InlineStart,
    InlineEnd,
    BlockStart,
    BlockEnd,
}

impl InputGroupAddonAlign {
    fn as_str(self) -> &'static str {
        match self {
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
            Self::BlockStart => "block-start",
            Self::BlockEnd => "block-end",
        }
    }
    fn class(self) -> &'static str {
        match self {
            Self::InlineStart => "cn-input-group-addon-align-inline-start order-first",
            Self::InlineEnd => "cn-input-group-addon-align-inline-end order-last",
            Self::BlockStart => {
                "cn-input-group-addon-align-block-start order-first w-full justify-start"
            }
            Self::BlockEnd => {
                "cn-input-group-addon-align-block-end order-last w-full justify-start"
            }
        }
    }
}

/// A leading/trailing addon region.
#[component]
pub fn InputGroupAddon(
    #[prop(into, optional)] align: Signal<InputGroupAddonAlign>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            role="group"
            data-slot="input-group-addon"
            data-align=move || align.get().as_str()
            class=move || {
                cn!(
                    "cn-input-group-addon flex cursor-text items-center justify-center select-none",
                    align.get().class(),
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// Compact button size for use inside an input group.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum InputGroupButtonSize {
    #[default]
    Xs,
    Sm,
    IconXs,
    IconSm,
}

impl InputGroupButtonSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::IconXs => "icon-xs",
            Self::IconSm => "icon-sm",
        }
    }
    fn class(self) -> &'static str {
        match self {
            Self::Xs => "cn-input-group-button-size-xs",
            Self::Sm => "cn-input-group-button-size-sm",
            Self::IconXs => "cn-input-group-button-size-icon-xs",
            Self::IconSm => "cn-input-group-button-size-icon-sm",
        }
    }
}

/// A small button inside an input group (ghost by default).
#[component]
pub fn InputGroupButton(
    #[prop(into, optional, default = ButtonVariant::Ghost.into())] variant: Signal<ButtonVariant>,
    #[prop(into, optional)] size: Signal<InputGroupButtonSize>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <Button
            variant=variant
            attr:data-size=move || size.get().as_str()
            class=Signal::derive(move || {
                cn!(
                    "cn-input-group-button flex items-center shadow-none", size.get().class(), class.get()
                )
            })
        >
            {children()}
        </Button>
    }
}

slot! {
    InputGroupText, span, "input-group-text",
    "cn-input-group-text flex items-center [&_svg]:pointer-events-none"
}

/// The text input control of an input group.
#[component]
pub fn InputGroupInput(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Input>,
) -> impl IntoView {
    view! {
        <input
            node_ref=node_ref
            data-slot="input-group-control"
            class=move || cn!(INPUT_BASE, "cn-input-group-input flex-1", class.get())
        />
    }
}

/// The textarea control of an input group.
#[component]
pub fn InputGroupTextarea(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Textarea>,
) -> impl IntoView {
    view! {
        <textarea
            node_ref=node_ref
            data-slot="input-group-control"
            class=move || {
                cn!(TEXTAREA_BASE, "cn-input-group-textarea flex-1 resize-none", class.get())
            }
        />
    }
}
