/// Generates a `tw_merge`-backed class type (and optionally a Leptos component)
/// from a base class plus `variant`/`size` axes. The first arm of each axis is
/// its `Default`. Six shapes are supported: variant+size, variant-only, and
/// size-only (type-only); plus variant+size with a generated component element,
/// optionally rendering as an `<a>` when given an `href` (with `aria-current`).
///
/// ```ignore
/// variants! {
///     Badge {
///         base: "inline-flex items-center rounded-md border w-fit",
///         variants: {
///             variant: {
///                 Default: "bg-primary text-primary-foreground",
///                 Outline: "text-foreground border-border",
///             },
///             size: { Default: "px-2.5 py-0.5 text-xs", Lg: "px-3 py-1 text-sm" }
///         }
///     }
/// }
/// // => BadgeClass, BadgeVariant { Default, Outline }, BadgeSize { Default, Lg }
/// ```
///
/// Add a `component: { element: span }` block to also generate a `Badge`
/// component; `support_href: true` (and `support_aria_current: true`) make it
/// render as a link when an `href` prop is supplied.
#[macro_export]
macro_rules! variants {
    // variant + size + generated component element
    (
        $component:ident {
            base: $base_class:literal,
            variants: {
                variant: {
                    $first_variant:ident: $first_variant_class:literal
                    $(, $variant_key:ident: $variant_class:literal)* $(,)?
                },
                size: {
                    $first_size:ident: $first_size_class:literal
                    $(, $size_key:ident: $size_class:literal)* $(,)?
                }
            },
            component: {
                element: $element:ident
            }
        }
    ) => {
        $crate::paste::paste! {
            use $crate::tw_merge::*;

            #[derive(TwClass, Clone, Copy)]
            #[tw(class = $base_class)]
            pub struct [<$component Class>] {
                pub variant: [<$component Variant>],
                pub size: [<$component Size>],
            }

            #[derive(TwVariant)]
            pub enum [<$component Variant>] {
                #[tw(default, class = $first_variant_class)]
                $first_variant,
                $(
                    #[tw(class = $variant_class)]
                    $variant_key,
                )*
            }

            #[derive(TwVariant)]
            pub enum [<$component Size>] {
                #[tw(default, class = $first_size_class)]
                $first_size,
                $(
                    #[tw(class = $size_class)]
                    $size_key,
                )*
            }

            #[::leptos::component]
            pub fn $component(
                #[prop(into, optional)] variant: ::leptos::prelude::Signal<[<$component Variant>]>,
                #[prop(into, optional)] size: ::leptos::prelude::Signal<[<$component Size>]>,
                #[prop(into, optional)] class: ::leptos::prelude::Signal<String>,
                #[prop(into, optional)] data_name: Option<String>,
                children: ::leptos::prelude::Children,
            ) -> impl ::leptos::prelude::IntoView {
                use ::leptos::prelude::*;

                let computed_class = move || {
                    let variant = variant.try_get().unwrap_or_default();
                    let size = size.try_get().unwrap_or_default();
                    let component_class = [<$component Class>] { variant, size };
                    component_class.with_class(class.try_get().unwrap_or_default())
                };

                let data_name = data_name.unwrap_or_else(|| stringify!($component).to_string());

                view! {
                    <$element class=computed_class data-name=data_name>
                        {children()}
                    </$element>
                }
            }
        }
    };

    // generated component that renders as <a> with aria-current when href is set
    (
        $component:ident {
            base: $base_class:literal,
            variants: {
                variant: {
                    $first_variant:ident: $first_variant_class:literal
                    $(, $variant_key:ident: $variant_class:literal)* $(,)?
                },
                size: {
                    $first_size:ident: $first_size_class:literal
                    $(, $size_key:ident: $size_class:literal)* $(,)?
                }
            },
            component: {
                element: $element:ident,
                support_href: true,
                support_aria_current: true
            }
        }
    ) => {
        $crate::paste::paste! {
            use $crate::tw_merge::*;

            #[derive(TwClass, Clone, Copy)]
            #[tw(class = $base_class)]
            pub struct [<$component Class>] {
                pub variant: [<$component Variant>],
                pub size: [<$component Size>],
            }

            #[derive(TwVariant)]
            pub enum [<$component Variant>] {
                #[tw(default, class = $first_variant_class)]
                $first_variant,
                $(#[tw(class = $variant_class)] $variant_key,)*
            }

            #[derive(TwVariant)]
            pub enum [<$component Size>] {
                #[tw(default, class = $first_size_class)]
                $first_size,
                $(#[tw(class = $size_class)] $size_key,)*
            }

            #[::leptos::component]
            pub fn $component(
                #[prop(into, optional)] variant: ::leptos::prelude::Signal<[<$component Variant>]>,
                #[prop(into, optional)] size: ::leptos::prelude::Signal<[<$component Size>]>,
                #[prop(into, optional)] class: ::leptos::prelude::Signal<String>,
                #[prop(into, optional)] data_name: Option<String>,
                #[prop(into, optional)] href: Option<String>,
                children: ::leptos::prelude::Children,
            ) -> impl ::leptos::prelude::IntoView {
                use ::leptos::prelude::*;

                let computed_class = move || {
                    let variant = variant.try_get().unwrap_or_default();
                    let size = size.try_get().unwrap_or_default();
                    let component_class = [<$component Class>] { variant, size };
                    component_class.with_class(class.try_get().unwrap_or_default())
                };

                let data_name = data_name.unwrap_or_else(|| stringify!($component).to_string());

                match href {
                    Some(href) => {
                        use ::leptos_router::hooks::use_location;

                        let location = use_location();
                        let is_active = {
                            let href = href.clone();
                            move || {
                                let path = location.pathname.try_get().unwrap_or_default();
                                path == href || path.starts_with(&format!("{href}/"))
                            }
                        };
                        let aria_current = move || if is_active() { "page" } else { "false" };

                        view! {
                            <a
                                class=computed_class
                                href=href
                                aria-current=aria_current
                                data-name=data_name
                            >
                                {children()}
                            </a>
                        }
                        .into_any()
                    }
                    None => view! {
                        <$element class=computed_class data-name=data_name>
                            {children()}
                        </$element>
                    }
                    .into_any(),
                }
            }
        }
    };

    // generated component that renders as <a> when href is set
    (
        $component:ident {
            base: $base_class:literal,
            variants: {
                variant: {
                    $first_variant:ident: $first_variant_class:literal
                    $(, $variant_key:ident: $variant_class:literal)* $(,)?
                },
                size: {
                    $first_size:ident: $first_size_class:literal
                    $(, $size_key:ident: $size_class:literal)* $(,)?
                }
            },
            component: {
                element: $element:ident,
                support_href: true
            }
        }
    ) => {
        $crate::paste::paste! {
            use $crate::tw_merge::*;

            #[derive(TwClass, Clone, Copy)]
            #[tw(class = $base_class)]
            pub struct [<$component Class>] {
                pub variant: [<$component Variant>],
                pub size: [<$component Size>],
            }

            #[derive(TwVariant)]
            pub enum [<$component Variant>] {
                #[tw(default, class = $first_variant_class)]
                $first_variant,
                $(#[tw(class = $variant_class)] $variant_key,)*
            }

            #[derive(TwVariant)]
            pub enum [<$component Size>] {
                #[tw(default, class = $first_size_class)]
                $first_size,
                $(#[tw(class = $size_class)] $size_key,)*
            }

            #[::leptos::component]
            pub fn $component(
                #[prop(into, optional)] variant: ::leptos::prelude::Signal<[<$component Variant>]>,
                #[prop(into, optional)] size: ::leptos::prelude::Signal<[<$component Size>]>,
                #[prop(into, optional)] class: ::leptos::prelude::Signal<String>,
                #[prop(into, optional)] data_name: Option<String>,
                #[prop(into, optional)] href: Option<String>,
                children: ::leptos::prelude::Children,
            ) -> impl ::leptos::prelude::IntoView {
                use ::leptos::prelude::*;

                let computed_class = move || {
                    let variant = variant.try_get().unwrap_or_default();
                    let size = size.try_get().unwrap_or_default();
                    let component_class = [<$component Class>] { variant, size };
                    component_class.with_class(class.try_get().unwrap_or_default())
                };

                let data_name = data_name.unwrap_or_else(|| stringify!($component).to_string());

                match href {
                    Some(href) => view! {
                        <a class=computed_class href=href data-name=data_name>
                            {children()}
                        </a>
                    }
                    .into_any(),
                    None => view! {
                        <$element class=computed_class data-name=data_name>
                            {children()}
                        </$element>
                    }
                    .into_any(),
                }
            }
        }
    };

    // variant + size, type only (no component)
    (
        $component:ident {
            base: $base_class:literal,
            variants: {
                variant: {
                    $first_variant:ident: $first_variant_class:literal
                    $(, $variant_key:ident: $variant_class:literal)* $(,)?
                },
                size: {
                    $first_size:ident: $first_size_class:literal
                    $(, $size_key:ident: $size_class:literal)* $(,)?
                }
            }
        }
    ) => {
        $crate::paste::paste! {
            use $crate::tw_merge::*;

            #[derive(TwClass, Clone, Copy)]
            #[tw(class = $base_class)]
            pub struct [<$component Class>] {
                pub variant: [<$component Variant>],
                pub size: [<$component Size>],
            }

            #[derive(TwVariant)]
            pub enum [<$component Variant>] {
                #[tw(default, class = $first_variant_class)]
                $first_variant,
                $(
                    #[tw(class = $variant_class)]
                    $variant_key,
                )*
            }

            #[derive(TwVariant)]
            pub enum [<$component Size>] {
                #[tw(default, class = $first_size_class)]
                $first_size,
                $(
                    #[tw(class = $size_class)]
                    $size_key,
                )*
            }
        }
    };

    // variant only, type only
    (
        $component:ident {
            base: $base_class:literal,
            variants: {
                variant: {
                    $first_variant:ident: $first_variant_class:literal
                    $(, $variant_key:ident: $variant_class:literal)* $(,)?
                }
            }
        }
    ) => {
        $crate::paste::paste! {
            use $crate::tw_merge::*;

            #[derive(TwClass, Clone, Copy)]
            #[tw(class = $base_class)]
            pub struct [<$component Class>] {
                pub variant: [<$component Variant>],
            }

            #[derive(TwVariant)]
            pub enum [<$component Variant>] {
                #[tw(default, class = $first_variant_class)]
                $first_variant,
                $(
                    #[tw(class = $variant_class)]
                    $variant_key,
                )*
            }
        }
    };

    // size only, type only
    (
        $component:ident {
            base: $base_class:literal,
            variants: {
                size: {
                    $first_size:ident: $first_size_class:literal
                    $(, $size_key:ident: $size_class:literal)* $(,)?
                }
            }
        }
    ) => {
        $crate::paste::paste! {
            use $crate::tw_merge::*;

            #[derive(TwClass, Clone, Copy)]
            #[tw(class = $base_class)]
            pub struct [<$component Class>] {
                pub size: [<$component Size>],
            }

            #[derive(TwVariant)]
            pub enum [<$component Size>] {
                #[tw(default, class = $first_size_class)]
                $first_size,
                $(
                    #[tw(class = $size_class)]
                    $size_key,
                )*
            }
        }
    };
}
