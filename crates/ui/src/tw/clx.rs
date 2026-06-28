/// Defines a Tailwind-styled wrapper component whose base classes merge with a
/// reactive `class` prop (later utilities win). Native attributes forward to the
/// root element. Leading `///` docs / attributes are applied to the component.
///
/// ```ignore
/// clx! {
///     /// Surface header.
///     CardHeader, div, "flex flex-col gap-1.5 px-6"
/// }
/// ```
#[macro_export]
macro_rules! clx {
    ($(#[$meta:meta])* $name:ident, $element:ident, $($base_class:expr),+ $(,)?) => {
        $(#[$meta])*
        #[component]
        pub fn $name(
            #[prop(into, optional)] class: Signal<String>,
            #[prop(optional)] children: Option<Children>,
        ) -> impl IntoView {
            view! {
                <$element
                    class=move || $crate::cn!($($base_class),+, class.get())
                    data-name=stringify!($name)
                >
                    {children.map(|children| children())}
                </$element>
            }
        }
    };
}

/// Like [`clx!`] but for [void elements](https://developer.mozilla.org/en-US/docs/Glossary/Void_element)
/// that take no children, e.g. `input`, `img`, `hr`.
#[macro_export]
macro_rules! void {
    ($(#[$meta:meta])* $name:ident, $element:ident, $($base_class:expr),+ $(,)?) => {
        $(#[$meta])*
        #[component]
        pub fn $name(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
            view! {
                <$element
                    class=move || $crate::cn!($($base_class),+, class.get())
                    data-name=stringify!($name)
                />
            }
        }
    };
}
