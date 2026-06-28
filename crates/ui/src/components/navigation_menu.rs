use crate::cn;
use crate::hooks::use_dismiss::use_dismiss;
use leptos::prelude::*;
use leptos_icons::Icon;

/// The semantic base classes for a navigation-menu trigger (the Rust analogue of
/// shadcn's variant-less `navigationMenuTriggerStyle` cva). Apply to any element
/// you want to look like a navigation-menu trigger.
pub const NAVIGATION_MENU_TRIGGER_STYLE: &str = "cn-navigation-menu-trigger group/navigation-menu-trigger inline-flex h-9 w-max items-center justify-center outline-none disabled:pointer-events-none";

#[derive(Clone, Copy)]
struct NavigationMenuCtx {
    /// `value` of the item whose content is currently open, if any.
    active: RwSignal<Option<String>>,
}

#[derive(Clone)]
struct NavigationMenuItemCtx {
    value: String,
    open: RwSignal<bool>,
}

/// NavigationMenu — shadcn Base UI `navigation-menu`. A horizontal nav whose items
/// can each reveal an anchored content panel. Only one item's content is open at a
/// time; clicking outside or pressing Escape dismisses it.
///
/// This is a simplified, JS-free viewport: each item's content is anchored directly
/// under that item rather than animated through a shared positioner/viewport.
#[component]
pub fn NavigationMenu(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let active = RwSignal::new(None::<String>);
    provide_context(NavigationMenuCtx { active });
    view! {
        <nav
            data-slot="navigation-menu"
            data-viewport="false"
            class=move || {
                cn!(
                    "cn-navigation-menu group/navigation-menu relative flex max-w-max flex-1 items-center justify-center",
                    class.get(),
                )
            }
        >
            {children()}
        </nav>
    }
}

/// The horizontal list of navigation-menu items.
#[component]
pub fn NavigationMenuList(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <ul
            data-slot="navigation-menu-list"
            class=move || {
                cn!(
                    "cn-navigation-menu-list group flex flex-1 list-none items-center justify-center",
                    class.get(),
                )
            }
        >
            {children()}
        </ul>
    }
}

/// A single navigation-menu item; wraps its own trigger + content so outside-click
/// and Escape dismissal are scoped to it. `value` identifies the item; when omitted
/// the trigger still toggles but cannot be cross-referenced.
#[component]
pub fn NavigationMenuItem(
    #[prop(into, optional)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<NavigationMenuCtx>();
    let open = RwSignal::new(false);
    let root = NodeRef::<leptos::html::Div>::new();
    let value_for_ctx = value.clone();

    // Keep the per-item `open` flag in sync with the shared active value, so only
    // one item is open at a time and outside-click can close the active one.
    let value_for_sync = value.clone();
    Effect::new(move |_| {
        let is_active = ctx.active.get().as_deref() == Some(value_for_sync.as_str());
        if open.get_untracked() != is_active {
            open.set(is_active);
        }
    });
    let value_for_dismiss = value;
    Effect::new(move |_| {
        if !open.get() && ctx.active.get_untracked().as_deref() == Some(value_for_dismiss.as_str())
        {
            ctx.active.set(None);
        }
    });

    use_dismiss(open, root);
    provide_context(NavigationMenuItemCtx {
        value: value_for_ctx,
        open,
    });
    view! {
        <li class="contents">
            <div
                node_ref=root
                data-slot="navigation-menu-item"
                class=move || cn!("cn-navigation-menu-item relative", class.get())
            >
                {children()}
            </div>
        </li>
    }
}

/// The control that toggles its item's content panel. Renders the trigger label
/// followed by a chevron that rotates when open.
#[component]
pub fn NavigationMenuTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let menu = expect_context::<NavigationMenuCtx>();
    let item = expect_context::<NavigationMenuItemCtx>();
    let value = item.value.clone();
    view! {
        <button
            type="button"
            data-slot="navigation-menu-trigger"
            data-popup-open=move || item.open.get().then_some("true")
            aria-expanded=move || item.open.get().to_string()
            class=move || cn!(NAVIGATION_MENU_TRIGGER_STYLE, "group", class.get())
            on:click=move |_| {
                let value = value.clone();
                menu.active
                    .update(|active| {
                        *active = if active.as_deref() == Some(value.as_str()) {
                            None
                        } else {
                            Some(value)
                        };
                    });
            }
        >
            {children()}
            <Icon
                icon=icondata::LuChevronDown
                attr:class="cn-navigation-menu-trigger-icon"
                attr:aria-hidden="true"
            />
        </button>
    }
}

/// The anchored content panel for its item; mounted while the item is open.
#[component]
pub fn NavigationMenuContent(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let item = expect_context::<NavigationMenuItemCtx>();
    view! {
        <Show when=move || item.open.get() fallback=|| ()>
            <div
                data-slot="navigation-menu-content"
                data-open="true"
                data-side="bottom"
                class=move || {
                    cn!(
                        "cn-navigation-menu-content absolute top-full left-0 z-50 mt-1.5 h-full w-auto",
                        class.get(),
                    )
                }
            >
                {children()}
            </div>
        </Show>
    }
}

/// A navigable link inside the navigation menu. Renders an `<a>`; pass `active` to
/// mark it as the current page. Supply `href` for the destination.
#[component]
pub fn NavigationMenuLink(
    #[prop(into, optional)] href: Option<String>,
    #[prop(into, optional)] active: Signal<bool>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <a
            data-slot="navigation-menu-link"
            href=href
            data-active=move || active.get().then_some("true")
            aria-current=move || active.get().then_some("page")
            class=move || cn!("cn-navigation-menu-link", class.get())
        >
            {children()}
        </a>
    }
}

/// The active-item indicator arrow rendered under the list. Visible while any item
/// is open.
#[component]
pub fn NavigationMenuIndicator(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let menu = expect_context::<NavigationMenuCtx>();
    view! {
        <div
            data-slot="navigation-menu-indicator"
            data-state=move || if menu.active.get().is_some() { "visible" } else { "hidden" }
            class=move || {
                cn!(
                    "cn-navigation-menu-indicator top-full z-1 flex h-1.5 items-end justify-center overflow-hidden",
                    class.get(),
                )
            }
        >
            <div class="cn-navigation-menu-indicator-arrow relative top-[60%] size-2 rotate-45"></div>
        </div>
    }
}

/// The shared viewport surface that frames open content. In this JS-free port the
/// content is anchored per item, so this is a thin styled container you may place
/// after the list if you want the popover chrome.
#[component]
pub fn NavigationMenuViewport(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let menu = expect_context::<NavigationMenuCtx>();
    view! {
        <div
            class="absolute top-full left-0 isolate z-50 flex justify-center"
            data-open=move || menu.active.get().is_some().then_some("true")
        >
            <div
                data-slot="navigation-menu-viewport"
                data-open=move || menu.active.get().is_some().then_some("true")
                class=move || {
                    cn!(
                        "cn-navigation-menu-viewport relative h-(--popup-height) w-(--popup-width) origin-(--transform-origin) overflow-hidden",
                        class.get(),
                    )
                }
            ></div>
        </div>
    }
}
