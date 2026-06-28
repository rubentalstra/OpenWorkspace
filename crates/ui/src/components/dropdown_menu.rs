use crate::hooks::use_anchored_position::use_anchor_rect;
use crate::hooks::use_dismiss::use_dismiss;
use crate::{cn, slot};
use leptos::prelude::*;
use leptos_icons::Icon;

#[derive(Clone, Copy)]
struct DropdownMenuCtx {
    open: RwSignal<bool>,
    anchor: NodeRef<leptos::html::Div>,
}

#[derive(Clone, Copy)]
struct DropdownMenuRadioCtx {
    value: RwSignal<String>,
    on_change: StoredValue<Option<Callback<String>>>,
}

/// Item tone, surfaced as `data-variant` for the nova layer.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DropdownMenuItemVariant {
    #[default]
    Default,
    Destructive,
}

impl DropdownMenuItemVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Destructive => "destructive",
        }
    }
}

/// DropdownMenu — shadcn Base UI `dropdown-menu` (Menu primitive). An anchored,
/// dismissible menu. Controlled via an external `open` signal or uncontrolled via
/// `default_open`. The root wraps trigger + content so outside-click/Escape
/// dismissal works.
#[component]
pub fn DropdownMenu(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let root = NodeRef::<leptos::html::Div>::new();
    provide_context(DropdownMenuCtx { open, anchor: root });
    use_dismiss(open, root);
    view! {
        <div
            node_ref=root
            data-slot="dropdown-menu"
            class=move || cn!("relative inline-block", class.get())
        >
            {children()}
        </div>
    }
}

/// The control that toggles the menu.
#[component]
pub fn DropdownMenuTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DropdownMenuCtx>();
    view! {
        <button
            type="button"
            data-slot="dropdown-menu-trigger"
            aria-haspopup="menu"
            aria-expanded=move || ctx.open.get().to_string()
            class=move || cn!("", class.get())
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
        </button>
    }
}

/// The menu panel; mounted (and enter-animated) while open.
#[component]
pub fn DropdownMenuContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<DropdownMenuCtx>();
    let position = use_anchor_rect(ctx.open, ctx.anchor).below();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                data-slot="dropdown-menu-content"
                role="menu"
                data-open="true"
                data-side="bottom"
                style=move || position.get()
                class=move || {
                    cn!(
                        "cn-dropdown-menu-content cn-dropdown-menu-content-logical cn-menu-target cn-menu-translucent z-50 max-h-96 min-w-32 origin-(--transform-origin) overflow-x-hidden overflow-y-auto outline-none data-closed:overflow-hidden",
                        class.get(),
                    )
                }
            >
                {children()}
            </div>
        </Show>
    }
}

slot! {
    /// Groups related menu items under an optional `DropdownMenuLabel`.
    DropdownMenuGroup, div, "dropdown-menu-group", ""
}

/// A non-interactive group heading. Set `inset` to align with item indicators.
#[component]
pub fn DropdownMenuLabel(
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="dropdown-menu-label"
            data-inset=inset.then_some("true")
            class=move || cn!("cn-dropdown-menu-label", class.get())
        >
            {children()}
        </div>
    }
}

/// A selectable menu command. Closes the menu on click. Set `inset` to align with
/// indicator-bearing siblings; `variant` switches to the destructive tone.
#[component]
pub fn DropdownMenuItem(
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] variant: Signal<DropdownMenuItemVariant>,
    #[prop(optional)] on_select: Option<Callback<()>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DropdownMenuCtx>();
    view! {
        <button
            type="button"
            role="menuitem"
            data-slot="dropdown-menu-item"
            data-inset=inset.then_some("true")
            data-variant=move || variant.get().as_str()
            class=move || {
                cn!(
                    "cn-dropdown-menu-item group/dropdown-menu-item relative flex w-full cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=move |_| {
                if let Some(cb) = on_select {
                    cb.run(());
                }
                ctx.open.set(false);
            }
        >
            {children()}
        </button>
    }
}

/// A toggleable menu command with a leading check indicator. Controlled: read
/// `checked`, react to `on_change`. Does not auto-close on toggle.
#[component]
pub fn DropdownMenuCheckboxItem(
    #[prop(into, optional)] checked: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            role="menuitemcheckbox"
            data-slot="dropdown-menu-checkbox-item"
            data-inset=inset.then_some("true")
            aria-checked=move || checked.get().to_string()
            data-checked=move || checked.get().to_string()
            class=move || {
                cn!(
                    "cn-dropdown-menu-checkbox-item relative flex w-full cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=move |_| {
                if let Some(cb) = on_change {
                    cb.run(!checked.get_untracked());
                }
            }
        >
            <span
                class="cn-dropdown-menu-item-indicator pointer-events-none"
                data-slot="dropdown-menu-checkbox-item-indicator"
            >
                <Show when=move || checked.get()>
                    <Icon icon=icondata::LuCheck />
                </Show>
            </span>
            {children()}
        </button>
    }
}

/// Wraps `DropdownMenuRadioItem`s into a single-selection group. Controlled via an
/// external `value` signal or uncontrolled via `default_value`; `on_change` fires
/// on selection.
#[component]
pub fn DropdownMenuRadioGroup(
    #[prop(optional)] value: Option<RwSignal<String>>,
    #[prop(into, optional)] default_value: String,
    #[prop(optional)] on_change: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let value = value.unwrap_or_else(|| RwSignal::new(default_value));
    provide_context(DropdownMenuRadioCtx {
        value,
        on_change: StoredValue::new(on_change),
    });
    view! {
        <div role="group" data-slot="dropdown-menu-radio-group">
            {children()}
        </div>
    }
}

/// A single radio option in a `DropdownMenuRadioGroup`, identified by its `value`.
#[component]
pub fn DropdownMenuRadioItem(
    #[prop(into)] value: String,
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DropdownMenuRadioCtx>();
    let for_memo = value.clone();
    let checked = Memo::new(move |_| ctx.value.get() == for_memo);
    view! {
        <button
            type="button"
            role="menuitemradio"
            data-slot="dropdown-menu-radio-item"
            data-inset=inset.then_some("true")
            aria-checked=move || checked.get().to_string()
            data-checked=move || checked.get().to_string()
            class=move || {
                cn!(
                    "cn-dropdown-menu-radio-item relative flex w-full cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=move |_| {
                ctx.value.set(value.clone());
                if let Some(cb) = ctx.on_change.get_value() {
                    cb.run(value.clone());
                }
            }
        >
            <span
                class="cn-dropdown-menu-item-indicator pointer-events-none"
                data-slot="dropdown-menu-radio-item-indicator"
            >
                <Show when=move || checked.get()>
                    <Icon icon=icondata::LuCheck />
                </Show>
            </span>
            {children()}
        </button>
    }
}

slot! {
    /// A thin horizontal rule between menu sections.
    DropdownMenuSeparator, div, "dropdown-menu-separator", "cn-dropdown-menu-separator"
}
slot! {
    /// Right-aligned keyboard hint shown beside an item's label.
    DropdownMenuShortcut, span, "dropdown-menu-shortcut", "cn-dropdown-menu-shortcut"
}

/// A nested submenu root. Wraps its `DropdownMenuSubTrigger` + `DropdownMenuSubContent`
/// so outside-click/Escape dismissal works for the submenu independently.
#[component]
pub fn DropdownMenuSub(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let root = NodeRef::<leptos::html::Div>::new();
    provide_context(DropdownMenuCtx { open, anchor: root });
    use_dismiss(open, root);
    view! {
        <div node_ref=root data-slot="dropdown-menu-sub" class=move || cn!("relative", class.get())>
            {children()}
        </div>
    }
}

/// The item that opens a submenu, with a trailing chevron.
#[component]
pub fn DropdownMenuSubTrigger(
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<DropdownMenuCtx>();
    view! {
        <button
            type="button"
            role="menuitem"
            data-slot="dropdown-menu-sub-trigger"
            data-inset=inset.then_some("true")
            aria-haspopup="menu"
            aria-expanded=move || ctx.open.get().to_string()
            data-open=move || ctx.open.get().then_some("true")
            class=move || {
                cn!(
                    "cn-dropdown-menu-sub-trigger flex w-full cursor-default items-center outline-hidden select-none data-popup-open:bg-accent data-popup-open:text-accent-foreground [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=move |_| ctx.open.update(|open| *open = !*open)
        >
            {children()}
            <Icon icon=icondata::LuChevronRight attr:class="cn-rtl-flip ml-auto" />
        </button>
    }
}

/// The nested submenu panel; mounted (and enter-animated) while its sub is open.
/// Anchored to the right of the trigger.
#[component]
pub fn DropdownMenuSubContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<DropdownMenuCtx>();
    let position = use_anchor_rect(ctx.open, ctx.anchor).right_of();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                data-slot="dropdown-menu-sub-content"
                role="menu"
                data-open="true"
                data-side="right"
                style=move || position.get()
                class=move || {
                    cn!(
                        "cn-dropdown-menu-content cn-dropdown-menu-content-logical cn-dropdown-menu-sub-content cn-menu-target cn-menu-translucent z-50 w-auto origin-(--transform-origin) overflow-x-hidden overflow-y-auto outline-none data-closed:overflow-hidden",
                        class.get(),
                    )
                }
            >
                {children()}
            </div>
        </Show>
    }
}
