use crate::cn;
use leptos::attribute_interceptor::AttributeInterceptor;
use leptos::html;
use leptos::prelude::*;
use leptos_icons::Icon;

const SELECT_NATIVE_BASE: &str = "peer inline-flex w-full cursor-pointer appearance-none items-center rounded-lg h-9 pe-8 ps-3 border border-input bg-background shadow-sm shadow-black/5 transition-shadow text-sm text-foreground focus-visible:border-ring focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/20 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 has-[option[disabled]:checked]:text-muted-foreground";

/// Styled native `<select>` with a trailing chevron. Pass `<option>`s as
/// children; forwarded native attributes and events land on the inner
/// `<select>` — set `attr:id`, `attr:name`, `prop:value`, `on:change`, etc. at
/// the call site (`bind:` is not available on a component; pair `prop:value`
/// with `on:change` for two-way control).
#[component]
pub fn SelectNative(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Select>,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <AttributeInterceptor let:attrs>
            <div data-name="SelectNative" class="relative">
                <select
                    {..attrs}
                    node_ref=node_ref
                    class=move || cn!(SELECT_NATIVE_BASE, class.get())
                >
                    {children()}
                </select>
                <span class="flex absolute inset-y-0 justify-center items-center w-9 h-full pointer-events-none end-0 text-muted-foreground/80 peer-disabled:opacity-50 [&_svg:not([class*='size-'])]:size-4">
                    <Icon icon=icondata::LuChevronDown />
                </span>
            </div>
        </AttributeInterceptor>
    }
}
