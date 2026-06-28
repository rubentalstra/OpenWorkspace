use crate::components::label::Label;
use crate::components::separator::Separator;
use crate::{cn, slot};
use leptos::prelude::*;

/// Field layout orientation.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum FieldOrientation {
    #[default]
    Vertical,
    Horizontal,
    Responsive,
}

impl FieldOrientation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
            Self::Responsive => "responsive",
        }
    }
    fn class(self) -> &'static str {
        match self {
            Self::Vertical => "cn-field-orientation-vertical flex-col *:w-full [&>.sr-only]:w-auto",
            Self::Horizontal => {
                "cn-field-orientation-horizontal flex-row items-center has-[>[data-slot=field-content]]:items-start *:data-[slot=field-label]:flex-auto"
            }
            Self::Responsive => {
                "cn-field-orientation-responsive flex-col *:w-full @md/field-group:flex-row @md/field-group:items-center @md/field-group:*:w-auto"
            }
        }
    }
}

/// Field — shadcn Base UI `field`. Groups a label, control, and description.
#[component]
pub fn Field(
    #[prop(into, optional)] orientation: Signal<FieldOrientation>,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            role="group"
            data-slot="field"
            data-orientation=move || orientation.get().as_str()
            class=move || {
                cn!("cn-field group/field flex w-full", orientation.get().class(), class.get())
            }
        >
            {children()}
        </div>
    }
}

slot! {
    FieldGroup, div, "field-group",
    "cn-field-group group/field-group @container/field-group flex w-full flex-col"
}
slot! { FieldSet, fieldset, "field-set", "cn-field-set flex flex-col" }
slot! {
    FieldContent, div, "field-content",
    "cn-field-content group/field-content flex flex-1 flex-col leading-snug"
}
slot! { FieldTitle, div, "field-label", "cn-field-title flex w-fit items-center" }
slot! {
    FieldDescription, p, "field-description",
    "cn-field-description leading-normal font-normal group-has-data-horizontal/field:text-balance last:mt-0 nth-last-2:-mt-1 [&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-primary"
}

/// The field's legend (for a `FieldSet`).
#[component]
pub fn FieldLegend(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <legend
            data-slot="field-legend"
            data-variant="legend"
            class=move || cn!("cn-field-legend", class.get())
        >
            {children()}
        </legend>
    }
}

/// The field's label (wraps `Label`); associate via `attr:for`.
#[component]
pub fn FieldLabel(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <Label class=Signal::derive(move || {
            cn!(
                "cn-field-label group/field-label peer/field-label flex w-fit has-[>[data-slot=field]]:w-full has-[>[data-slot=field]]:flex-col",
                class.get(),
            )
        })>{children()}</Label>
    }
}

/// An error message for the field (announced via `role=alert`).
#[component]
pub fn FieldError(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            role="alert"
            data-slot="field-error"
            class=move || cn!("cn-field-error font-normal", class.get())
        >
            {children()}
        </div>
    }
}

/// A separator with optional centered label content.
#[component]
pub fn FieldSeparator(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let has_content = children.is_some();
    view! {
        <div
            data-slot="field-separator"
            data-content=has_content.to_string()
            class=move || cn!("cn-field-separator relative", class.get())
        >
            <Separator class="absolute inset-0 top-1/2" />
            {children
                .map(|children| {
                    view! {
                        <span
                            class="cn-field-separator-content bg-background relative mx-auto block w-fit"
                            data-slot="field-separator-content"
                        >
                            {children()}
                        </span>
                    }
                })}
        </div>
    }
}
