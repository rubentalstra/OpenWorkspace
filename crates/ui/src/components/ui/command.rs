use crate::{
    Button, ButtonVariant, clx, cn, use_lock_body_scroll, use_random_id, use_random_id_for,
};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::portal::Portal;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

clx! {
    /// Heading block for a [`CommandDialog`]; hidden on small screens and
    /// left-aligned from the `sm` breakpoint up.
    CommandHeader, div, "flex flex-col gap-2 text-center hidden sm:text-left"
}
clx! {
    /// Prominent heading inside a [`CommandHeader`].
    CommandTitle, h2, "text-lg font-semibold leading-none"
}
clx! {
    /// Supporting copy beneath a [`CommandTitle`].
    CommandDescription, p, "text-sm text-muted-foreground"
}
clx! {
    /// Section heading inside a [`CommandGroup`].
    CommandGroupLabel, div, "text-muted-foreground px-2 py-1.5 text-xs font-medium"
}
clx! {
    /// Footer rail for a [`CommandDialog`]; holds hints such as keyboard shortcuts.
    CommandFooter, footer, "flex gap-4 items-center px-4 h-10 text-xs font-medium rounded-b-xl border-t text-muted-foreground border-t-border bg-muted"
}

/// Registration record for one navigable option, contributed by a
/// [`CommandItem`]/[`CommandItemLink`] to its [`Command`] root. Ordered by `seq`
/// so keyboard navigation visits options in document order, and carries the
/// lowercased `label` that drives client-side filtering and type-ahead.
#[derive(Clone)]
struct CommandItemMeta {
    id: String,
    label: String,
    seq: u64,
    disabled: bool,
}

/// Shared command state: the search query that filters options, the registry of
/// navigable options, the active (highlighted) option id, and the filtering
/// switch. Provided by [`Command`] to every descendant part through context.
#[derive(Clone, Copy)]
struct CommandContext {
    query: RwSignal<String>,
    items: RwSignal<Vec<CommandItemMeta>>,
    active: RwSignal<Option<String>>,
    should_filter: bool,
}

impl CommandContext {
    fn matches(&self, label: &str) -> bool {
        if !self.should_filter {
            return true;
        }
        let needle = self.query.get().to_lowercase();
        let needle = needle.trim();
        needle.is_empty() || label.to_lowercase().contains(needle)
    }

    fn visible(&self) -> Vec<CommandItemMeta> {
        let query = self.query.get().to_lowercase();
        let query = query.trim().to_owned();
        let mut items: Vec<CommandItemMeta> = self
            .items
            .get()
            .into_iter()
            .filter(|item| {
                !item.disabled
                    && (!self.should_filter || query.is_empty() || item.label.contains(&query))
            })
            .collect();
        items.sort_by_key(|item| item.seq);
        items
    }

    fn move_active(&self, forward: bool) {
        let visible = self.visible();
        if visible.is_empty() {
            self.active.set(None);
            return;
        }
        let current = self
            .active
            .get_untracked()
            .and_then(|id| visible.iter().position(|item| item.id == id));
        let next = match current {
            Some(i) if forward => (i + 1) % visible.len(),
            Some(0) => visible.len() - 1,
            Some(i) => i - 1,
            None if forward => 0,
            None => visible.len() - 1,
        };
        self.active
            .set(visible.get(next).map(|item| item.id.clone()));
    }

    fn activate_edge(&self, first: bool) {
        let visible = self.visible();
        let target = if first {
            visible.first()
        } else {
            visible.last()
        };
        self.active.set(target.map(|item| item.id.clone()));
    }

    fn click_active(&self) {
        let Some(id) = self.active.get_untracked() else {
            return;
        };
        let Some(el) = document().get_element_by_id(&id) else {
            return;
        };
        if let Some(el) = el.dyn_ref::<web_sys::HtmlElement>() {
            el.click();
        }
    }
}

/// Open state and id wiring for a [`CommandDialog`], shared with its trigger,
/// panel and the command palette it hosts so the trigger can be refocused on
/// close and the palette can dismiss the dialog after a selection.
#[derive(Clone, Copy)]
struct CommandDialogContext {
    open: RwSignal<bool>,
    title_id: StoredValue<String>,
    trigger_id: StoredValue<String>,
}

/// Root for a modal command palette. Owns the open state (seeded from
/// `default_open`) and shares it with the trigger and panel; opening is also
/// bound to the global Cmd/Ctrl+K shortcut and to `/` outside text fields.
#[component]
pub fn CommandDialogProvider(
    /// External signal driving open/closed; when omitted an internal signal
    /// seeded from `default_open` is used.
    #[prop(optional)]
    open: Option<RwSignal<bool>>,
    /// Initial open state when uncontrolled.
    #[prop(default = false)]
    default_open: bool,
    children: Children,
) -> impl IntoView {
    let open = open.unwrap_or_else(|| RwSignal::new(default_open));
    let ctx = CommandDialogContext {
        open,
        title_id: StoredValue::new(use_random_id_for("command_title")),
        trigger_id: StoredValue::new(use_random_id_for("command_trigger")),
    };

    let handle = window_event_listener(ev::keydown, move |event| {
        let key = event.key();
        let open_combo = (event.meta_key() || event.ctrl_key()) && key == "k";
        let slash = key == "/" && !open.get_untracked() && !in_text_field();
        if open_combo || slash {
            event.prevent_default();
            open.set(true);
        }
    });
    on_cleanup(move || handle.remove());

    view! { <Provider value=ctx>{children()}</Provider> }
}

/// Button that opens the enclosing [`CommandDialogProvider`]. Defaults to the
/// outline variant and carries the id the panel refocuses on close.
#[component]
pub fn CommandDialogTrigger(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CommandDialogContext>();

    view! {
        <Button
            attr:data-name="CommandDialogTrigger"
            variant=ButtonVariant::Outline
            class=class
            attr:id=move || ctx.trigger_id.get_value()
            attr:aria-haspopup="dialog"
            attr:aria-expanded=move || if ctx.open.get() { "true" } else { "false" }
            on:click=move |_| ctx.open.set(true)
        >
            {children()}
        </Button>
    }
}

/// Modal panel and backdrop for a [`CommandDialogProvider`], portalled to the
/// document body and rendered only while open. Locks body scrolling, focuses the
/// panel on open, returns focus to the trigger on close, and dismisses on Escape
/// or a backdrop click. Host a [`Command`] inside it.
#[component]
pub fn CommandDialog(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<CommandDialogContext>();
    let panel_ref = NodeRef::<html::Div>::new();
    let children = StoredValue::new(children);

    let locked = use_lock_body_scroll(false);
    Effect::new(move |was_open: Option<bool>| {
        let is_open = ctx.open.get();
        locked.set(is_open);
        if is_open {
            if let Some(el) = panel_ref.get() {
                _ = el.focus();
            }
        } else if was_open == Some(true)
            && let Some(el) = document().get_element_by_id(&ctx.trigger_id.get_value())
            && let Some(el) = el.dyn_ref::<web_sys::HtmlElement>()
        {
            _ = el.focus();
        }
        is_open
    });

    let handle = window_event_listener(ev::keydown, move |event| {
        if event.key() == "Escape" && ctx.open.get_untracked() {
            event.prevent_default();
            ctx.open.set(false);
        }
    });
    on_cleanup(move || handle.remove());

    let panel = Signal::derive(move || {
        cn!(
            "grid fixed z-100 gap-4 p-2 w-full bg-clip-padding rounded-xl ring-4 shadow-2xl outline-none sm:max-w-lg bg-background top-[50%] left-[50%] max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] ring-neutral-200/80",
            class.get()
        )
    });

    view! {
        <Show when=move || ctx.open.get()>
            <Portal>
                <div
                    data-name="CommandDialogBackdrop"
                    aria-hidden="true"
                    class="fixed inset-0 z-60 bg-black/50"
                    on:click=move |_| ctx.open.set(false)
                />
                <div
                    node_ref=panel_ref
                    data-name="CommandDialog"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby=move || ctx.title_id.get_value()
                    tabindex="-1"
                    class=panel
                >
                    {children.get_value()()}
                </div>
            </Portal>
        </Show>
    }
}

/// Command palette container. Owns the search query and the registry of
/// navigable options and shares them with the nested [`CommandInput`],
/// [`CommandList`] and items through context. When hosted inside a
/// [`CommandDialog`] it resets its query and active option each time the dialog
/// opens. Set `should_filter` to false to drive results server-side.
#[component]
pub fn Command(
    #[prop(into, optional)] class: Signal<String>,
    /// Disable client-side filtering when results are produced server-side.
    #[prop(default = true)]
    should_filter: bool,
    children: Children,
) -> impl IntoView {
    let ctx = CommandContext {
        query: RwSignal::new(String::new()),
        items: RwSignal::new(Vec::new()),
        active: RwSignal::new(None),
        should_filter,
    };
    provide_context(ctx);

    if let Some(dialog) = use_context::<CommandDialogContext>() {
        Effect::new(move |was_open: Option<bool>| {
            let is_open = dialog.open.get();
            if is_open && was_open != Some(true) {
                ctx.query.set(String::new());
            }
            is_open
        });
    }

    Effect::new(move |_| {
        let visible = ctx.visible();
        let active_valid = ctx
            .active
            .get_untracked()
            .is_some_and(|id| visible.iter().any(|item| item.id == id));
        if !active_valid {
            ctx.active.set(visible.first().map(|item| item.id.clone()));
        }
    });

    let merged = move || {
        cn!(
            "flex overflow-hidden flex-col w-full h-full bg-transparent rounded-none text-popover-foreground",
            class.get()
        )
    };

    view! {
        <div data-name="Command" class=merged tabindex="-1">
            {children()}
        </div>
    }
}

/// Filter input for a [`Command`] palette. Carries `role="combobox"`, binds to
/// the shared query, and drives option highlighting: ArrowUp/Down and Home/End
/// move the active option, Enter activates it. `on_search_change` reports each
/// query change for server-side search.
#[component]
pub fn CommandInput(
    #[prop(into, optional)] class: Signal<String>,
    /// Fired on every query change; use to drive server-side search.
    #[prop(optional)]
    on_search_change: Option<Callback<String>>,
    #[prop(optional)] node_ref: NodeRef<html::Input>,
) -> impl IntoView {
    let ctx = expect_context::<CommandContext>();
    let merged = move || {
        cn!(
            "flex py-3 w-full h-10 text-sm bg-transparent rounded-md disabled:opacity-50 disabled:cursor-not-allowed placeholder:text-muted-foreground outline-hidden",
            class.get()
        )
    };

    let on_keydown = move |event: ev::KeyboardEvent| match event.key().as_str() {
        "ArrowDown" => {
            event.prevent_default();
            ctx.move_active(true);
        }
        "ArrowUp" => {
            event.prevent_default();
            ctx.move_active(false);
        }
        "Home" => {
            event.prevent_default();
            ctx.activate_edge(true);
        }
        "End" => {
            event.prevent_default();
            ctx.activate_edge(false);
        }
        "Enter" => {
            event.prevent_default();
            ctx.click_active();
        }
        _ => {}
    };

    view! {
        <input
            node_ref=node_ref
            data-name="CommandInput"
            class=merged
            autocomplete="off"
            spellcheck="false"
            aria-autocomplete="list"
            role="combobox"
            aria-expanded="true"
            type="text"
            prop:value=move || ctx.query.get()
            on:input=move |event| {
                let value = event_target_value(&event);
                ctx.query.set(value.clone());
                ctx.active.set(ctx.visible().first().map(|item| item.id.clone()));
                if let Some(callback) = on_search_change {
                    callback.run(value);
                }
            }
            on:keydown=on_keydown
            data-1p-ignore="true"
            data-bwignore="true"
            data-lpignore="true"
        />
    }
}

/// Scrollable results region for a [`Command`] palette, carrying `role="listbox"`
/// for the options it contains.
#[component]
pub fn CommandList(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        cn!(
            "overflow-y-auto overflow-x-hidden max-h-[300px] scroll-py-1 min-h-80 scroll-pt-2 scroll-pb-1.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
            class.get()
        )
    };

    view! {
        <div data-name="CommandList" role="listbox" class=merged>
            {children()}
        </div>
    }
}

/// Labelled grouping inside a [`CommandList`]. Collapses to nothing when none of
/// its options match the current query, so empty sections disappear during
/// filtering.
#[component]
pub fn CommandGroup(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CommandContext>();
    let group_id = StoredValue::new(use_random_id());

    let has_visible = move || {
        if !ctx.should_filter {
            return true;
        }
        let group_prefix = format!("{}__opt_", group_id.get_value());
        ctx.items.get().iter().any(|item| {
            !item.disabled && item.id.starts_with(&group_prefix) && ctx.matches(&item.label)
        })
    };

    let merged = move || {
        cn!(
            "overflow-hidden p-1 text-foreground",
            if has_visible() { "" } else { "hidden" },
            class.get()
        )
    };

    view! {
        <Provider value=CommandGroupContext {
            prefix: group_id,
        }>
            <div data-name="CommandGroup" role="group" class=merged>
                {children()}
            </div>
        </Provider>
    }
}

/// Id prefix shared from a [`CommandGroup`] to its items so the group can tell
/// which registered options belong to it when deciding whether to collapse.
#[derive(Clone, Copy)]
struct CommandGroupContext {
    prefix: StoredValue<String>,
}

/// Placeholder shown when no option matches the current query. Renders only while
/// the palette has zero visible options.
#[component]
pub fn CommandEmpty(
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let ctx = expect_context::<CommandContext>();
    let merged = move || cn!("py-6 text-sm text-center", class.get());

    view! {
        <Show when=move || ctx.visible().is_empty()>
            <div data-name="CommandEmpty" class=merged>
                {children()}
            </div>
        </Show>
    }
}

/// Selectable option in a [`Command`] palette. Registers itself for keyboard
/// navigation and filtering (matched against `value`), carries `role="option"`
/// with `aria-selected` tracking the active highlight, and hides when it does not
/// match the query. Runs `on_select` and, inside a [`CommandDialog`], closes it.
#[component]
pub fn CommandItem(
    #[prop(into, optional)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] on_select: Option<Callback<()>>,
    /// Disable selection and skip the option during keyboard navigation.
    #[prop(default = false)]
    disabled: bool,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        cn!(
            "group relative flex gap-2 items-center px-2 py-1.5 text-sm rounded-sm cursor-default hover:cursor-pointer select-none outline-none data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50 aria-selected:bg-muted hover:bg-muted",
            class.get()
        )
    };

    command_option(
        "CommandItem",
        OptionTag::Div,
        value,
        merged,
        on_select,
        disabled,
        children,
    )
}

/// Anchor variant of [`CommandItem`] for options that navigate. Same registration,
/// filtering and highlight behaviour, rendered as an `<a>`.
#[component]
pub fn CommandItemLink(
    #[prop(into, optional)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] on_select: Option<Callback<()>>,
    /// Disable selection and skip the option during keyboard navigation.
    #[prop(default = false)]
    disabled: bool,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        cn!(
            "[&_svg:not([class*='text-'])]:text-muted-foreground relative flex cursor-default hover:cursor-pointer items-center gap-2 px-2 py-1.5 text-sm outline-hidden select-none data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 aria-selected:bg-muted/50 aria-selected:text-accent-foreground hover:bg-muted h-9 rounded-md border border-transparent font-medium",
            class.get()
        )
    };

    command_option(
        "CommandItemLink",
        OptionTag::Anchor,
        value,
        merged,
        on_select,
        disabled,
        children,
    )
}

#[derive(Clone, Copy)]
enum OptionTag {
    Div,
    Anchor,
}

/// Shared body for [`CommandItem`] and [`CommandItemLink`]. Registers the option
/// with its [`Command`] root on mount (and a [`CommandGroup`] prefix when nested),
/// deregisters on cleanup, reflects the active highlight via `aria-selected`, and
/// hides itself when filtered out.
#[expect(
    clippy::needless_pass_by_value,
    reason = "owned so the returned view does not capture the argument's lifetime"
)]
fn command_option(
    data_name: &'static str,
    tag: OptionTag,
    value: String,
    class: impl Fn() -> String + Send + Sync + 'static,
    on_select: Option<Callback<()>>,
    disabled: bool,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CommandContext>();
    let group = use_context::<CommandGroupContext>();

    let seq = next_seq();
    let prefix = group
        .map(|g| g.prefix.get_value())
        .unwrap_or_else(use_random_id);
    let id = StoredValue::new(format!("{prefix}__opt_{seq}"));
    let label = StoredValue::new(value.to_lowercase());

    {
        let meta = CommandItemMeta {
            id: id.get_value(),
            label: label.get_value(),
            seq,
            disabled,
        };
        ctx.items.update(|items| items.push(meta));
    }
    on_cleanup(move || {
        let target = id.get_value();
        ctx.items
            .update(|items| items.retain(|item| item.id != target));
    });

    let is_visible = move || ctx.matches(&label.get_value());
    let is_active = move || ctx.active.get().as_deref() == Some(id.get_value().as_str());

    let on_click = move |_| {
        if disabled {
            return;
        }
        if let Some(callback) = on_select {
            callback.run(());
        }
        if let Some(dialog) = use_context::<CommandDialogContext>() {
            dialog.open.set(false);
        }
    };

    match tag {
        OptionTag::Div => view! {
            <div
                id=move || id.get_value()
                data-name=data_name
                class=class
                role="option"
                aria-selected=move || is_active().to_string()
                data-disabled=disabled.to_string()
                style:display=move || if is_visible() { "flex" } else { "none" }
                on:mousemove=move |_| {
                    if !disabled {
                        ctx.active.set(Some(id.get_value()));
                    }
                }
                on:click=on_click
            >
                {children()}
            </div>
        }
        .into_any(),
        OptionTag::Anchor => view! {
            <a
                id=move || id.get_value()
                data-name=data_name
                class=class
                role="option"
                aria-selected=move || is_active().to_string()
                data-disabled=disabled.to_string()
                style:display=move || if is_visible() { "flex" } else { "none" }
                on:mousemove=move |_| {
                    if !disabled {
                        ctx.active.set(Some(id.get_value()));
                    }
                }
                on:click=on_click
            >
                {children()}
            </a>
        }
        .into_any(),
    }
}

/// Reports whether the active element is a text-entry field, so the `/` global
/// shortcut does not hijack typing. Runs only inside the keydown handler, so the
/// `web_sys` access never executes during server rendering.
fn in_text_field() -> bool {
    let Some(active) = document().active_element() else {
        return false;
    };
    if let Some(el) = active.dyn_ref::<web_sys::HtmlElement>()
        && el.is_content_editable()
    {
        return true;
    }
    matches!(active.tag_name().as_str(), "INPUT" | "TEXTAREA")
}

/// Monotonic ordering source for registered options so navigation follows
/// document order regardless of mount timing.
fn next_seq() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
