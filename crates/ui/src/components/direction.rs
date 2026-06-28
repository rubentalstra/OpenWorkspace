use crate::cn;
use leptos::prelude::*;

/// Reading direction of text — the Base UI `TextDirection` (`"ltr" | "rtl"`).
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Direction {
    /// Left-to-right (the default).
    #[default]
    Ltr,
    /// Right-to-left.
    Rtl,
}

impl Direction {
    /// The `dir` attribute / `data-direction` value (`"ltr"` or `"rtl"`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}

#[derive(Clone, Copy)]
struct DirectionCtx {
    direction: Signal<Direction>,
}

/// DirectionProvider — shadcn Base UI `direction-provider`. Enables RTL behaviour
/// for descendant components by providing a [`Direction`] context (read with
/// [`use_direction`]). Defaults to [`Direction::Ltr`].
///
/// Base UI's provider sets context only; this port additionally renders a
/// `display: contents` wrapper carrying the `dir` attribute so plain CSS RTL also
/// applies without disturbing layout. The wrapper is reactive — updating the
/// `direction` signal re-renders `dir`/`data-direction` and the provided context.
#[component]
pub fn DirectionProvider(
    #[prop(into, optional)] direction: Signal<Direction>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    provide_context(DirectionCtx { direction });
    view! {
        <div
            data-slot="direction-provider"
            data-direction=move || direction.get().as_str()
            dir=move || direction.get().as_str()
            class=move || cn!("contents", class.get())
        >
            {children()}
        </div>
    }
}

/// Read the current text [`Direction`] from the nearest [`DirectionProvider`].
///
/// Mirrors Base UI's `useDirection`. Returns a reactive [`Signal`]; when no
/// provider is present it resolves to [`Direction::Ltr`].
#[must_use]
pub fn use_direction() -> Signal<Direction> {
    match use_context::<DirectionCtx>() {
        Some(ctx) => ctx.direction,
        None => Signal::derive(Direction::default),
    }
}
