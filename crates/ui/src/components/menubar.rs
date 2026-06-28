use crate::hooks::use_dismiss::use_dismiss;
use crate::{cn, slot};
use leptos::prelude::*;
use leptos_icons::Icon;

#[derive(Clone, Copy)]
struct MenubarCtx {
    /// The id of the currently open menu, or empty when the bar is closed.
    active: RwSignal<String>,
}

#[derive(Clone)]
struct MenubarMenuCtx {
    id: String,
    open: RwSignal<bool>,
}

#[derive(Clone, Copy)]
struct MenubarSubCtx {
    open: RwSignal<bool>,
}

#[derive(Clone, Copy)]
struct MenubarRadioCtx {
    value: RwSignal<String>,
    on_change: StoredValue<Option<Callback<String>>>,
}

/// Item tone, surfaced as `data-variant` for the nova layer.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum MenubarItemVariant {
    #[default]
    Default,
    Destructive,
}

impl MenubarItemVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Destructive => "destructive",
        }
    }
}

/// Menubar — shadcn Base UI `menubar`. A horizontal bar of menus where only one
/// menu is open at a time. Each [`MenubarMenu`] is an anchored, dismissible popup.
#[component]
pub fn Menubar(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    provide_context(MenubarCtx {
        active: RwSignal::new(String::new()),
    });
    view! {
        <div data-slot="menubar" class=move || cn!("cn-menubar flex items-center", class.get())>
            {children()}
        </div>
    }
}

/// A single menu within the bar. Wraps its trigger + content so outside-click and
/// Escape dismissal work; opening one menu closes any sibling that was open.
#[component]
pub fn MenubarMenu(#[prop(into)] value: String, children: Children) -> impl IntoView {
    let bar = expect_context::<MenubarCtx>();
    let id = value.clone();
    let open = RwSignal::new(false);
    let root = NodeRef::<leptos::html::Div>::new();

    Effect::new(move |_| {
        let is_active = bar.active.get() == id;
        if open.get_untracked() != is_active {
            open.set(is_active);
        }
    });

    provide_context(MenubarMenuCtx {
        id: value.clone(),
        open,
    });
    use_dismiss(open, root);

    let close_id = value;
    Effect::new(move |_| {
        if !open.get() && bar.active.get_untracked() == close_id {
            bar.active.set(String::new());
        }
    });

    view! {
        <div node_ref=root data-slot="menubar-menu" class="relative inline-block">
            {children()}
        </div>
    }
}

/// The control that toggles its menu.
#[component]
pub fn MenubarTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let bar = expect_context::<MenubarCtx>();
    let ctx = expect_context::<MenubarMenuCtx>();
    let open = ctx.open;
    let toggle_id = ctx.id.clone();
    let hover_id = ctx.id.clone();
    view! {
        <button
            type="button"
            data-slot="menubar-trigger"
            aria-haspopup="menu"
            aria-expanded=move || open.get().to_string()
            class=move || {
                cn!(
                    "cn-menubar-trigger flex items-center outline-hidden select-none",
                    class.get(),
                )
            }
            on:click=move |_| {
                if open.get_untracked() {
                    bar.active.set(String::new());
                } else {
                    bar.active.set(toggle_id.clone());
                }
            }
            on:pointerenter=move |_| {
                if !bar.active.get_untracked().is_empty() {
                    bar.active.set(hover_id.clone());
                }
            }
        >
            {children()}
        </button>
    }
}

/// The menu panel; mounted (and enter-animated) while its menu is open.
#[component]
pub fn MenubarContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let open = expect_context::<MenubarMenuCtx>().open;
    view! {
        <Show when=move || open.get() fallback=|| ()>
            <div
                role="menu"
                data-slot="menubar-content"
                data-open="true"
                data-side="bottom"
                class=move || {
                    cn!(
                        "cn-menubar-content cn-menubar-content-logical cn-menu-target cn-menu-translucent absolute top-full left-0 z-50 mt-1 origin-(--transform-origin) outline-hidden",
                        class.get(),
                    )
                }
            >
                {children()}
            </div>
        </Show>
    }
}

/// A selectable menu command. Closes the menu on selection. Set `inset` to align
/// with indicator-bearing siblings; `variant` switches to the destructive tone.
#[component]
pub fn MenubarItem(
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] variant: Signal<MenubarItemVariant>,
    #[prop(optional)] on_select: Option<Callback<()>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let bar = expect_context::<MenubarCtx>();
    view! {
        <button
            type="button"
            role="menuitem"
            data-slot="menubar-item"
            data-inset=inset.then_some("true")
            data-variant=move || variant.get().as_str()
            class=move || {
                cn!(
                    "cn-menubar-item group/menubar-item relative flex w-full cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=move |_| {
                if let Some(cb) = on_select {
                    cb.run(());
                }
                bar.active.set(String::new());
            }
        >
            {children()}
        </button>
    }
}

/// A checkable menu item. Controlled via `checked`; `on_change` fires on toggle.
#[component]
pub fn MenubarCheckboxItem(
    #[prop(into, optional)] checked: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let toggle = move |_| {
        if let Some(cb) = on_change {
            cb.run(!checked.get_untracked());
        }
    };
    view! {
        <button
            type="button"
            role="menuitemcheckbox"
            data-slot="menubar-checkbox-item"
            data-inset=inset.then_some("true")
            aria-checked=move || checked.get().to_string()
            data-checked=move || checked.get().to_string()
            class=move || {
                cn!(
                    "cn-menubar-checkbox-item relative flex w-full cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:click=toggle
        >
            <span class="cn-menubar-checkbox-item-indicator pointer-events-none absolute flex items-center justify-center">
                <Show when=move || checked.get()>
                    <Icon icon=icondata::LuCheck />
                </Show>
            </span>
            {children()}
        </button>
    }
}

/// Groups [`MenubarRadioItem`]s sharing one selected `value`.
#[component]
pub fn MenubarRadioGroup(
    #[prop(optional)] value: Option<RwSignal<String>>,
    #[prop(into, optional)] default_value: String,
    #[prop(optional)] on_change: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let value = value.unwrap_or_else(|| RwSignal::new(default_value));
    provide_context(MenubarRadioCtx {
        value,
        on_change: StoredValue::new(on_change),
    });
    view! {
        <div role="group" data-slot="menubar-radio-group">
            {children()}
        </div>
    }
}

/// A single radio option inside a [`MenubarRadioGroup`], identified by its `value`.
#[component]
pub fn MenubarRadioItem(
    #[prop(into)] value: String,
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MenubarRadioCtx>();
    let for_memo = value.clone();
    let checked = Memo::new(move |_| ctx.value.get() == for_memo);
    view! {
        <button
            type="button"
            role="menuitemradio"
            data-slot="menubar-radio-item"
            data-inset=inset.then_some("true")
            aria-checked=move || checked.get().to_string()
            data-checked=move || checked.get().to_string()
            class=move || {
                cn!(
                    "cn-menubar-radio-item relative flex w-full cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0",
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
            <span class="cn-menubar-radio-item-indicator pointer-events-none absolute flex items-center justify-center">
                <Show when=move || checked.get()>
                    <Icon icon=icondata::LuCheck />
                </Show>
            </span>
            {children()}
        </button>
    }
}

/// A non-interactive label inside a menu or group.
#[component]
pub fn MenubarLabel(
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="menubar-label"
            data-inset=inset.then_some("true")
            class=move || cn!("cn-menubar-label", class.get())
        >
            {children()}
        </div>
    }
}

/// A nested submenu. Wraps its sub-trigger + sub-content so outside-click/Escape
/// dismissal works for the submenu independently of the parent menu.
#[component]
pub fn MenubarSub(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let root = NodeRef::<leptos::html::Div>::new();
    provide_context(MenubarSubCtx { open });
    use_dismiss(open, root);
    view! {
        <div node_ref=root data-slot="menubar-sub" class=move || cn!("relative", class.get())>
            {children()}
        </div>
    }
}

/// The control that opens its submenu.
#[component]
pub fn MenubarSubTrigger(
    #[prop(default = false)] inset: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MenubarSubCtx>();
    view! {
        <button
            type="button"
            role="menuitem"
            data-slot="menubar-sub-trigger"
            data-inset=inset.then_some("true")
            data-open=move || ctx.open.get().then_some("true")
            aria-haspopup="menu"
            aria-expanded=move || ctx.open.get().to_string()
            class=move || {
                cn!(
                    "cn-menubar-sub-trigger flex w-full cursor-default items-center outline-hidden select-none [&_svg]:pointer-events-none [&_svg]:shrink-0",
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

/// The submenu panel; mounted while the submenu is open.
#[component]
pub fn MenubarSubContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<MenubarSubCtx>();
    view! {
        <Show when=move || ctx.open.get() fallback=|| ()>
            <div
                role="menu"
                data-slot="menubar-sub-content"
                data-open="true"
                data-side="right"
                class=move || {
                    cn!(
                        "cn-menubar-sub-content cn-menu-target cn-menu-translucent absolute top-0 left-full z-50 ml-1 origin-(--transform-origin) outline-hidden",
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
    /// Groups related menu items under an optional [`MenubarLabel`].
    MenubarGroup, div, "menubar-group", ""
}
slot! {
    /// Structural portal hook; rendered inline (we do not teleport content).
    MenubarPortal, div, "menubar-portal", ""
}
slot! {
    /// A thin horizontal rule between menu sections.
    MenubarSeparator, div, "menubar-separator", "cn-menubar-separator -mx-1 my-1 h-px"
}
slot! {
    /// Right-aligned keyboard hint shown beside an item's label.
    MenubarShortcut, span, "menubar-shortcut", "cn-menubar-shortcut ml-auto"
}
