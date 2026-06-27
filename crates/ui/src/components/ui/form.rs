use crate::{
    Field, FieldContext, Form as FormState, FormContext, Input, SetValueFn, TouchFieldFn, clx, cn,
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

clx! {
    /// `<fieldset>` grouping form controls with consistent vertical rhythm.
    FormSet, fieldset,
    "flex flex-col gap-6 has-[>[data-name=CheckboxGroup]]:gap-3 has-[>[data-name=RadioGroup]]:gap-3"
}

/// Bridges a [`FormState`] into context so descendant field components can read
/// and mutate it without naming the model type `T`.
#[component]
pub fn FormProvider<T>(form: FormState<T>, children: Children) -> impl IntoView
where
    T: Validate + Clone + Default + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    let set_value: SetValueFn = Box::new(move |field, value| form.set_value(field, value));
    let touch_field: TouchFieldFn = Box::new(move |field| form.touch_field(field));

    provide_context(FormContext {
        values_signal: form.values_signal,
        errors_signal: form.errors_signal,
        touched_signal: form.touched_signal,
        set_value: StoredValue::new(set_value),
        touch_field: StoredValue::new(touch_field),
    });

    children()
}

/// `<form>` element bound to the surrounding [`FormProvider`]. Native attributes,
/// events and bindings forward to the root — set `on:submit` at the call site.
#[component]
pub fn Form(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    _ = expect_context::<FormContext>();

    view! {
        <form data-name="Form" class=move || cn!("w-full", class.get())>
            {children()}
        </form>
    }
}

/// Names the model field its descendants bind to and reflects validation state
/// through `data-invalid`, shown only once the field has been touched.
#[component]
pub fn FormField(#[prop(into)] field: String, children: Children) -> impl IntoView {
    provide_context(FieldContext {
        name: field.clone(),
    });

    let ctx = expect_context::<FormContext>();
    let invalid = move || {
        let touched = ctx.touched_signal.read().contains(&field);
        let has_error = ctx
            .errors_signal
            .read()
            .get(&field)
            .is_some_and(Option::is_some);
        (touched && has_error).then_some("true")
    };

    view! { <Field attr:data-invalid=invalid>{children()}</Field> }
}

/// [`Input`] wired to the enclosing [`FormField`]: it reads and writes the field
/// value, marks the field touched on blur, and toggles `aria-invalid` once a
/// touched field has an error.
#[component]
pub fn FormInput() -> impl IntoView {
    let field = expect_context::<FieldContext>().name;
    let ctx = expect_context::<FormContext>();

    let aria_invalid = {
        let field = field.clone();
        move || {
            let touched = ctx.touched_signal.read().contains(&field);
            let has_error = ctx
                .errors_signal
                .read()
                .get(&field)
                .is_some_and(Option::is_some);
            (touched && has_error).then_some("true")
        }
    };
    let value = {
        let field = field.clone();
        move || {
            ctx.values_signal
                .read()
                .get(&field)
                .cloned()
                .unwrap_or_default()
        }
    };
    let on_input = {
        let field = field.clone();
        move |ev: leptos::ev::Event| {
            ctx.set_value
                .with_value(|set| set(&field, event_target_value(&ev)));
        }
    };
    let on_blur = {
        let field = field.clone();
        move |_| ctx.touch_field.with_value(|touch| touch(&field))
    };

    view! {
        <Input
            attr:id=field
            attr:aria-invalid=aria_invalid
            prop:value=value
            on:input=on_input
            on:blur=on_blur
        />
    }
}
