use crate::cn;
use leptos::context::Provider;
use leptos::html;
use leptos::prelude::*;

/// Reading direction shared with descendants via context and reflected on the
/// root element's `dir` attribute.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Direction {
    /// Left-to-right (default).
    #[default]
    Ltr,
    /// Right-to-left.
    Rtl,
}

impl Direction {
    fn as_attr(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}

/// Establishes the reading direction for its subtree. The chosen [`Direction`]
/// is set on the root's `dir` attribute and exposed to descendants through
/// context (read it with [`use_direction`]). Native attributes, events and
/// bindings forward to the root element.
#[component]
pub fn DirectionProvider(
    #[prop(into, optional)] dir: Signal<Direction>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    children: Children,
) -> impl IntoView {
    let attr = move || dir.get().as_attr();
    let merged = move || cn!(class.get());

    view! {
        <Provider value=dir>
            <div node_ref=node_ref data-name="DirectionProvider" dir=attr class=merged>
                {children()}
            </div>
        </Provider>
    }
}

/// Reads the ambient reading [`Direction`] provided by the nearest
/// [`DirectionProvider`], defaulting to [`Direction::Ltr`] when none is present.
#[must_use]
pub fn use_direction() -> Signal<Direction> {
    use_context::<Signal<Direction>>().unwrap_or_else(|| Signal::derive(Direction::default))
}
