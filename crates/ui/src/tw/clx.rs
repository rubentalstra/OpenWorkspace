/// Defines a Tailwind-styled wrapper component whose base classes merge with a
/// reactive `class` prop (later utilities win). Native attributes forward to the
/// root element.
///
/// ```ignore
/// clx! {Card, div, "rounded-lg p-4", "bg-card"}
/// // <Card class="p-6"/> renders <div class="rounded-lg bg-card p-6" data-name="Card">
/// ```
#[macro_export]
macro_rules! clx {
    ($name:ident, $element:ident, $($base_class:expr),+ $(,)?) => {
        #[component]
        pub fn $name(
            #[prop(into, optional)] class: Signal<String>,
            children: Children,
        ) -> impl IntoView {
            view! {
                <$element
                    class=move || $crate::cn!($($base_class),+, class.get())
                    data-name=stringify!($name)
                >
                    {children()}
                </$element>
            }
        }
    };
}

/// Like [`clx!`] but for [void elements](https://developer.mozilla.org/en-US/docs/Glossary/Void_element)
/// that take no children, e.g. `input`, `img`, `hr`.
#[macro_export]
macro_rules! void {
    ($name:ident, $element:ident, $($base_class:expr),+ $(,)?) => {
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
