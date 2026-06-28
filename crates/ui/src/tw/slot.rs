//! `slot!` — generates a thin Base-UI wrapper component: one root element carrying
//! `data-slot` plus the `cn-*` base classes merged (via `cn!`) with the caller's
//! reactive `class`. Extra attributes, event listeners, and `bind:`s placed on the
//! component at the call site are spread onto this root element by Leptos.
//!
//! Use for the many structural wrappers (card parts, list containers, labels, …).
//! Void elements that need a typed `node_ref` (`Input`, `Textarea`) are written as
//! explicit `#[component]`s instead.

/// Generate `#[component] fn $name` rendering `<$el data-slot=$slot class=…>` with
/// optional `children`. `$base` is one or more class fragments (string literals or
/// expressions) merged ahead of the caller's `class`.
#[macro_export]
macro_rules! slot {
    ($(#[$meta:meta])* $name:ident, $el:ident, $slot:literal, $($base:expr),+ $(,)?) => {
        $(#[$meta])*
        #[component]
        pub fn $name(
            #[prop(into, optional)] class: Signal<String>,
            #[prop(optional)] children: Option<Children>,
        ) -> impl IntoView {
            view! {
                <$el data-slot=$slot class=move || $crate::cn!($($base),+, class.get())>
                    {children.map(|children| children())}
                </$el>
            }
        }
    };
}
