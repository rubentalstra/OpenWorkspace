use crate::{Input, Textarea, clx, cn, variants};
use leptos::html;
use leptos::prelude::*;

const INPUT_GROUP_BASE: &str = "group/input-group border-input dark:bg-input/30 relative flex w-full items-center rounded-md border shadow-xs transition-[color,box-shadow] outline-none h-9 min-w-0 has-[>textarea]:h-auto has-[>[data-align=inline-start]]:[&>input]:pl-2 has-[>[data-align=inline-end]]:[&>input]:pr-2 has-[>[data-align=block-start]]:h-auto has-[>[data-align=block-start]]:flex-col has-[>[data-align=block-start]]:[&>input]:pb-3 has-[>[data-align=block-end]]:h-auto has-[>[data-align=block-end]]:flex-col has-[>[data-align=block-end]]:[&>input]:pt-3 has-[[data-slot=input-group-control]:focus-visible]:border-ring has-[[data-slot=input-group-control]:focus-visible]:ring-ring/50 has-[[data-slot=input-group-control]:focus-visible]:ring-[3px] has-[[data-slot][aria-invalid=true]]:ring-destructive/20 has-[[data-slot][aria-invalid=true]]:border-destructive dark:has-[[data-slot][aria-invalid=true]]:ring-destructive/40";

const INPUT_GROUP_ADDON_BASE: &str = "text-muted-foreground flex h-auto cursor-text items-center justify-center gap-2 py-1.5 text-sm font-medium select-none [&>svg:not([class*='size-'])]:size-4 [&>kbd]:rounded-[calc(var(--radius)-5px)] group-data-[disabled=true]/input-group:opacity-50";

const INPUT_GROUP_CONTROL_BASE: &str = "flex-1 rounded-none border-0 bg-transparent shadow-none focus-visible:ring-0 dark:bg-transparent";

clx! {
    /// Inline label or hint text for an [`InputGroupAddon`]. Styles icons and
    /// keyboard hints rendered alongside the text.
    InputGroupText, span,
    "text-muted-foreground flex items-center gap-2 text-sm [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4"
}

/// Edge an [`InputGroupAddon`] sits on relative to the control: inline (leading
/// or trailing) or block (above or below).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum InputGroupAddonAlign {
    #[default]
    InlineStart,
    InlineEnd,
    BlockStart,
    BlockEnd,
}

impl InputGroupAddonAlign {
    fn class(self) -> &'static str {
        match self {
            Self::InlineStart => {
                "order-first pl-3 has-[>button]:ml-[-0.45rem] has-[>kbd]:ml-[-0.35rem]"
            }
            Self::InlineEnd => {
                "order-last pr-3 has-[>button]:mr-[-0.45rem] has-[>kbd]:mr-[-0.35rem]"
            }
            Self::BlockStart => {
                "order-first w-full justify-start px-3 pt-3 [.border-b]:pb-3 group-has-[>input]/input-group:pt-2.5"
            }
            Self::BlockEnd => {
                "order-last w-full justify-start px-3 pb-3 [.border-t]:pt-3 group-has-[>input]/input-group:pb-2.5"
            }
        }
    }

    fn data_align(self) -> &'static str {
        match self {
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
            Self::BlockStart => "block-start",
            Self::BlockEnd => "block-end",
        }
    }
}

variants! {
    /// Action button sized to sit inside an [`InputGroupAddon`] without breaking
    /// the group's height. Renders a bare `<button>`; set `attr:type` at the call
    /// site.
    InputGroupButton {
        base: "inline-flex items-center justify-center gap-2 whitespace-nowrap text-sm font-medium shadow-none transition-all outline-none disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50 focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] [&_svg]:pointer-events-none [&_svg]:shrink-0",
        variants: {
            variant: {
                Ghost: "",
            },
            size: {
                Xs: "h-6 gap-1 px-2 rounded-[calc(var(--radius)-5px)] [&>svg:not([class*='size-'])]:size-3.5 has-[>svg]:px-2",
                Sm: "h-8 px-2.5 gap-1.5 rounded-md has-[>svg]:px-2.5",
                IconXs: "size-6 rounded-[calc(var(--radius)-5px)] p-0 has-[>svg]:p-0",
                IconSm: "size-8 p-0 has-[>svg]:p-0",
            }
        },
        component: {
            element: button
        }
    }
}

/// Container that fuses a control with leading and trailing addons into a single
/// bordered surface. Place an [`InputGroupInput`] or [`InputGroupTextarea`]
/// alongside one or more [`InputGroupAddon`]s as children.
#[component]
pub fn InputGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-name="InputGroup"
            data-slot="input-group"
            role="group"
            class=move || cn!(INPUT_GROUP_BASE, class.get())
        >
            {children()}
        </div>
    }
}

/// Slot for content attached to an [`InputGroup`] control — icons, text via
/// [`InputGroupText`], or an [`InputGroupButton`]. `align` selects the edge.
#[component]
pub fn InputGroupAddon(
    #[prop(into, optional)] align: Signal<InputGroupAddonAlign>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-name="InputGroupAddon"
            data-slot="input-group-addon"
            data-align=move || align.get().data_align()
            role="group"
            class=move || cn!(INPUT_GROUP_ADDON_BASE, align.get().class(), class.get())
        >
            {children()}
        </div>
    }
}

/// Text input wired as the [`InputGroup`] control. Forwards every native
/// `<input>` attribute, event and binding to the underlying element.
#[component]
pub fn InputGroupInput(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Input>,
) -> impl IntoView {
    view! {
        <Input
            node_ref=node_ref
            attr:data-slot="input-group-control"
            class=Signal::derive(move || cn!(INPUT_GROUP_CONTROL_BASE, class.get()))
        />
    }
}

/// Multi-line text field wired as the [`InputGroup`] control. Forwards every
/// native `<textarea>` attribute, event and binding to the underlying element.
#[component]
pub fn InputGroupTextarea(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Textarea>,
) -> impl IntoView {
    view! {
        <Textarea
            node_ref=node_ref
            attr:data-slot="input-group-control"
            class=Signal::derive(move || {
                cn!(
                    "flex-1 resize-none rounded-none border-0 bg-transparent py-3 shadow-none focus-visible:ring-0 dark:bg-transparent",
                    class.get(),
                )
            })
        />
    }
}
