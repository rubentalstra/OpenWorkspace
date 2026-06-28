//! `variants!` — the Rust analogue of shadcn's `cva()`. Generates a `#[default]`
//! enum per variant axis (`{Name}Variant`, optionally `{Name}Size`) whose private
//! `class()` returns the **semantic** Base-UI class for that arm (`cn-button-…`),
//! plus a reactive `#[component]` that renders `<el data-slot=…>` (or `<a>` when a
//! `href` is set) with `cn!(base, variant, size, class)`.
//!
//! First-party only: no third-party derive macros and no `use` globs, so any number
//! of invocations coexist in one module.

/// Generate a unit enum (first arm = `Default`) and a private `class()` mapping each
/// arm to its semantic class. Internal to [`variants!`].
#[macro_export]
#[doc(hidden)]
macro_rules! __variants_enum {
    ($name:ident $suffix:ident, $first:ident: $first_class:literal $(, $key:ident: $class:literal)* $(,)?) => {
        $crate::paste::paste! {
            #[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
            pub enum [<$name $suffix>] {
                #[default]
                $first,
                $($key,)*
            }

            impl [<$name $suffix>] {
                fn class(self) -> &'static str {
                    match self {
                        Self::$first => $first_class,
                        $(Self::$key => $class,)*
                    }
                }
            }
        }
    };
}

/// See the module docs. Supported shapes:
/// - `variant` + `size` with a `component { element }` block
/// - `variant` alone with a `component { element }` block
/// - `variant` + `size`, or `variant` alone — enums only (no component)
///
/// Components always expose an optional `href`: when set the root renders as `<a>`,
/// otherwise as the given element. `data-slot` is always emitted.
#[macro_export]
macro_rules! variants {
    // variant + size + component
    (
        $(#[$meta:meta])*
        $name:ident {
            slot: $slot:literal,
            base: $base:literal,
            variants: {
                variant: { $v0:ident: $v0c:literal $(, $vk:ident: $vc:literal)* $(,)? },
                size: { $s0:ident: $s0c:literal $(, $sk:ident: $sc:literal)* $(,)? } $(,)?
            },
            component: { element: $el:ident $(,)? } $(,)?
        }
    ) => {
        $crate::__variants_enum!($name Variant, $v0: $v0c $(, $vk: $vc)*);
        $crate::__variants_enum!($name Size, $s0: $s0c $(, $sk: $sc)*);
        $crate::paste::paste! {
            $(#[$meta])*
            #[component]
            pub fn $name(
                #[prop(into, optional)] variant: Signal<[<$name Variant>]>,
                #[prop(into, optional)] size: Signal<[<$name Size>]>,
                #[prop(into, optional)] class: Signal<String>,
                #[prop(into, optional)] href: Option<String>,
                #[prop(optional)] children: Option<Children>,
            ) -> impl IntoView {
                let computed = move || {
                    $crate::cn!($base, variant.get().class(), size.get().class(), class.get())
                };
                match href {
                    Some(href) => view! {
                        <a data-slot=$slot href=href class=computed>
                            {children.map(|children| children())}
                        </a>
                    }
                    .into_any(),
                    None => view! {
                        <$el data-slot=$slot class=computed>
                            {children.map(|children| children())}
                        </$el>
                    }
                    .into_any(),
                }
            }
        }
    };

    // variant + component
    (
        $(#[$meta:meta])*
        $name:ident {
            slot: $slot:literal,
            base: $base:literal,
            variants: {
                variant: { $v0:ident: $v0c:literal $(, $vk:ident: $vc:literal)* $(,)? } $(,)?
            },
            component: { element: $el:ident $(,)? } $(,)?
        }
    ) => {
        $crate::__variants_enum!($name Variant, $v0: $v0c $(, $vk: $vc)*);
        $crate::paste::paste! {
            $(#[$meta])*
            #[component]
            pub fn $name(
                #[prop(into, optional)] variant: Signal<[<$name Variant>]>,
                #[prop(into, optional)] class: Signal<String>,
                #[prop(into, optional)] href: Option<String>,
                #[prop(optional)] children: Option<Children>,
            ) -> impl IntoView {
                let computed = move || $crate::cn!($base, variant.get().class(), class.get());
                match href {
                    Some(href) => view! {
                        <a data-slot=$slot href=href class=computed>
                            {children.map(|children| children())}
                        </a>
                    }
                    .into_any(),
                    None => view! {
                        <$el data-slot=$slot class=computed>
                            {children.map(|children| children())}
                        </$el>
                    }
                    .into_any(),
                }
            }
        }
    };

    // variant + size enums only
    (
        $name:ident {
            variants: {
                variant: { $v0:ident: $v0c:literal $(, $vk:ident: $vc:literal)* $(,)? },
                size: { $s0:ident: $s0c:literal $(, $sk:ident: $sc:literal)* $(,)? } $(,)?
            }
        }
    ) => {
        $crate::__variants_enum!($name Variant, $v0: $v0c $(, $vk: $vc)*);
        $crate::__variants_enum!($name Size, $s0: $s0c $(, $sk: $sc)*);
    };

    // variant enum only
    (
        $name:ident {
            variants: { variant: { $v0:ident: $v0c:literal $(, $vk:ident: $vc:literal)* $(,)? } }
        }
    ) => {
        $crate::__variants_enum!($name Variant, $v0: $v0c $(, $vk: $vc)*);
    };
}
