use crate::hooks::use_dismiss::use_dismiss;
use crate::{cn, slot};
use leptos::prelude::*;
use leptos_icons::Icon;

#[derive(Clone, Copy)]
struct ContextMenuCtx {
    open: RwSignal<bool>,
    x: RwSignal<i32>,
    y: RwSignal<i32>,
}

#[derive(Clone, Copy)]
struct ContextMenuRadioCtx {
    value: RwSignal<String>,
    on_change: StoredValue<Option<Callback<String>>>,
}

/// ContextMenu — shadcn Base UI `context-menu`. A menu that opens on right-click
/// (`contextmenu`) positioned at the cursor. Controlled via an external `open`
/// signal or uncontrolled via `default_open`; the root wraps trigger + content so
/// outside-click/Escape dismissal works.
#[component]
pub fn ContextMenu(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let root = NodeRef::<leptos::html::Div>::new();
    provide_context(ContextMenuCtx {
        open,
        x: RwSignal::new(0),
        y: RwSignal::new(0),
    });
    use_dismiss(open, root);
    view! {
        <div
            node_ref=root
            data-slot="context-menu"
            class=move || cn!("relative inline-block", class.get())
        >
            {children()}
        </div>
    }
}

/// The surface that opens the menu on right-click, anchoring it at the pointer.
#[component]
pub fn ContextMenuTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuCtx>();
    view! {
        <div
            data-slot="context-menu-trigger"
            class=move || cn!("cn-context-menu-trigger select-none", class.get())
            on:contextmenu=move |event| {
                event.prevent_default();
                ctx.x.set(event.client_x());
                ctx.y.set(event.client_y());
                ctx.open.set(true);
            }
        >
            {children()}
        </div>
    }
}

/// The menu surface; mounted (and enter-animated) while open, fixed at the cursor.
#[component]
pub fn ContextMenuContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuCtx>();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                data-slot="context-menu-content"
                data-open="true"
                data-side="right"
                class=move || {
                    cn!(
                        "cn-context-menu-content cn-context-menu-content-logical cn-menu-target cn-menu-translucent fixed z-50 max-h-(--available-height) origin-(--transform-origin) overflow-x-hidden overflow-y-auto outline-none",
                        class.get(),
                    )
                }
                style=move || format!("left: {}px; top: {}px;", ctx.x.get(), ctx.y.get())
            >
                {children()}
            </div>
        </Show>
    }
}

/// Groups related items for spacing and labelling.
#[component]
pub fn ContextMenuGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div role="group" data-slot="context-menu-group" class=move || cn!("", class.get())>
            {children()}
        </div>
    }
}

/// A non-interactive group label.
#[component]
pub fn ContextMenuLabel(
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="context-menu-label"
            data-inset=inset.then_some("true")
            class=move || cn!("cn-context-menu-label", class.get())
        >
            {children()}
        </div>
    }
}

/// Item tone, surfaced as `data-variant` for the nova layer.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ContextMenuItemVariant {
    #[default]
    Default,
    Destructive,
}

impl ContextMenuItemVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Destructive => "destructive",
        }
    }
}

/// A selectable menu item; closes the menu and fires `on_select` when activated.
#[component]
pub fn ContextMenuItem(
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] variant: Signal<ContextMenuItemVariant>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_select: Option<Callback<()>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuCtx>();
    view! {
        <div
            role="menuitem"
            data-slot="context-menu-item"
            data-inset=inset.then_some("true")
            data-variant=move || variant.get().as_str()
            data-disabled=disabled.then_some("true")
            aria-disabled=disabled.then_some("true")
            class=move || {
                cn!(
                    "cn-context-menu-item group/context-menu-item relative flex cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=move |_| {
                if disabled {
                    return;
                }
                if let Some(cb) = on_select {
                    cb.run(());
                }
                ctx.open.set(false);
            }
        >
            {children()}
        </div>
    }
}

/// A checkable menu item; controlled via `checked` + `on_change`.
#[component]
pub fn ContextMenuCheckboxItem(
    #[prop(into, optional)] checked: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
    #[prop(default = false)] inset: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            role="menuitemcheckbox"
            data-slot="context-menu-checkbox-item"
            data-inset=inset.then_some("true")
            data-disabled=disabled.then_some("true")
            aria-checked=move || checked.get().to_string()
            aria-disabled=disabled.then_some("true")
            class=move || {
                cn!(
                    "cn-context-menu-checkbox-item relative flex cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=move |_| {
                if disabled {
                    return;
                }
                if let Some(cb) = on_change {
                    cb.run(!checked.get_untracked());
                }
            }
        >
            <span class="cn-context-menu-item-indicator pointer-events-none">
                <Show when=move || checked.get()>
                    <Icon icon=icondata::LuCheck />
                </Show>
            </span>
            {children()}
        </div>
    }
}

/// Groups radio items into a single-selection set.
#[component]
pub fn ContextMenuRadioGroup(
    #[prop(optional)] value: Option<RwSignal<String>>,
    #[prop(into, optional)] default_value: String,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let value = value.unwrap_or_else(|| RwSignal::new(default_value));
    provide_context(ContextMenuRadioCtx {
        value,
        on_change: StoredValue::new(on_change),
    });
    view! {
        <div role="group" data-slot="context-menu-radio-group" class=move || cn!("", class.get())>
            {children()}
        </div>
    }
}

/// A single radio option, identified by its `value`; reads selection from
/// the enclosing [`ContextMenuRadioGroup`].
#[component]
pub fn ContextMenuRadioItem(
    #[prop(into)] value: String,
    #[prop(default = false)] inset: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuRadioCtx>();
    let for_memo = value.clone();
    let checked = Memo::new(move |_| ctx.value.get() == for_memo);
    view! {
        <div
            role="menuitemradio"
            data-slot="context-menu-radio-item"
            data-inset=inset.then_some("true")
            data-disabled=disabled.then_some("true")
            aria-checked=move || checked.get().to_string()
            aria-disabled=disabled.then_some("true")
            class=move || {
                cn!(
                    "cn-context-menu-radio-item relative flex cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=move |_| {
                if disabled {
                    return;
                }
                ctx.value.set(value.clone());
                if let Some(cb) = ctx.on_change.get_value() {
                    cb.run(value.clone());
                }
            }
        >
            <span class="cn-context-menu-item-indicator pointer-events-none">
                <Show when=move || checked.get()>
                    <Icon icon=icondata::LuCheck />
                </Show>
            </span>
            {children()}
        </div>
    }
}

/// A submenu root; provides hover/focus open state to its trigger + content.
#[component]
pub fn ContextMenuSub(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    provide_context(ContextMenuSubCtx { open });
    view! {
        <div data-slot="context-menu-sub" class="relative">
            {children()}
        </div>
    }
}

#[derive(Clone, Copy)]
struct ContextMenuSubCtx {
    open: RwSignal<bool>,
}

/// Opens the nested submenu; renders a trailing chevron.
#[component]
pub fn ContextMenuSubTrigger(
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuSubCtx>();
    view! {
        <div
            role="menuitem"
            aria-haspopup="menu"
            data-slot="context-menu-sub-trigger"
            data-inset=inset.then_some("true")
            data-open=move || ctx.open.get().then_some("true")
            aria-expanded=move || ctx.open.get().to_string()
            class=move || {
                cn!(
                    "cn-context-menu-sub-trigger flex cursor-default items-center outline-hidden select-none [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:pointerenter=move |_| ctx.open.set(true)
            on:pointerleave=move |_| ctx.open.set(false)
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
            <Icon icon=icondata::LuChevronRight attr:class="cn-rtl-flip ml-auto" />
        </div>
    }
}

/// The nested submenu surface; mounted while the submenu is open.
#[component]
pub fn ContextMenuSubContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<ContextMenuSubCtx>();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                data-slot="context-menu-sub-content"
                data-open="true"
                data-side="right"
                class=move || {
                    cn!(
                        "cn-context-menu-subcontent cn-context-menu-sub-content cn-menu-target cn-menu-translucent absolute top-0 left-full z-50 max-h-(--available-height) origin-(--transform-origin) overflow-x-hidden overflow-y-auto outline-none",
                        class.get(),
                    )
                }
                on:pointerenter=move |_| ctx.open.set(true)
                on:pointerleave=move |_| ctx.open.set(false)
            >
                {children()}
            </div>
        </Show>
    }
}

slot! { ContextMenuSeparator, div, "context-menu-separator", "cn-context-menu-separator" }
slot! { ContextMenuShortcut, span, "context-menu-shortcut", "cn-context-menu-shortcut" }
