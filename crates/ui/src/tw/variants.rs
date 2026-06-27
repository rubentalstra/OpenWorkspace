//! `variants!` — generates `{Name}Variant`/`{Name}Size` enums (the first arm of
//! each axis is its `Default`) and, when a `component` block is present, a Leptos
//! component whose reactive class is the base merged (via `cn!`) with the selected
//! variant/size classes and the caller's `class`. First-party: no third-party
//! derive macros, no `use` globs, so any number of invocations can share a module.

/// Generates a unit enum (first arm = `Default`) plus a private `class()` mapping
/// each arm to its Tailwind classes. Internal to [`variants!`].
#[macro_export]
#[doc(hidden)]
macro_rules! __variants_enum {
    ($name:ident $suffix:ident, $first:ident: $first_class:literal $(, $key:ident: $class:literal)* $(,)?) => {
        $crate::paste::paste! {
            #[derive(Clone, Copy, PartialEq, Eq, Default)]
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
/// - `variant` + `size` with a `component { element [, support_href [, support_aria_current]] }` block
/// - `variant` + `size`, or `variant` alone — enums only (no component)
#[macro_export]
macro_rules! variants {
    // variant + size + component rendered as <a> (with aria-current) when href is set
    (
        $(#[$meta:meta])*
        $name:ident {
            base: $base:literal,
            variants: {
                variant: { $v0:ident: $v0c:literal $(, $vk:ident: $vc:literal)* $(,)? },
                size: { $s0:ident: $s0c:literal $(, $sk:ident: $sc:literal)* $(,)? }
            },
            component: { element: $el:ident, support_href: true, support_aria_current: true }
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
                children: Children,
            ) -> impl IntoView {
                let computed =
                    move || $crate::cn!($base, variant.get().class(), size.get().class(), class.get());
                match href {
                    Some(href) => {
                        let location = leptos_router::hooks::use_location();
                        let target = href.clone();
                        let aria_current = move || {
                            let path = location.pathname.get();
                            if path == target || path.starts_with(&format!("{target}/")) {
                                "page"
                            } else {
                                "false"
                            }
                        };
                        view! {
                            <a
                                class=computed
                                href=href
                                aria-current=aria_current
                                data-name=stringify!($name)
                            >
                                {children()}
                            </a>
                        }
                        .into_any()
                    }
                    None => view! {
                        <$el class=computed data-name=stringify!($name)>
                            {children()}
                        </$el>
                    }
                    .into_any(),
                }
            }
        }
    };

    // variant + size + component rendered as <a> when href is set
    (
        $(#[$meta:meta])*
        $name:ident {
            base: $base:literal,
            variants: {
                variant: { $v0:ident: $v0c:literal $(, $vk:ident: $vc:literal)* $(,)? },
                size: { $s0:ident: $s0c:literal $(, $sk:ident: $sc:literal)* $(,)? }
            },
            component: { element: $el:ident, support_href: true }
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
                children: Children,
            ) -> impl IntoView {
                let computed =
                    move || $crate::cn!($base, variant.get().class(), size.get().class(), class.get());
                match href {
                    Some(href) => view! {
                        <a class=computed href=href data-name=stringify!($name)>
                            {children()}
                        </a>
                    }
                    .into_any(),
                    None => view! {
                        <$el class=computed data-name=stringify!($name)>
                            {children()}
                        </$el>
                    }
                    .into_any(),
                }
            }
        }
    };

    // variant + size + component
    (
        $(#[$meta:meta])*
        $name:ident {
            base: $base:literal,
            variants: {
                variant: { $v0:ident: $v0c:literal $(, $vk:ident: $vc:literal)* $(,)? },
                size: { $s0:ident: $s0c:literal $(, $sk:ident: $sc:literal)* $(,)? }
            },
            component: { element: $el:ident }
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
                children: Children,
            ) -> impl IntoView {
                let computed =
                    move || $crate::cn!($base, variant.get().class(), size.get().class(), class.get());
                view! {
                    <$el class=computed data-name=stringify!($name)>
                        {children()}
                    </$el>
                }
            }
        }
    };

    // variant + size enums only
    (
        $name:ident {
            variants: {
                variant: { $v0:ident: $v0c:literal $(, $vk:ident: $vc:literal)* $(,)? },
                size: { $s0:ident: $s0c:literal $(, $sk:ident: $sc:literal)* $(,)? }
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
