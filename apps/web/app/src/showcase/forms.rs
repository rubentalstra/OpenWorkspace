use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use ui::{
    AutoForm, AutoFormFields, Button, ButtonVariant, Country, Field, FieldDescription, FieldError,
    FieldSeparator, Form, FormCheckbox, FormField, FormInput, FormProvider, FormSet, FormSwitch,
    FormTextarea, InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot, InputPhone,
    InputPrompt, InputPromptFooter, InputPromptSubmit, InputPromptTextarea, InputPromptTools,
    Label, PhoneNumber, use_form,
};
use validator::Validate;

use super::{Demo, Page, Section};

/// Validated forms, auto-forms, OTP, phone and prompt inputs.
#[component]
pub fn FormsPage() -> impl IntoView {
    view! {
        <Page title="Forms" subtitle="Validated forms, auto-forms, OTP, phone and prompt inputs.">
            <ManualFormSection />
            <AutoFormSection />
            <OtpSection />
            <PhoneSection />
            <PromptSection />
        </Page>
    }
}

/// Sign-in model: a validated email plus a minimum-length password.
#[derive(Clone, Default, Serialize, Deserialize, Validate)]
struct LoginModel {
    #[validate(email(message = "Enter a valid email"))]
    email: String,
    #[validate(length(min = 8, message = "At least 8 characters"))]
    password: String,
}

#[component]
fn ManualFormSection() -> impl IntoView {
    let form = use_form::<LoginModel>();
    let submitted = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        submitted.set(form.validate_and_get().is_ok());
    };

    let email_errors = Signal::derive(move || form.error("email").into_iter().collect::<Vec<_>>());
    let password_errors =
        Signal::derive(move || form.error("password").into_iter().collect::<Vec<_>>());

    view! {
        <Section
            title="Validated form"
            description="Hand-assembled with FormProvider, Form, FormField, FormInput and FieldError. Errors surface only after a field is blurred."
        >
            <Demo>
                <FormProvider form=form>
                    <Form class="max-w-sm space-y-6" on:submit=on_submit>
                        <FormSet>
                            <FormField field="email">
                                <Label r#for="email">"Email"</Label>
                                <FormInput />
                                <FieldDescription>"We never share your address."</FieldDescription>
                                <FieldError errors=email_errors />
                            </FormField>
                            <FormField field="password">
                                <Label r#for="password">"Password"</Label>
                                <FormInput />
                                <FieldError errors=password_errors />
                            </FormField>
                        </FormSet>
                        <div class="flex gap-3 items-center">
                            <Button attr:r#type="submit">"Sign in"</Button>
                            <Button
                                variant=ButtonVariant::Ghost
                                attr:r#type="button"
                                on:click=move |_| {
                                    submitted.set(false);
                                    form.reset();
                                }
                            >
                                "Reset"
                            </Button>
                            <Show when=move || submitted.get()>
                                <span class="text-sm text-success">"Signed in."</span>
                            </Show>
                        </div>
                    </Form>
                </FormProvider>
            </Demo>
        </Section>
    }
}

/// Account model rendered automatically by [`AutoForm`] via [`AutoFormFields`].
#[derive(Clone, Default, Serialize, Deserialize, Validate)]
struct SignupModel {
    #[validate(length(min = 2, message = "Tell us your name"))]
    name: String,
    #[validate(email(message = "Enter a valid email"))]
    email: String,
    bio: String,
    subscribe: String,
    marketing: String,
}

impl AutoFormFields for SignupModel {
    fn render_fields(_form: Form<Self>) -> impl IntoView {
        view! {
            <FormField field="name">
                <Label r#for="name">"Full name"</Label>
                <FormInput />
            </FormField>
            <FormField field="email">
                <Label r#for="email">"Email"</Label>
                <FormInput />
            </FormField>
            <FormField field="bio">
                <Label r#for="bio">"Short bio"</Label>
                <FormTextarea placeholder="A sentence or two about you" />
            </FormField>
            <FieldSeparator />
            <FormField field="subscribe">
                <FormCheckbox label="Subscribe to the changelog" />
            </FormField>
            <FormField field="marketing">
                <FormSwitch label="Receive product news" />
            </FormField>
        }
    }
}

#[component]
fn AutoFormSection() -> impl IntoView {
    let form = use_form::<SignupModel>();
    let created = RwSignal::new(false);
    let on_submit = Callback::new(move |_model: SignupModel| created.set(true));

    view! {
        <Section
            title="Auto form"
            description="AutoForm renders the model's fields from an AutoFormFields impl: text inputs, a textarea, a checkbox and a switch, all bound to one form."
        >
            <Demo>
                <AutoForm form=form on_submit=on_submit>
                    <div class="flex gap-3 items-center">
                        <Button attr:r#type="submit">"Create account"</Button>
                        <Show when=move || created.get()>
                            <span class="text-sm text-success">"Account created."</span>
                        </Show>
                    </div>
                </AutoForm>
            </Demo>
        </Section>
    }
}

#[component]
fn OtpSection() -> impl IntoView {
    view! {
        <Section
            title="One-time code"
            description="InputOTP renders digit slots backed by a hidden numeric input; typing is mirrored into the slots on the client. Grouped, ungrouped and disabled layouts."
        >
            <Demo col=true>
                <Demo label="Six digits, split in two groups">
                    <InputOTP max_length=6>
                        <InputOTPGroup>
                            <InputOTPSlot index=0 />
                            <InputOTPSlot index=1 />
                            <InputOTPSlot index=2 />
                        </InputOTPGroup>
                        <InputOTPSeparator />
                        <InputOTPGroup>
                            <InputOTPSlot index=3 />
                            <InputOTPSlot index=4 />
                            <InputOTPSlot index=5 />
                        </InputOTPGroup>
                    </InputOTP>
                </Demo>
                <Demo label="Four digits, single group">
                    <InputOTP max_length=4>
                        <InputOTPGroup>
                            <InputOTPSlot index=0 />
                            <InputOTPSlot index=1 />
                            <InputOTPSlot index=2 />
                            <InputOTPSlot index=3 />
                        </InputOTPGroup>
                    </InputOTP>
                </Demo>
                <Demo label="Prefilled & disabled">
                    <InputOTP max_length=4 value="1234".to_string() disabled=true>
                        <InputOTPGroup>
                            <InputOTPSlot index=0 />
                            <InputOTPSlot index=1 />
                            <InputOTPSlot index=2 />
                            <InputOTPSlot index=3 />
                        </InputOTPGroup>
                    </InputOTP>
                </Demo>
            </Demo>
        </Section>
    }
}

#[component]
fn PhoneSection() -> impl IntoView {
    let nl_country = RwSignal::new(Country::Netherlands);
    let nl_value = RwSignal::new(PhoneNumber::default());
    let us_country = RwSignal::new(Country::UnitedStatesOfAmerica);
    let us_value = RwSignal::new(PhoneNumber::default());

    let nl_digits = Signal::derive(move || nl_value.with(|n| n.digits().to_string()));
    let invalid = Signal::derive(move || {
        nl_value.with(|n| !n.is_empty()) && nl_value.with(|n| n.digits().len()) < 9
    });

    view! {
        <Section
            title="Phone number"
            description="InputPhone pairs a searchable country selector with a national-format field. The number is kept as raw digits and grouped per country on display."
        >
            <Demo col=true>
                <Demo label="Netherlands (validated length)">
                    <Field>
                        <InputPhone
                            country_signal=nl_country
                            value_signal=nl_value
                            invalid=invalid
                        />
                        <FieldDescription>
                            {move || format!("Raw digits: {}", nl_digits.get())}
                        </FieldDescription>
                    </Field>
                </Demo>
                <Demo label="United States (default selection)">
                    <InputPhone country_signal=us_country value_signal=us_value />
                </Demo>
                <Demo label="Disabled">
                    <InputPhone disabled=true />
                </Demo>
            </Demo>
        </Section>
    }
}

#[component]
fn PromptSection() -> impl IntoView {
    let value = RwSignal::new(String::new());
    let sent = RwSignal::new(0_u32);
    let on_submit = Callback::new(move |()| {
        if value.with(|v| !v.trim().is_empty()) {
            sent.update(|n| *n += 1);
            value.set(String::new());
        }
    });

    view! {
        <Section
            title="Prompt composer"
            description="InputPrompt is a bordered column: an auto-growing textarea above a footer with leading tools and a trailing round submit button. Enter submits, Shift+Enter inserts a newline."
        >
            <Demo>
                <div class="w-full max-w-lg">
                    <InputPrompt>
                        <InputPromptTextarea
                            prop:value=move || value.get()
                            on:input=move |ev| value.set(event_target_value(&ev))
                            attr:placeholder="Ask anything…"
                            on_submit=on_submit
                        />
                        <InputPromptFooter>
                            <InputPromptTools>
                                <Button variant=ButtonVariant::Ghost attr:r#type="button">
                                    "Attach"
                                </Button>
                                <Button variant=ButtonVariant::Ghost attr:r#type="button">
                                    "Model"
                                </Button>
                            </InputPromptTools>
                            <InputPromptSubmit>"Go"</InputPromptSubmit>
                        </InputPromptFooter>
                    </InputPrompt>
                    <p class="mt-2 text-sm text-muted-foreground">
                        {move || format!("Submitted {} prompt(s)", sent.get())}
                    </p>
                </div>
            </Demo>
        </Section>
    }
}
