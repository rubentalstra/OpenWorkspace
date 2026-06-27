use crate::{clx, cn};
use leptos::context::Provider;
use leptos::prelude::*;

clx! {
    /// Vertical stack of [`FaqSection`] disclosures.
    Faq, div, "flex flex-col gap-3 w-full max-w-screen-md"
}
clx! {
    /// Heading rendered above a [`Faq`] group.
    FaqTitle, span, "text-lg text-primary"
}
clx! {
    /// Supporting copy rendered under a [`FaqTitle`].
    FaqDescription, p, "pr-6 mb-2 text-muted-foreground"
}

#[derive(Clone, Copy)]
struct FaqSectionContext {
    open: RwSignal<bool>,
}

fn data_state(open: RwSignal<bool>) -> &'static str {
    if open.get() { "open" } else { "closed" }
}

/// A single expandable FAQ entry. Owns the open/closed state shared with its
/// [`FaqLabel`] and [`FaqContent`] through context.
///
/// Pass `open` to drive the state externally (controlled); otherwise an
/// internal signal seeded from `default_open` is used. Native attributes,
/// events and bindings forward to the root element.
#[component]
pub fn FaqSection(
    /// External signal driving open/closed; when omitted the section manages
    /// its own state seeded from `default_open`.
    #[prop(optional)]
    open: Option<RwSignal<bool>>,
    /// Initial open state when uncontrolled.
    #[prop(default = false)]
    default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));

    view! {
        <Provider value=FaqSectionContext { open }>
            <div
                data-name="FaqSection"
                data-state=move || data_state(open)
                class=move || {
                    cn!("w-full rounded bg-accent/30 hover:bg-accent flex flex-col", class.get())
                }
            >
                {children()}
            </div>
        </Provider>
    }
}

/// Clickable header that toggles its enclosing [`FaqSection`]. Reflects the
/// section state via `aria-expanded` and `data-state` for styling.
#[component]
pub fn FaqLabel(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Button>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<FaqSectionContext>();

    view! {
        <button
            node_ref=node_ref
            type="button"
            data-name="FaqLabel"
            data-state=move || data_state(ctx.open)
            aria-expanded=move || if ctx.open.get() { "true" } else { "false" }
            class=move || {
                cn!(
                    "flex justify-between items-center py-2 px-4 mt-2 w-full cursor-pointer text-left",
                    class.get(),
                )
            }
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
        </button>
    }
}

/// Panel revealed when the enclosing [`FaqSection`] is open. Uses the CSS
/// grid-rows reveal so height animates without JavaScript.
#[component]
pub fn FaqContent(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<FaqSectionContext>();

    view! {
        <div
            data-name="FaqContent"
            data-state=move || data_state(ctx.open)
            class="grid overflow-hidden mt-2 transition-all duration-500 data-[state=closed]:grid-rows-[0fr] data-[state=open]:grid-rows-[1fr]"
        >
            <div class=move || cn!("min-h-0 px-4", class.get())>{children()}</div>
        </div>
    }
}
