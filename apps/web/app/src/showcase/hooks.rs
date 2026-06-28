use std::time::Duration;

use leptos::html::Div;
use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{
    Button, ButtonGroup, ButtonVariant, DATA_SCROLL_TARGET, HorizontalScrollState, LockableParam,
    UseHistory, UseLocks, use_can_scroll_vertical, use_copy_clipboard, use_data_scrolled,
    use_history, use_horizontal_scroll, use_locks, use_random_id, use_random_id_for,
    use_random_transition_name,
};

use super::{Demo, Page, Section};

/// The reactive hooks behind the kit, shown in isolation.
#[component]
pub fn HooksPage() -> impl IntoView {
    // `use_history` and `use_locks` read their state from context; both must be
    // initialized at the top of the page before any descendant reads them.
    let _history = UseHistory::init();
    let _locks = UseLocks::init();

    view! {
        <Page title="Hooks" subtitle="The reactive hooks behind the kit, shown in isolation.">
            <HistorySection />
            <LocksSection />
            <ClipboardSection />
            <DataScrolledSection />
            <RandomSection />
            <VerticalScrollSection />
            <HorizontalScrollSection />
        </Page>
    }
}

/// Small monospace pill used to surface a hook's live readout value.
#[component]
fn Readout(#[prop(into)] label: String, children: Children) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 text-sm">
            <span class="text-muted-foreground">{label}</span>
            <span class="rounded bg-muted px-2 py-0.5 font-mono text-xs text-foreground">
                {children()}
            </span>
        </div>
    }
}

#[component]
fn HistorySection() -> impl IntoView {
    let history = use_history();
    let position = history.position();
    let total = history.total();
    let current = history.current();
    let can_back = history.can_go_back();
    let can_forward = history.can_go_forward();

    let counter = RwSignal::new(0_u32);
    let push = move |_| {
        counter.update(|n| *n += 1);
        history.push(format!("?step={}", counter.get_untracked()));
    };

    view! {
        <Section
            title="use_history"
            description="An undo/redo stack over URL query strings, driven with history.replaceState. Cmd/Ctrl+Z and Cmd/Ctrl+Shift+Z are also wired up."
        >
            <Demo col=true>
                <ButtonGroup>
                    <Button variant=ButtonVariant::Outline on:click=push>
                        <Icon icon=icondata::LuPlus attr:class="size-4" />
                        "Push state"
                    </Button>
                    <Button
                        variant=ButtonVariant::Outline
                        attr:disabled=move || !can_back.get()
                        on:click=move |_| {
                            history.go_back();
                        }
                    >
                        <Icon icon=icondata::LuUndo2 attr:class="size-4" />
                        "Undo"
                    </Button>
                    <Button
                        variant=ButtonVariant::Outline
                        attr:disabled=move || !can_forward.get()
                        on:click=move |_| {
                            history.go_forward();
                        }
                    >
                        <Icon icon=icondata::LuRedo2 attr:class="size-4" />
                        "Redo"
                    </Button>
                </ButtonGroup>
                <div class="flex flex-wrap gap-4">
                    <Readout label="position / total">
                        {move || format!("{} / {}", position.get(), total.get())}
                    </Readout>
                    <Readout label="current">
                        {move || {
                            let url = current.get();
                            if url.is_empty() { "(empty)".to_string() } else { url }
                        }}
                    </Readout>
                    <Readout label="can_go_back">{move || can_back.get().to_string()}</Readout>
                    <Readout label="can_go_forward">
                        {move || can_forward.get().to_string()}
                    </Readout>
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn LocksSection() -> impl IntoView {
    let locks = use_locks();

    let chips = [
        LockableParam::Theme,
        LockableParam::Font,
        LockableParam::Radius,
    ]
    .into_iter()
    .map(|param| {
        let is_locked = locks.is_locked(param);
        view! {
            <Button
                variant=ButtonVariant::Outline
                on:click=move |_| {
                    locks.toggle_lock(param);
                }
            >
                {move || {
                    view! {
                        <Icon
                            icon=if is_locked.get() {
                                icondata::LuLock
                            } else {
                                icondata::LuLockOpen
                            }
                            attr:class="size-4"
                        />
                    }
                }}
                {param.label()}
            </Button>
        }
    })
    .collect_view();

    let theme_locked = locks.is_locked(LockableParam::Theme);
    let font_can_randomize = locks.can_randomize(LockableParam::Font);

    view! {
        <Section
            title="use_locks"
            description="Tracks which design parameters are pinned against randomization. Toggle a param to flip its lock; the readouts expose is_locked and can_randomize."
        >
            <Demo col=true>
                <div class="flex flex-wrap gap-2">{chips}</div>
                <div class="flex flex-wrap gap-4">
                    <Readout label="Theme is_locked">
                        {move || theme_locked.get().to_string()}
                    </Readout>
                    <Readout label="Font can_randomize">
                        {move || font_can_randomize.get().to_string()}
                    </Readout>
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn ClipboardSection() -> impl IntoView {
    let (copy_default, copied_default) = use_copy_clipboard(None);
    let (copy_fast, copied_fast) = use_copy_clipboard(Some(Duration::from_millis(600)));

    view! {
        <Section
            title="use_copy_clipboard"
            description="Writes text to the clipboard and flips a `copied` flag that resets after the timeout (2s by default)."
        >
            <Demo label="Default 2s reset">
                <Button
                    variant=ButtonVariant::Outline
                    on:click=move |_| copy_default("ow_sk_live_8f2c9a")
                >
                    <Icon icon=icondata::LuClipboardCopy attr:class="size-4" />
                    {move || if copied_default.get() { "Copied!" } else { "Copy token" }}
                </Button>
            </Demo>
            <Demo label="Custom 600ms reset">
                <Button
                    variant=ButtonVariant::Outline
                    on:click=move |_| copy_fast("https://openworkspace.dev")
                >
                    <Icon icon=icondata::LuClipboardCopy attr:class="size-4" />
                    {move || if copied_fast.get() { "Copied!" } else { "Copy link" }}
                </Button>
            </Demo>
        </Section>
    }
}

#[component]
fn DataScrolledSection() -> impl IntoView {
    let scrolled = use_data_scrolled(120);

    let rows = (1..=40)
        .map(|n| {
            view! {
                <p class="border-b border-border/50 py-2 text-sm">
                    {format!("Row {n} — scroll this panel to cross the 120px threshold")}
                </p>
            }
        })
        .collect_view();

    view! {
        <Section
            title="use_data_scrolled"
            description="Watches the DATA_SCROLL_TARGET container (or the window) and flips a flag once it scrolls past a pixel threshold."
        >
            <Demo col=true>
                <Readout label="scrolled past 120px">{move || scrolled.get().to_string()}</Readout>
                <div
                    id=DATA_SCROLL_TARGET
                    class="h-48 overflow-y-auto rounded-md border border-border px-4"
                >
                    {rows}
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn RandomSection() -> impl IntoView {
    let id = use_random_id();
    let scoped_id = use_random_id_for("trigger");
    let transition = use_random_transition_name();

    view! {
        <Section
            title="use_random_id"
            description="Process-unique identifiers for element ids and CSS view-transition names. Stable for the life of the render."
        >
            <Demo col=true>
                <Readout label="use_random_id">{id}</Readout>
                <Readout label="use_random_id_for(\"trigger\")">{scoped_id}</Readout>
                <Readout label="use_random_transition_name">{transition}</Readout>
            </Demo>
        </Section>
    }
}

#[component]
fn VerticalScrollSection() -> impl IntoView {
    let (on_scroll, can_up, can_down) = use_can_scroll_vertical();

    let rows = (1..=30)
        .map(|n| {
            view! { <p class="border-b border-border/50 py-2 text-sm">{format!("Line {n}")}</p> }
        })
        .collect_view();

    view! {
        <Section
            title="use_can_scroll_vertical"
            description="Reports whether a scroll container can still scroll up or down, ideal for fading top/bottom edges."
        >
            <Demo col=true>
                <div class="flex flex-wrap gap-4">
                    <Readout label="can_scroll_up">{move || can_up.get().to_string()}</Readout>
                    <Readout label="can_scroll_down">{move || can_down.get().to_string()}</Readout>
                </div>
                <div
                    on:scroll=on_scroll
                    class="h-48 overflow-y-auto rounded-md border border-border px-4"
                >
                    {rows}
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn HorizontalScrollSection() -> impl IntoView {
    let node_ref = NodeRef::<Div>::new();
    let ctx = use_horizontal_scroll(node_ref, None, None);
    let scroll_state = ctx.scroll_state;
    let scroll_by = ctx.scroll_by;
    let on_scroll = ctx.on_scroll;

    let cards = (1..=12)
        .map(|n| {
            view! {
                <div class="flex h-24 w-40 shrink-0 items-center justify-center rounded-md border border-border bg-muted text-sm">
                    {format!("Card {n}")}
                </div>
            }
        })
        .collect_view();

    let at_start = move || scroll_state.get() == HorizontalScrollState::Start;
    let at_end = move || scroll_state.get() == HorizontalScrollState::End;

    view! {
        <Section
            title="use_horizontal_scroll"
            description="Drives a horizontally scrollable container: scroll_by(-1/1) jumps half a width, and scroll_state tracks Start / Middle / End."
        >
            <Demo col=true>
                <div class="flex items-center gap-4">
                    <ButtonGroup>
                        <Button
                            variant=ButtonVariant::Outline
                            attr:aria-label="Scroll left"
                            attr:disabled=at_start
                            on:click=move |_| {
                                scroll_by.run(-1);
                            }
                        >
                            <Icon icon=icondata::LuChevronLeft attr:class="size-4" />
                        </Button>
                        <Button
                            variant=ButtonVariant::Outline
                            attr:aria-label="Scroll right"
                            attr:disabled=at_end
                            on:click=move |_| {
                                scroll_by.run(1);
                            }
                        >
                            <Icon icon=icondata::LuChevronRight attr:class="size-4" />
                        </Button>
                    </ButtonGroup>
                    <Readout label="scroll_state">{move || scroll_state.get().to_string()}</Readout>
                </div>
                <div
                    node_ref=node_ref
                    on:scroll=move |ev| on_scroll.run(ev)
                    class="flex gap-4 overflow-x-auto rounded-md border border-border p-4"
                >
                    {cards}
                </div>
            </Demo>
        </Section>
    }
}
