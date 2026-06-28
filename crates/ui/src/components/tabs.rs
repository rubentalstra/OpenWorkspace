use crate::cn;
use leptos::prelude::*;

/// Tab strip orientation.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum TabsOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl TabsOrientation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

#[derive(Clone, Copy)]
struct TabsCtx {
    selected: RwSignal<String>,
}

/// Tabs — shadcn Base UI `tabs`. Controlled via an external `value` signal or
/// uncontrolled via `default_value` (matched against each trigger/content `value`).
#[component]
pub fn Tabs(
    #[prop(optional)] value: Option<RwSignal<String>>,
    #[prop(into, optional)] default_value: String,
    #[prop(into, optional)] orientation: Signal<TabsOrientation>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let selected = value.unwrap_or_else(|| RwSignal::new(default_value));
    provide_context(TabsCtx { selected });
    view! {
        <div
            data-slot="tabs"
            data-orientation=move || orientation.get().as_str()
            class=move || cn!("cn-tabs group/tabs flex data-horizontal:flex-col", class.get())
        >
            {children()}
        </div>
    }
}

/// Tab list visual style.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum TabsListVariant {
    #[default]
    Default,
    Line,
}

impl TabsListVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Line => "line",
        }
    }
    fn class(self) -> &'static str {
        match self {
            Self::Default => "cn-tabs-list-variant-default bg-muted",
            Self::Line => "cn-tabs-list-variant-line gap-1 bg-transparent",
        }
    }
}

/// The row of tab triggers.
#[component]
pub fn TabsList(
    #[prop(into, optional)] variant: Signal<TabsListVariant>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            role="tablist"
            data-slot="tabs-list"
            data-variant=move || variant.get().as_str()
            class=move || {
                cn!(
                    "cn-tabs-list group/tabs-list inline-flex w-fit items-center justify-center text-muted-foreground group-data-vertical/tabs:h-fit group-data-vertical/tabs:flex-col",
                    variant.get().class(),
                    class.get(),
                )
            }
        >
            {children()}
        </div>
    }
}

/// A tab trigger; activates the content with the matching `value`.
#[component]
pub fn TabsTrigger(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<TabsCtx>();
    let for_memo = value.clone();
    let active = Memo::new(move |_| ctx.selected.get() == for_memo);
    view! {
        <button
            type="button"
            role="tab"
            data-slot="tabs-trigger"
            data-active=move || active.get().then_some("true")
            aria-selected=move || active.get().to_string()
            class=move || {
                cn!(
                    "cn-tabs-trigger relative inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center whitespace-nowrap text-foreground/60 transition-all group-data-vertical/tabs:w-full group-data-vertical/tabs:justify-start hover:text-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-1 focus-visible:outline-ring disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 dark:text-muted-foreground dark:hover:text-foreground [&_svg]:pointer-events-none [&_svg]:shrink-0 group-data-[variant=line]/tabs-list:bg-transparent group-data-[variant=line]/tabs-list:data-active:bg-transparent dark:group-data-[variant=line]/tabs-list:data-active:border-transparent dark:group-data-[variant=line]/tabs-list:data-active:bg-transparent data-active:bg-background data-active:text-foreground dark:data-active:border-input dark:data-active:bg-input/30 dark:data-active:text-foreground after:absolute after:bg-foreground after:opacity-0 after:transition-opacity group-data-horizontal/tabs:after:inset-x-0 group-data-horizontal/tabs:after:bottom-[-5px] group-data-horizontal/tabs:after:h-0.5 group-data-vertical/tabs:after:inset-y-0 group-data-vertical/tabs:after:-right-1 group-data-vertical/tabs:after:w-0.5 group-data-[variant=line]/tabs-list:data-active:after:opacity-100",
                    class.get(),
                )
            }
            on:click=move |_| ctx.selected.set(value.clone())
        >
            {children()}
        </button>
    }
}

/// The panel shown for the active tab `value`.
#[component]
pub fn TabsContent(
    #[prop(into)] value: String,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<TabsCtx>();
    view! {
        <div
            role="tabpanel"
            data-slot="tabs-content"
            class:hidden=move || ctx.selected.get() != value
            class=move || cn!("cn-tabs-content flex-1 outline-none", class.get())
        >
            {children()}
        </div>
    }
}
