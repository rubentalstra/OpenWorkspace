use crate::cn;
use leptos::prelude::*;
use leptos_icons::Icon;

#[derive(Clone, Copy)]
struct AccordionItemCtx {
    open: RwSignal<bool>,
}

/// Accordion — shadcn Base UI `accordion`. Each `AccordionItem` toggles
/// independently (multiple may be open).
#[component]
pub fn Accordion(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="accordion"
            class=move || cn!("cn-accordion flex w-full flex-col", class.get())
        >
            {children()}
        </div>
    }
}

/// A single accordion section.
#[component]
pub fn AccordionItem(
    #[prop(default = false)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    provide_context(AccordionItemCtx {
        open: RwSignal::new(default_open),
    });
    view! {
        <div data-slot="accordion-item" class=move || cn!("cn-accordion-item", class.get())>
            {children()}
        </div>
    }
}

/// The clickable header that toggles its item; shows a chevron that flips when open.
#[component]
pub fn AccordionTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AccordionItemCtx>();
    view! {
        <h3 class="flex">
            <button
                type="button"
                data-slot="accordion-trigger"
                aria-expanded=move || ctx.open.get().to_string()
                class=move || {
                    cn!(
                        "cn-accordion-trigger group/accordion-trigger relative flex flex-1 items-start justify-between border border-transparent transition-all outline-none aria-disabled:pointer-events-none aria-disabled:opacity-50",
                        class.get(),
                    )
                }
                on:click=move |_| ctx.open.update(|open| *open = !*open)
            >
                {children()}
                <Icon
                    icon=icondata::LuChevronDown
                    attr:data-slot="accordion-trigger-icon"
                    attr:class="cn-accordion-trigger-icon pointer-events-none shrink-0 group-aria-expanded/accordion-trigger:hidden"
                />
                <Icon
                    icon=icondata::LuChevronUp
                    attr:data-slot="accordion-trigger-icon"
                    attr:class="cn-accordion-trigger-icon pointer-events-none hidden shrink-0 group-aria-expanded/accordion-trigger:inline"
                />
            </button>
        </h3>
    }
}

/// The collapsible body for an item.
#[component]
pub fn AccordionContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<AccordionItemCtx>();
    view! {
        <div
            data-slot="accordion-content"
            class="cn-accordion-content overflow-hidden"
            class:hidden=move || !ctx.open.get()
        >
            <div class=move || {
                cn!(
                    "cn-accordion-content-inner [&_a]:underline [&_a]:underline-offset-3 [&_a]:hover:text-foreground [&_p:not(:last-child)]:mb-4",
                    class.get(),
                )
            }>{children()}</div>
        </div>
    }
}
