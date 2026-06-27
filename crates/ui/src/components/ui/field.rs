use crate::{Label, Separator, clx, cn, variants};
use leptos::prelude::*;

clx! {FieldSet, fieldset, "flex flex-col gap-6 has-[>[data-name=CheckboxGroup]]:gap-3 has-[>[data-name=RadioGroup]]:gap-3"}
clx! {FieldGroup, div, "group/field-group @container/field-group flex flex-col gap-7 w-full has-[>[data-name=CheckboxGroup]]:gap-3 [&>[data-name=FieldGroup]]:gap-4"}
clx! {FieldContent, div, "group/field-content flex flex-1 flex-col gap-1.5 leading-snug"}
clx! {FieldTitle, div, "flex items-center gap-2 text-sm leading-snug font-medium w-fit group-data-[disabled=true]/field:opacity-50"}
clx! {FieldDescription, p, "text-muted-foreground text-sm leading-normal font-normal group-has-[[data-orientation=horizontal]]/field:text-balance last:mt-0 nth-last-2:-mt-1 [[data-variant=legend]+&]:-mt-1.5 [&>a:hover]:text-primary [&>a]:underline [&>a]:underline-offset-4"}

variants! {
    Field {
        base: "group/field flex gap-3 w-full data-[invalid=true]:text-destructive",
        variants: {
            variant: {
                Vertical: "flex-col [&>*]:w-full [&>.hidden]:w-auto",
                Horizontal: "flex-row items-center [&>[data-slot=field-label]]:flex-auto has-[>[data-name=FieldContent]]:items-start has-[>[data-name=FieldContent]]:[&>[role=checkbox],[role=radio]]:mt-px",
                Responsive: "flex-col [&>*]:w-full [&>.hidden]:w-auto @md/field-group:flex-row @md/field-group:items-center @md/field-group:[&>*]:w-auto @md/field-group:[&>[data-slot=field-label]]:flex-auto @md/field-group:has-[>[data-name=FieldContent]]:items-start @md/field-group:has-[>[data-name=FieldContent]]:[&>[role=checkbox],[role=radio]]:mt-px",
            },
            size: {
                Default: "",
            }
        },
        component: {
            element: div
        }
    }
}

/// `<legend>` for a [`FieldSet`]. `variant` toggles between legend and label
/// sizing via the `data-variant` styling hook.
#[component]
pub fn FieldLegend(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] variant: Signal<FieldLegendVariant>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        cn!(
            "mb-3 font-medium data-[variant=legend]:text-base data-[variant=label]:text-sm",
            class.get()
        )
    };
    let data_variant = move || match variant.get() {
        FieldLegendVariant::Legend => "legend",
        FieldLegendVariant::Label => "label",
    };

    view! {
        <legend
            data-name="FieldLegend"
            data-slot="field-legend"
            data-variant=data_variant
            class=merged
        >
            {children()}
        </legend>
    }
}

/// Sizing axis for [`FieldLegend`].
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldLegendVariant {
    #[default]
    Legend,
    Label,
}

/// [`Label`] bound to a field control. Forward `attr:for` to associate it, or
/// wrap the control as a child for nested layouts.
#[component]
pub fn FieldLabel(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let merged = move || {
        cn!(
            "group/field-label peer/field-label flex gap-2 leading-snug w-fit group-data-[disabled=true]/field:opacity-50 has-[>[data-name=Field]]:w-full has-[>[data-name=Field]]:flex-col has-[>[data-name=Field]]:rounded-md has-[>[data-name=Field]]:border [&>*]:data-[name=Field]:p-4 has-data-[state=checked]:bg-primary/5 has-data-[state=checked]:border-primary dark:has-data-[state=checked]:bg-primary/10 has-[:checked]:bg-primary/5 has-[:checked]:border-primary dark:has-[:checked]:bg-primary/10",
            class.get()
        )
    };

    view! {
        <Label attr:data-name="FieldLabel" attr:data-slot="field-label" class=merged>
            {children()}
        </Label>
    }
}

/// [`Separator`] tuned for field groups; optional children render as inline,
/// centered content (e.g. "Or") over the rule.
#[component]
pub fn FieldSeparator(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let has_content = children.is_some();
    let merged = move || {
        cn!(
            "relative -my-2 h-5 text-sm group-data-[variant=outline]/field-group:-mb-2",
            class.get()
        )
    };

    view! {
        <div
            data-name="FieldSeparator"
            data-slot="field-separator"
            data-content=has_content.to_string()
            class=merged
        >
            <Separator class="absolute inset-0 top-1/2" />
            {children
                .map(|children| {
                    view! {
                        <span
                            data-slot="field-separator-content"
                            class="block relative px-2 mx-auto bg-background text-muted-foreground w-fit"
                        >
                            {children()}
                        </span>
                    }
                })}
        </div>
    }
}

/// Validation message for a field. Render free-form `children`, or pass a list
/// of `errors` to render one inline (singular) or a bulleted list (plural).
#[component]
pub fn FieldError(
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] children: Option<Children>,
    #[prop(into, optional)] errors: Signal<Vec<String>>,
) -> impl IntoView {
    let merged = move || cn!("text-destructive text-sm font-normal", class.get());

    if let Some(children) = children {
        return view! {
            <div data-name="FieldError" data-slot="field-error" role="alert" class=merged>
                {children()}
            </div>
        }
        .into_any();
    }

    view! {
        {move || {
            let errors = errors.get();
            match errors.as_slice() {
                [] => None,
                [single] => {
                    let single = single.clone();
                    Some(
                        view! {
                            <div
                                data-name="FieldError"
                                data-slot="field-error"
                                role="alert"
                                class=merged
                            >
                                <span>{single}</span>
                            </div>
                        }
                            .into_any(),
                    )
                }
                many => {
                    let items = many
                        .iter()
                        .cloned()
                        .map(|error| view! { <li>{error}</li> })
                        .collect_view();
                    Some(
                        view! {
                            <div
                                data-name="FieldError"
                                data-slot="field-error"
                                role="alert"
                                class=merged
                            >
                                <ul class="flex flex-col gap-1 ml-4 list-disc">{items}</ul>
                            </div>
                        }
                            .into_any(),
                    )
                }
            }
        }}
    }
    .into_any()
}
