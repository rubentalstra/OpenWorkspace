use crate::components::dialog::{
    Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle,
};
use crate::{cn, slot};
use leptos::prelude::*;
use leptos_icons::Icon;
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct CommandCtx {
    query: RwSignal<String>,
    matches: RwSignal<HashMap<usize, bool>>,
    next_id: StoredValue<usize>,
}

impl CommandCtx {
    fn register(self) -> usize {
        let id = self.next_id.get_value();
        self.next_id.set_value(id + 1);
        id
    }
}

/// Command — shadcn Base UI `command` (cmdk command palette). Holds the filter
/// `query` and a registry of which `CommandItem`s currently match it, so
/// `CommandEmpty` and per-item visibility stay reactive. Compose with
/// `CommandInput`, `CommandList`, `CommandGroup`, `CommandItem`, etc.
#[component]
pub fn Command(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    provide_context(CommandCtx {
        query: RwSignal::new(String::new()),
        matches: RwSignal::new(HashMap::new()),
        next_id: StoredValue::new(0),
    });
    view! {
        <div
            data-slot="command"
            class=move || cn!("cn-command flex size-full flex-col overflow-hidden", class.get())
        >
            {children()}
        </div>
    }
}

/// CommandDialog — a `Command` mounted inside a centered modal `Dialog`. The
/// title/description are visually hidden (`sr-only`) for accessibility.
#[component]
pub fn CommandDialog(
    #[prop(optional)] open: Option<RwSignal<bool>>,
    #[prop(default = false)] default_open: bool,
    #[prop(into, default = "Command Palette".to_string())] title: String,
    #[prop(into, default = "Search for a command to run...".to_string())] description: String,
    #[prop(default = false)] show_close: bool,
    #[prop(into, optional)] class: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let children = StoredValue::new(children);
    view! {
        <Dialog open=open.unwrap_or_else(|| RwSignal::new(default_open))>
            <DialogHeader class="sr-only">
                <DialogTitle>{title}</DialogTitle>
                <DialogDescription>{description}</DialogDescription>
            </DialogHeader>
            <DialogContent
                show_close=show_close
                class=Signal::derive(move || {
                    cn!(
                        "cn-command-dialog top-1/3 translate-y-0 overflow-hidden p-0",
                        class.get(),
                    )
                })
            >
                <Command>{children.with_value(|c| c())}</Command>
            </DialogContent>
        </Dialog>
    }
}

/// CommandInput — the search field that drives the `Command` filter, with a leading
/// search icon, wrapped in the cmdk input-group structure. Pass an external `value`
/// signal to observe/control the query; otherwise the internal `Command` query is
/// used.
#[component]
pub fn CommandInput(
    #[prop(optional)] value: Option<RwSignal<String>>,
    #[prop(into, optional)] placeholder: Signal<String>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let ctx = expect_context::<CommandCtx>();
    let read = value.unwrap_or(ctx.query);
    let write = move |next: String| {
        ctx.query.set(next.clone());
        if let Some(external) = value {
            external.set(next);
        }
    };
    view! {
        <div data-slot="command-input-wrapper" class="cn-command-input-wrapper">
            <div data-slot="input-group" class="cn-command-input-group">
                <input
                    data-slot="command-input"
                    type="text"
                    placeholder=move || placeholder.get()
                    prop:value=move || read.get()
                    class=move || {
                        cn!(
                            "cn-command-input outline-hidden disabled:cursor-not-allowed disabled:opacity-50",
                            class.get(),
                        )
                    }
                    on:input=move |ev| write(event_target_value(&ev))
                />
                <span data-slot="input-group-addon">
                    <Icon icon=icondata::LuSearch attr:class="cn-command-input-icon" />
                </span>
            </div>
        </div>
    }
}

/// CommandList — the scrollable region holding groups and items.
#[component]
pub fn CommandList(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="command-list"
            class=move || { cn!("cn-command-list overflow-x-hidden overflow-y-auto", class.get()) }
        >
            {children()}
        </div>
    }
}

/// CommandEmpty — shown only when no `CommandItem` matches the current query.
#[component]
pub fn CommandEmpty(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CommandCtx>();
    let any_match = Memo::new(move |_| ctx.matches.with(|m| m.values().any(|v| *v)));
    view! {
        <div
            data-slot="command-empty"
            class:hidden=move || any_match.get()
            class=move || cn!("cn-command-empty", class.get())
        >
            {children()}
        </div>
    }
}

slot! {
    /// CommandGroup — a labelled section of items.
    CommandGroup, div, "command-group", "cn-command-group"
}

slot! {
    /// CommandSeparator — a thin divider between groups.
    CommandSeparator, div, "command-separator", "cn-command-separator"
}

/// CommandItem — a selectable row carrying a text `value`. It is hidden whenever its
/// `value` does not contain the current query (case-insensitive), and reports its
/// match state to the `Command` so `CommandEmpty` stays in sync. `selected` drives
/// the trailing check indicator (`data-checked`); `on_select` fires the `value` on
/// click.
#[component]
pub fn CommandItem(
    #[prop(into)] value: String,
    #[prop(into, optional)] selected: Signal<bool>,
    #[prop(optional)] on_select: Option<Callback<String>>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<CommandCtx>();
    let id = ctx.register();
    let haystack = value.to_lowercase();
    let matches = Memo::new(move |_| {
        let needle = ctx.query.get().to_lowercase();
        needle.is_empty() || haystack.contains(&needle)
    });
    Effect::new(move |_| {
        let hit = matches.get();
        ctx.matches.update(|m| {
            m.insert(id, hit);
        });
    });
    on_cleanup(move || {
        ctx.matches.update(|m| {
            m.remove(&id);
        });
    });
    let select_value = value;
    let highlighted = RwSignal::new(false);
    view! {
        <div
            data-slot="command-item"
            data-checked=move || selected.get().then_some("true")
            data-selected=move || highlighted.get().then_some("true")
            data-highlighted=move || highlighted.get().then_some("true")
            class:hidden=move || !matches.get()
            class=move || {
                cn!(
                    "cn-command-item group/command-item data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                    class.get(),
                )
            }
            on:pointermove=move |_| highlighted.set(true)
            on:pointerleave=move |_| highlighted.set(false)
            on:click=move |_| {
                if let Some(cb) = on_select {
                    cb.run(select_value.clone());
                }
            }
        >
            {children()}
            <Icon
                icon=icondata::LuCheck
                attr:class="cn-command-item-indicator ml-auto opacity-0 group-has-data-[slot=command-shortcut]/command-item:hidden group-data-[checked=true]/command-item:opacity-100"
            />
        </div>
    }
}

slot! {
    /// CommandShortcut — a trailing keyboard-shortcut hint inside a `CommandItem`.
    CommandShortcut, span, "command-shortcut", "cn-command-shortcut"
}
