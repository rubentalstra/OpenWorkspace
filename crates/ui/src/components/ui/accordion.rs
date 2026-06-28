use crate::{clx, cn, use_random_id};
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_icons::Icon;

clx! {
    /// Disclosure group. Wraps a column of [`AccordionItem`]s with dividers; each
    /// item manages its own open state, so several may be expanded at once.
    Accordion, div, "divide-y divide-input w-full"
}

clx! {
    /// Optional heading inside an [`AccordionTrigger`].
    AccordionTitle, h4, "text-sm font-medium"
}

clx! {
    /// Leading row inside an [`AccordionTrigger`] — aligns an icon with its title.
    AccordionHeader, div, "flex gap-2 items-center [&_svg:not([class*='size-'])]:size-4"
}

clx! {
    /// Muted body copy for use inside [`AccordionContent`].
    AccordionDescription, p, "text-muted-foreground text-sm"
}

const ACCORDION_LINK_BASE: &str = "grid gap-2.5 items-center p-2 grid-cols-[auto_1fr] [&_svg:not([class*='size-'])]:size-4 hover:bg-muted";

/// Anchor row styled to sit inside an expanded panel. Renders an `<a>` to
/// `href`; native attributes, events and children forward to it.
#[component]
pub fn AccordionLink(
    #[prop(into)] href: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <a data-name="AccordionLink" href=href class=move || cn!(ACCORDION_LINK_BASE, class.get())>
            {children()}
        </a>
    }
}

/// Per-item disclosure state plus the id wiring that ties the trigger to its
/// panel for assistive technology. Shared from [`AccordionItem`] to its
/// descendant trigger and content via context.
#[derive(Clone, Copy)]
struct AccordionItemContext {
    open: RwSignal<bool>,
    trigger_id: StoredValue<String>,
    content_id: StoredValue<String>,
}

/// A single collapsible row. Owns its open state and provides it to the nested
/// [`AccordionTrigger`] and [`AccordionContent`].
#[component]
pub fn AccordionItem(
    /// Initial open state.
    #[prop(default = false)]
    default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = AccordionItemContext {
        open: RwSignal::new(default_open),
        trigger_id: StoredValue::new(use_random_id()),
        content_id: StoredValue::new(use_random_id()),
    };
    let merged = move || cn!("w-full", class.get());

    view! {
        <Provider value=ctx>
            <div
                data-name="AccordionItem"
                data-state=move || if ctx.open.get() { "open" } else { "closed" }
                class=merged
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Glyph shown at the trailing edge of an [`AccordionTrigger`]; it rotates or
/// swaps to signal the open state.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AccordionTriggerIcon {
    #[default]
    ChevronDown,
    Plus,
}

/// Clickable header that toggles its [`AccordionItem`]. Renders a trailing
/// indicator and wires `aria-expanded`/`aria-controls` to its panel.
#[component]
pub fn AccordionTrigger(
    #[prop(into, optional)] icon: Signal<AccordionTriggerIcon>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AccordionItemContext>();
    let merged = move || {
        cn!(
            "flex w-full justify-between items-center p-3 text-left list-none cursor-pointer select-none [&_svg:not([class*='size-'])]:size-4 outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
            class.get()
        )
    };
    let indicator = move || match icon.get() {
        AccordionTriggerIcon::ChevronDown => view! {
            <Icon
                icon=icondata::LuChevronDown
                attr:class=move || {
                    cn!(
                        "transition-transform duration-300",
                        if ctx.open.get() { "rotate-180" } else { "" }
                    )
                }
            />
        }
        .into_any(),
        AccordionTriggerIcon::Plus => view! {
            <Icon
                icon=icondata::LuPlus
                attr:class=move || {
                    cn!(
                        "transition-transform duration-300",
                        if ctx.open.get() { "rotate-45" } else { "" }
                    )
                }
            />
        }
        .into_any(),
    };

    view! {
        <button
            type="button"
            data-name="AccordionTrigger"
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            id=move || ctx.trigger_id.get_value()
            aria-expanded=move || if ctx.open.get() { "true" } else { "false" }
            aria-controls=move || ctx.content_id.get_value()
            class=merged
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
            {indicator}
        </button>
    }
}

/// Animated panel revealed when its [`AccordionItem`] is open. Uses the CSS grid
/// `0fr`/`1fr` trick so height transitions without measuring the DOM.
#[component]
pub fn AccordionContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AccordionItemContext>();
    let outer = move || {
        cn!(
            "grid overflow-hidden transition-all duration-300",
            if ctx.open.get() {
                "grid-rows-[1fr]"
            } else {
                "grid-rows-[0fr]"
            }
        )
    };
    let inner = move || cn!("p-3 pt-0", class.get());

    view! {
        <article
            data-name="AccordionContent"
            role="region"
            id=move || ctx.content_id.get_value()
            aria-labelledby=move || ctx.trigger_id.get_value()
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            class=outer
        >
            <div class="min-h-0">
                <div class=inner>{children()}</div>
            </div>
        </article>
    }
}
