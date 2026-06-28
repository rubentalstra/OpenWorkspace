//! Automatic form generation from a struct model.
//!
//! `AutoForm` renders the fields of any `T: AutoFormFields` (implement that
//! trait by hand on the model to describe the field layout) and submits the
//! validated model.

use crate::{
    AutoFormFields, Checkbox, FieldContext, Form, FormContext, FormProvider, FormSet, Label,
    SwitchLabel, cn,
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

const FORM_TEXTAREA_BASE: &str = "border-input placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 flex field-sizing-content min-h-16 w-full rounded-md border bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm";

/// Renders a form from the model `T` and submits the validated model via
/// `on_submit`. The field layout comes from `T::render_fields`; pass a submit
/// button (or other trailing controls) as children.
#[component]
pub fn AutoForm<T>(
    /// The form state from `use_form::<T>()`.
    form: Form<T>,
    /// Fires with the validated model on submit.
    #[prop(optional, into)]
    on_submit: Option<Callback<T>>,
    #[prop(into, optional)] class: Signal<String>,
    /// Trailing controls, typically a submit button.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView
where
    T: AutoFormFields
        + Validate
        + Clone
        + Default
        + Serialize
        + for<'de> Deserialize<'de>
        + Send
        + Sync
        + 'static,
{
    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(callback) = on_submit
            && let Ok(data) = form.validate_and_get()
        {
            callback.run(data);
        }
    };

    view! {
        <FormProvider form=form>
            <Form class=move || cn!("max-w-md", class.get()) on:submit=handle_submit>
                <FormSet>
                    <AutoFormFieldsWrapper form=form />
                </FormSet>
                {children.map(|c| view! { <div class="mt-6">{c()}</div> })}
            </Form>
        </FormProvider>
    }
}

/// Inner wrapper so `T::render_fields` runs with the `FormContext` in scope.
#[component]
fn AutoFormFieldsWrapper<T>(form: Form<T>) -> impl IntoView
where
    T: AutoFormFields + 'static,
{
    T::render_fields(form)
}

/// Multi-line input bound to the enclosing [`FormField`]'s value; marks the
/// field touched on blur and toggles `aria-invalid` once a touched field errors.
#[component]
pub fn FormTextarea(
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] placeholder: String,
) -> impl IntoView {
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
    let on_blur = move |_| ctx.touch_field.with_value(|touch| touch(&field));

    view! {
        <textarea
            data-name="FormTextarea"
            class=cn!(FORM_TEXTAREA_BASE, class)
            placeholder=placeholder
            aria-invalid=aria_invalid
            prop:value=value
            on:input=on_input
            on:blur=on_blur
        />
    }
}

/// Checkbox bound to the enclosing [`FormField`]'s value (stored as `"true"`/`"false"`).
#[component]
pub fn FormCheckbox(
    #[prop(into, optional)] class: String,
    #[prop(into)] label: String,
) -> impl IntoView {
    let field = expect_context::<FieldContext>().name;
    let ctx = expect_context::<FormContext>();

    let checked = Signal::derive({
        let field = field.clone();
        move || {
            ctx.values_signal
                .read()
                .get(&field)
                .is_some_and(|v| v == "true")
        }
    });
    let on_change = {
        let field = field.clone();
        Callback::new(move |value: bool| {
            ctx.set_value
                .with_value(|set| set(&field, value.to_string()));
        })
    };

    view! {
        <div class=cn!("flex items-center gap-2", class)>
            <Checkbox checked=checked on_checked_change=on_change />
            <Label r#for=field class="text-sm font-medium cursor-pointer">
                {label}
            </Label>
        </div>
    }
}

/// Toggle switch bound to the enclosing [`FormField`]'s value.
#[component]
pub fn FormSwitch(
    #[prop(into, optional)] class: String,
    #[prop(into)] label: String,
) -> impl IntoView {
    let field = expect_context::<FieldContext>().name;
    let ctx = expect_context::<FormContext>();

    let checked = {
        let field = field.clone();
        move || {
            ctx.values_signal
                .read()
                .get(&field)
                .is_some_and(|v| v == "true")
        }
    };
    let on_change = {
        let field = field.clone();
        move |ev: leptos::ev::Event| {
            let value = event_target::<web_sys::HtmlInputElement>(&ev).checked();
            ctx.set_value
                .with_value(|set| set(&field, value.to_string()));
        }
    };

    view! {
        <div class=cn!("flex gap-2", class)>
            <label class="inline-flex relative items-center cursor-pointer" tabindex="0">
                <input
                    type="checkbox"
                    class="hidden peer"
                    id=field
                    checked=checked
                    on:change=on_change
                />
                <div
                    data-name="Switch"
                    class="w-11 h-6 bg-gray-200 rounded-full peer-focus:outline-hidden peer-focus:ring-ring/50 peer-focus:ring-[3px] peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:size-5 after:transition-all peer-checked:bg-primary"
                />
            </label>
            <SwitchLabel>{label}</SwitchLabel>
        </div>
    }
}
