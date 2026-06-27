use crate::{cn, use_random_id};
use leptos::prelude::*;
use leptos_icons::Icon;
use std::time::Duration;

/// Semantic kind of a toast — drives its accent colour and leading glyph.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ToastType {
    #[default]
    Default,
    Success,
    Error,
    Warning,
    Info,
    Loading,
}

impl ToastType {
    fn surface(self) -> &'static str {
        match self {
            ToastType::Default => "bg-background text-foreground border-border",
            ToastType::Success => "bg-success text-success-foreground border-transparent",
            ToastType::Error => {
                "bg-destructive text-white border-transparent dark:bg-destructive/60"
            }
            ToastType::Warning => "bg-warning text-warning-foreground border-transparent",
            ToastType::Info => "bg-info text-info-foreground border-transparent",
            ToastType::Loading => "bg-secondary text-secondary-foreground border-transparent",
        }
    }

    fn data_variant(self) -> &'static str {
        match self {
            ToastType::Default => "default",
            ToastType::Success => "success",
            ToastType::Error => "error",
            ToastType::Warning => "warning",
            ToastType::Info => "info",
            ToastType::Loading => "loading",
        }
    }
}

/// Corner the toast stack docks to. The vertical half also sets whether new
/// toasts enter from the top or the bottom of the column.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SonnerPosition {
    TopLeft,
    TopCenter,
    TopRight,
    #[default]
    BottomRight,
    BottomCenter,
    BottomLeft,
}

impl SonnerPosition {
    fn anchor(self) -> &'static str {
        match self {
            SonnerPosition::TopLeft => "top-6 left-6 items-start",
            SonnerPosition::TopRight => "top-6 right-6 items-end",
            SonnerPosition::TopCenter => "top-6 left-1/2 -translate-x-1/2 items-center",
            SonnerPosition::BottomLeft => "bottom-6 left-6 items-start",
            SonnerPosition::BottomRight => "bottom-6 right-6 items-end",
            SonnerPosition::BottomCenter => "bottom-6 left-1/2 -translate-x-1/2 items-center",
        }
    }

    fn is_top(self) -> bool {
        matches!(
            self,
            SonnerPosition::TopLeft | SonnerPosition::TopCenter | SonnerPosition::TopRight
        )
    }

    fn data_position(self) -> &'static str {
        match self {
            SonnerPosition::TopLeft => "top-left",
            SonnerPosition::TopCenter => "top-center",
            SonnerPosition::TopRight => "top-right",
            SonnerPosition::BottomRight => "bottom-right",
            SonnerPosition::BottomCenter => "bottom-center",
            SonnerPosition::BottomLeft => "bottom-left",
        }
    }
}

/// Default lifetime of a toast before it auto-dismisses.
const DEFAULT_DURATION: Duration = Duration::from_secs(4);

/// A single queued toast. `id` is process-unique so the stack can key on it and
/// target the right entry for dismissal.
#[derive(Clone, PartialEq, Eq)]
pub struct Toast {
    id: String,
    kind: ToastType,
    title: String,
    description: Option<String>,
    duration: Option<Duration>,
}

impl Toast {
    /// Builds a toast of `kind` with the given heading.
    #[must_use]
    pub fn new(kind: ToastType, title: impl Into<String>) -> Self {
        Self {
            id: use_random_id(),
            kind,
            title: title.into(),
            description: None,
            duration: Some(DEFAULT_DURATION),
        }
    }

    /// Adds supporting body copy under the title.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Overrides the auto-dismiss lifetime. Pass `None` to make the toast
    /// persist until it is dismissed explicitly (e.g. an in-flight `Loading`).
    #[must_use]
    pub fn with_duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }
}

/// Handle to the toast queue, provided by [`Toaster`] and retrieved with
/// [`use_toaster`]. Cloning is cheap — it shares one underlying queue.
#[derive(Clone, Copy)]
pub struct ToasterContext {
    queue: RwSignal<Vec<Toast>>,
}

impl ToasterContext {
    /// Pushes `toast` onto the stack, arming its auto-dismiss timer when it
    /// carries a finite duration. Returns the id so callers can dismiss a
    /// persistent toast later.
    pub fn show(self, toast: Toast) -> String {
        let id = toast.id.clone();
        let duration = toast.duration;
        self.queue.update(|toasts| toasts.push(toast));
        if let Some(duration) = duration {
            let id = id.clone();
            // `set_timeout` schedules on the browser event loop without parking a
            // `Closure` in the reactive graph, and no-ops on the server.
            set_timeout(move || self.dismiss(&id), duration);
        }
        id
    }

    /// Removes the toast with `id` from the stack, if still present.
    pub fn dismiss(self, id: &str) {
        self.queue
            .try_update(|toasts| toasts.retain(|toast| toast.id != id));
    }

    /// Clears every toast at once.
    pub fn clear(self) {
        self.queue.try_update(Vec::clear);
    }
}

/// Returns the ambient [`ToasterContext`].
///
/// # Panics
///
/// Panics if no [`Toaster`] is mounted above the caller — mount one once near
/// the app root.
#[must_use]
pub fn use_toaster() -> ToasterContext {
    expect_context::<ToasterContext>()
}

const TOAST_BASE: &str = "pointer-events-auto flex w-80 items-start gap-3 rounded-lg border px-4 py-3 text-sm shadow-lg [&_svg:not([class*='size-'])]:size-4 [&_svg]:mt-0.5 [&_svg]:shrink-0";

fn toast_icon(kind: ToastType) -> impl IntoView {
    match kind {
        ToastType::Default => {
            view! { <Icon icon=icondata::LuInfo attr:class="size-4 opacity-70" /> }.into_any()
        }
        ToastType::Success => {
            view! { <Icon icon=icondata::LuCircleCheck attr:class="size-4" /> }.into_any()
        }
        ToastType::Error => {
            view! { <Icon icon=icondata::LuCircleAlert attr:class="size-4" /> }.into_any()
        }
        ToastType::Warning => {
            view! { <Icon icon=icondata::LuTriangleAlert attr:class="size-4" /> }.into_any()
        }
        ToastType::Info => view! { <Icon icon=icondata::LuInfo attr:class="size-4" /> }.into_any(),
        ToastType::Loading => {
            view! { <Icon icon=icondata::LuLoaderCircle attr:class="size-4 animate-spin" /> }
                .into_any()
        }
    }
}

/// Toast stack root. Provides a [`ToasterContext`] to descendants and renders
/// the fixed-position column of live toasts. Mount it once near the top of the
/// tree, then raise toasts from anywhere below with [`use_toaster`].
///
/// `position` docks the stack to a screen corner and sets the enter edge.
/// Native attributes, events and bindings forward to the root container.
#[component]
pub fn Toaster(
    #[prop(into, optional)] position: Signal<SonnerPosition>,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let ctx = ToasterContext {
        queue: RwSignal::new(Vec::new()),
    };
    provide_context(ctx);

    let merged = move || {
        cn!(
            "pointer-events-none fixed z-50 flex max-h-screen w-fit flex-col gap-3 p-0",
            position.get().anchor(),
            class.get()
        )
    };
    let entries = move || {
        let mut toasts = ctx.queue.get();
        if position.get().is_top() {
            toasts.reverse();
        }
        toasts
    };

    view! {
        <div
            data-name="Toaster"
            role="region"
            aria-label="Notifications"
            aria-live="polite"
            data-position=move || position.get().data_position()
            class=merged
        >
            <For each=entries key=|toast| toast.id.clone() let:toast>
                {
                    let kind = toast.kind;
                    let id = toast.id.clone();
                    let description = toast.description.clone();
                    view! {
                        <div
                            data-name="Toast"
                            role="status"
                            data-variant=kind.data_variant()
                            class=cn!(TOAST_BASE, kind.surface())
                        >
                            {toast_icon(kind)}
                            <div class="flex flex-1 flex-col gap-0.5">
                                <div class="font-medium">{toast.title.clone()}</div>
                                {description
                                    .map(|text| view! { <div class="opacity-90">{text}</div> })}
                            </div>
                            <button
                                type="button"
                                aria-label="Dismiss"
                                class="opacity-60 transition-opacity hover:opacity-100 focus-visible:opacity-100 focus-visible:outline-none"
                                on:click=move |_| ctx.dismiss(&id)
                            >
                                <Icon icon=icondata::LuX attr:class="size-4" />
                            </button>
                        </div>
                    }
                }
            </For>
        </div>
    }
}

const TRIGGER_BASE: &str = "inline-flex w-fit items-center justify-center gap-2 whitespace-nowrap rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs transition-all outline-none hover:bg-primary/90 hover:cursor-pointer active:scale-[0.98] focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50";

/// Button that raises a toast on click. `title`/`description`/`variant`
/// describe the toast it queues into the ambient [`Toaster`]. Native
/// attributes, events and bindings forward to the underlying `<button>`.
#[component]
pub fn SonnerTrigger(
    #[prop(into)] title: Signal<String>,
    #[prop(into, optional)] description: Signal<String>,
    #[prop(into, optional)] variant: Signal<ToastType>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let ctx = use_toaster();
    let raise = move |_| {
        let mut toast = Toast::new(variant.get(), title.get());
        let description = description.get();
        if !description.is_empty() {
            toast = toast.with_description(description);
        }
        _ = ctx.show(toast);
    };

    view! {
        <button
            type="button"
            data-name="SonnerTrigger"
            class=move || cn!(TRIGGER_BASE, class.get())
            on:click=raise
        >
            {children()}
        </button>
    }
}
