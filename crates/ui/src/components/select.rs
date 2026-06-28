use crate::hooks::focus_on_hover::focus_on_hover;
use crate::hooks::use_anchored_position::use_anchor_rect;
use crate::hooks::use_dismiss::use_dismiss;
use crate::{cn, slot};
use leptos::prelude::*;
use leptos_icons::Icon;
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct SelectCtx {
    value: RwSignal<String>,
    open: RwSignal<bool>,
    anchor: NodeRef<leptos::html::Div>,
    labels: RwSignal<HashMap<String, String>>,
    on_change: StoredValue<Option<Callback<String>>>,
}

/// Select — shadcn Base UI `select`. An anchored listbox. Controlled via an
/// external `value` signal or uncontrolled via `default_value`; opening/dismissal
/// follows the popover pattern (outside-click + Escape). The root wraps trigger +
/// content so dismissal works.
#[component]
pub fn Select(
    #[prop(optional)] value: Option<RwSignal<String>>,
    #[prop(into, optional)] default_value: String,
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let value = value.unwrap_or_else(|| RwSignal::new(default_value));
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let labels = RwSignal::new(HashMap::<String, String>::new());
    let root = NodeRef::<leptos::html::Div>::new();
    provide_context(SelectCtx {
        value,
        open,
        anchor: root,
        labels,
        on_change: StoredValue::new(on_change),
    });
    use_dismiss(open, root);
    view! {
        <div
            node_ref=root
            data-slot="select"
            class=move || cn!("relative inline-block", class.get())
        >
            {children()}
        </div>
    }
}

/// Wraps a label + a set of related items inside the content.
#[component]
pub fn SelectGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            role="group"
            data-slot="select-group"
            class=move || cn!("cn-select-group", class.get())
        >
            {children()}
        </div>
    }
}

/// Renders the chosen item's label, or `placeholder` when nothing is selected.
#[component]
pub fn SelectValue(
    #[prop(into, optional)] placeholder: String,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let ctx = expect_context::<SelectCtx>();
    let is_placeholder = move || ctx.value.get().is_empty();
    let text = move || {
        let value = ctx.value.get();
        if value.is_empty() {
            placeholder.clone()
        } else {
            ctx.labels
                .with(|labels| labels.get(&value).cloned().unwrap_or(value))
        }
    };
    view! {
        <span
            data-slot="select-value"
            data-placeholder=move || is_placeholder().then_some("true")
            class=move || cn!("cn-select-value", class.get())
        >
            {text}
        </span>
    }
}

/// The control that opens the listbox; shows the `SelectValue` and a chevron.
#[component]
pub fn SelectTrigger(
    #[prop(into, optional)] size: Signal<String>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<SelectCtx>();
    let size = Memo::new(move |_| {
        let value = size.get();
        if value.is_empty() {
            "default".to_owned()
        } else {
            value
        }
    });
    view! {
        <button
            type="button"
            role="combobox"
            data-slot="select-trigger"
            data-size=move || size.get()
            aria-haspopup="listbox"
            aria-expanded=move || ctx.open.get().to_string()
            class=move || {
                cn!(
                    "cn-select-trigger flex w-fit items-center justify-between whitespace-nowrap outline-none disabled:cursor-not-allowed disabled:opacity-50 *:data-[slot=select-value]:line-clamp-1 *:data-[slot=select-value]:flex *:data-[slot=select-value]:items-center [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
            <Icon
                icon=icondata::LuChevronDown
                attr:class="cn-select-trigger-icon pointer-events-none"
            />
        </button>
    }
}

/// The popup listbox; mounted (and enter-animated) while open.
#[component]
pub fn SelectContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<SelectCtx>();
    let position = use_anchor_rect(ctx.open, ctx.anchor).below();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                role="listbox"
                data-slot="select-content"
                data-open="true"
                data-side="bottom"
                style=move || position.get()
                class=move || {
                    cn!(
                        "cn-select-content cn-select-content-logical cn-menu-target cn-menu-translucent isolate z-50 max-h-96 origin-(--transform-origin) overflow-x-hidden overflow-y-auto data-[align-trigger=true]:animate-none",
                        class.get(),
                    )
                }
            >
                {children()}
            </div>
        </Show>
    }
}

/// A non-selectable label for a `SelectGroup`.
#[component]
pub fn SelectLabel(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div data-slot="select-label" class=move || cn!("cn-select-label", class.get())>
            {children()}
        </div>
    }
}

/// A selectable option; sets the value and closes the listbox on click.
#[component]
pub fn SelectItem(
    #[prop(into)] value: String,
    #[prop(into, optional)] label: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<SelectCtx>();
    let item_value = value;
    let selected = {
        let item_value = item_value.clone();
        Memo::new(move |_| ctx.value.get() == item_value)
    };
    if !label.is_empty() {
        let registry_value = item_value.clone();
        Effect::new(move |_| {
            let registry_value = registry_value.clone();
            let label = label.clone();
            ctx.labels.update(move |labels| {
                labels.insert(registry_value, label);
            });
        });
    }
    let on_click = {
        let item_value = item_value.clone();
        move |_| {
            ctx.value.set(item_value.clone());
            ctx.open.set(false);
            if let Some(cb) = ctx.on_change.get_value() {
                cb.run(item_value.clone());
            }
        }
    };
    view! {
        <div
            role="option"
            tabindex="-1"
            data-slot="select-item"
            data-selected=move || selected.get().then_some("true")
            aria-selected=move || selected.get().to_string()
            class=move || {
                cn!(
                    "cn-select-item relative flex w-full cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:pointermove=focus_on_hover
            on:click=on_click
        >
            <span class="cn-select-item-text shrink-0 whitespace-nowrap">{children()}</span>
            <Show when=move || selected.get()>
                <span class="cn-select-item-indicator">
                    <Icon
                        icon=icondata::LuCheck
                        attr:class="cn-select-item-indicator-icon pointer-events-none"
                    />
                </span>
            </Show>
        </div>
    }
}

slot! { SelectSeparator, div, "select-separator", "cn-select-separator pointer-events-none" }
