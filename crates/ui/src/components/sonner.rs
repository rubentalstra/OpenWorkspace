use crate::cn;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use leptos::prelude::*;
use leptos_icons::Icon;

/// Toast tone, mirroring sonner's built-in types. Each maps to a leading icon and
/// is surfaced as `data-type` for the nova layer to style.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ToastVariant {
    /// Neutral, iconless toast.
    #[default]
    Default,
    /// Success toast (check-in-circle icon).
    Success,
    /// Informational toast (info icon).
    Info,
    /// Warning toast (triangle-alert icon).
    Warning,
    /// Error toast (octagon-x icon).
    Error,
    /// Loading toast (spinning loader icon).
    Loading,
}

impl ToastVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Success => "success",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Loading => "loading",
        }
    }
}

/// A single queued toast.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Toast {
    /// Stable identifier used as the list key and for dismissal.
    pub id: u64,
    /// Tone driving the leading icon and `data-type`.
    pub variant: ToastVariant,
    /// Primary message line.
    pub title: String,
    /// Optional secondary line.
    pub description: Option<String>,
}

/// Shared toast queue. Provide once near the app root via [`provide_toaster`], read
/// via [`expect_context`], and mutate through [`ToasterContext::push`] /
/// [`ToasterContext::dismiss`].
#[derive(Clone, Copy)]
pub struct ToasterContext {
    /// The live toast queue. The [`Toaster`] component renders this reactively.
    pub toasts: RwSignal<Vec<Toast>>,
    next_id: RwSignal<u64>,
}

impl ToasterContext {
    /// Create an empty toast queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            toasts: RwSignal::new(Vec::new()),
            next_id: RwSignal::new(0),
        }
    }

    /// Append a toast, assigning it a fresh id; returns that id.
    pub fn push(&self, variant: ToastVariant, title: impl Into<String>) -> u64 {
        self.push_full(variant, title.into(), None)
    }

    /// Append a toast with a description; returns the assigned id.
    pub fn push_with_description(
        &self,
        variant: ToastVariant,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> u64 {
        self.push_full(variant, title.into(), Some(description.into()))
    }

    fn push_full(&self, variant: ToastVariant, title: String, description: Option<String>) -> u64 {
        let id = self.next_id.get_untracked();
        self.next_id.set(id + 1);
        self.toasts.update(|list| {
            list.push(Toast {
                id,
                variant,
                title,
                description,
            });
        });
        id
    }

    /// Remove the toast with the given id, if present.
    pub fn dismiss(&self, id: u64) {
        self.toasts
            .update(|list| list.retain(|toast| toast.id != id));
    }

    /// Remove every queued toast.
    pub fn clear(&self) {
        self.toasts.update(Vec::clear);
    }
}

impl Default for ToasterContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a [`ToasterContext`], provide it to descendants, and return it so the
/// caller can push toasts. Call once near the app root, alongside a [`Toaster`].
pub fn provide_toaster() -> ToasterContext {
    let ctx = ToasterContext::new();
    provide_context(ctx);
    ctx
}

/// Toaster — pure-Leptos reimplementation of the sonner toaster. Renders the
/// queue from [`ToasterContext`] as stacked cards fixed to the bottom-right. The
/// `cn-toast` hook from nova styles each card. Place once near the app root.
#[component]
pub fn Toaster(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    let ctx = expect_context::<ToasterContext>();
    view! {
        <ol
            data-slot="toaster"
            class=move || {
                cn!(
                    "toaster group pointer-events-none fixed right-4 bottom-4 z-100 flex w-full max-w-[356px] flex-col gap-2",
                    class.get(),
                )
            }
        >
            <For each=move || ctx.toasts.get() key=|toast| toast.id let:toast>
                <ToastItem toast=toast />
            </For>
        </ol>
    }
}

/// A single rendered toast card.
#[component]
fn ToastItem(toast: Toast) -> impl IntoView {
    let ctx = expect_context::<ToasterContext>();
    let id = toast.id;
    let variant = toast.variant;
    let description = toast.description.clone();
    view! {
        <li
            data-slot="toast"
            data-type=variant.as_str()
            data-open="true"
            role="status"
            aria-live="polite"
            class="cn-toast group pointer-events-auto bg-popover text-popover-foreground border-border flex items-start gap-2 border p-4 shadow-lg"
        >
            {toast_icon(variant)}
            <div data-slot="toast-content" class="flex grow flex-col gap-1">
                <div data-slot="toast-title" class="text-sm font-medium">
                    {toast.title}
                </div>
                {description
                    .map(|description| {
                        view! {
                            <div
                                data-slot="toast-description"
                                class="text-muted-foreground text-sm"
                            >
                                {description}
                            </div>
                        }
                    })}
            </div>
            <Button
                variant=ButtonVariant::Ghost
                size=ButtonSize::IconSm
                class="shrink-0"
                attr:r#type="button"
                attr:data-slot="toast-close"
                attr:aria-label="Close"
                on:click=move |_| ctx.dismiss(id)
            >
                <Icon icon=icondata::LuX attr:class="size-4" />
            </Button>
        </li>
    }
}

/// The leading icon for a toast tone, or nothing for [`ToastVariant::Default`].
fn toast_icon(variant: ToastVariant) -> impl IntoView {
    let icon = match variant {
        ToastVariant::Default => return None,
        ToastVariant::Success => icondata::LuCircleCheck,
        ToastVariant::Info => icondata::LuInfo,
        ToastVariant::Warning => icondata::LuTriangleAlert,
        ToastVariant::Error => icondata::LuOctagonX,
        ToastVariant::Loading => icondata::LuLoaderCircle,
    };
    let spin = matches!(variant, ToastVariant::Loading);
    Some(view! {
        <span data-slot="toast-icon" class="shrink-0 pt-0.5">
            <Icon icon=icon attr:class=if spin { "size-4 animate-spin" } else { "size-4" } />
        </span>
    })
}
