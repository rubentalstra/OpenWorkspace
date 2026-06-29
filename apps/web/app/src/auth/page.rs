//! The sign-in and sign-up pages. The login card lists the configured SSO
//! providers (each links to the server's `/auth/{slug}/start`) above an
//! email/password form wired to [`password_login`](super::password_login). On
//! success it navigates with a full reload so the new session cookie drives a
//! fresh authenticated render.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;
use leptos_router::hooks::use_query_map;
use ui::{
    Button, ButtonVariant, Card, CardContent, CardDescription, CardHeader, CardTitle, Field,
    FieldDescription, FieldGroup, FieldLabel, FieldSeparator, Input,
};

use super::{OidcProviderDto, list_oidc_providers, login};

/// login page: brand mark over the login card, centered on a muted background.
#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="bg-muted flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10">
            <div class="flex w-full max-w-sm flex-col gap-6">
                <a href="/" class="flex items-center gap-2 self-center font-medium">
                    <div class="bg-primary text-primary-foreground flex size-6 items-center justify-center rounded-md">
                        <Icon icon=icondata::LuGalleryVerticalEnd attr:class="size-4" />
                    </div>
                    "OpenWorkspace"
                </a>
                <LoginForm />
            </div>
        </div>
    }
}

/// The login card: SSO buttons (when any provider is configured), a separator, and
/// the email/password form.
#[component]
fn LoginForm() -> impl IntoView {
    let providers = Resource::new(|| (), |()| list_oidc_providers());
    let query = use_query_map();
    // A post-login destination, constrained to a local path (no open redirect).
    let return_to = move || {
        let raw = query.read().get("return_to").unwrap_or_default();
        if raw.starts_with('/') && !raw.starts_with("//") {
            raw
        } else {
            "/".to_owned()
        }
    };

    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let pending = RwSignal::new(false);

    let submit = move || {
        if pending.get_untracked() {
            return;
        }
        error.set(None);
        pending.set(true);
        let email_v = email.get_untracked();
        let password_v = password.get_untracked();
        let target = return_to();
        spawn_local(async move {
            match login(email_v, password_v).await {
                Ok(outcome) if outcome.mfa_required => {
                    pending.set(false);
                    error.set(Some(
                        "Multi-factor authentication is required for this account. \
                         Complete it to finish signing in."
                            .to_owned(),
                    ));
                }
                Ok(_) => navigate_to(&target),
                Err(e) => {
                    pending.set(false);
                    let msg = e.to_string();
                    error.set(Some(if msg.contains("invalid email or password") {
                        "Invalid email or password.".to_owned()
                    } else {
                        "Sign-in failed. Please try again.".to_owned()
                    }));
                }
            }
        });
    };

    let sso = move || {
        let target = return_to();
        Suspend::new(async move {
            match providers.await {
                Ok(list) if !list.is_empty() => sso_block(&list, &target),
                _ => ().into_any(),
            }
        })
    };

    view! {
        <div class="flex flex-col gap-6">
            <Card>
                <CardHeader class="text-center">
                    <CardTitle class="text-xl">"Welcome back"</CardTitle>
                    <CardDescription>"Sign in to your workspace"</CardDescription>
                </CardHeader>
                <CardContent>
                    <form on:submit=move |ev| {
                        ev.prevent_default();
                        submit();
                    }>
                        <FieldGroup>
                            <Suspense fallback=|| ()>{sso}</Suspense>
                            <Field>
                                <FieldLabel attr:r#for="email">"Email"</FieldLabel>
                                <Input
                                    attr:id="email"
                                    attr:name="email"
                                    attr:r#type="email"
                                    attr:autocomplete="email"
                                    attr:placeholder="m@example.com"
                                    attr:required=true
                                    prop:value=move || email.get()
                                    on:input=move |ev| email.set(event_target_value(&ev))
                                />
                            </Field>
                            <Field>
                                <FieldLabel attr:r#for="password">"Password"</FieldLabel>
                                <Input
                                    attr:id="password"
                                    attr:name="password"
                                    attr:r#type="password"
                                    attr:autocomplete="current-password"
                                    attr:required=true
                                    prop:value=move || password.get()
                                    on:input=move |ev| password.set(event_target_value(&ev))
                                />
                            </Field>
                            {move || {
                                error
                                    .get()
                                    .map(|e| {
                                        view! {
                                            <p class="text-destructive text-sm" role="alert">
                                                {e}
                                            </p>
                                        }
                                    })
                            }}
                            <Field>
                                <Button attr:r#type="submit" attr:disabled=move || pending.get()>
                                    "Sign in"
                                </Button>
                                <FieldDescription class="text-center">
                                    "Don't have an account? " <a href="/signup">"Sign up"</a>
                                </FieldDescription>
                            </Field>
                        </FieldGroup>
                    </form>
                </CardContent>
            </Card>
            <FieldDescription class="px-6 text-center">
                "By continuing, you agree to our " <a href="#">"Terms of Service"</a> " and "
                <a href="#">"Privacy Policy"</a> "."
            </FieldDescription>
        </div>
    }
}

/// The SSO button group + separator, shown only when a provider is configured.
fn sso_block(list: &[OidcProviderDto], return_to: &str) -> AnyView {
    let buttons = list
        .iter()
        .map(|p| provider_button(p, return_to))
        .collect_view();
    view! {
        <Field>{buttons}</Field>
        <FieldSeparator class="*:data-[slot=field-separator-content]:bg-card">
            "Or continue with"
        </FieldSeparator>
    }
    .into_any()
}

/// One "Continue with …" button: a full navigation to the provider's start route.
/// `+ use<>` keeps the returned view `'static` (it owns `href`/`label`, so it need
/// not capture the borrowed inputs).
fn provider_button(provider: &OidcProviderDto, return_to: &str) -> impl IntoView + use<> {
    let href = format!("/auth/{}/start?return_to={return_to}", provider.slug);
    let label = format!("Continue with {}", provider.label);
    view! {
        <Button href=href variant=ButtonVariant::Outline class="w-full">
            {label}
        </Button>
    }
}

/// Navigate with a full page load so the freshly-set session cookie drives a new
/// authenticated server render (an SPA push would keep the anonymous context).
fn navigate_to(path: &str) {
    if let Some(window) = web_sys::window() {
        // A failed navigation is unrecoverable and nothing the user can act on.
        _ = window.location().set_href(path);
    }
}

/// signup page: brand mark over the signup card, centered on a muted background.
#[component]
pub fn SignupPage() -> impl IntoView {
    view! {
        <div class="bg-muted flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10">
            <div class="flex w-full max-w-sm flex-col gap-6">
                <a href="/" class="flex items-center gap-2 self-center font-medium">
                    <div class="bg-primary text-primary-foreground flex size-6 items-center justify-center rounded-md">
                        <Icon icon=icondata::LuGalleryVerticalEnd attr:class="size-4" />
                    </div>
                    "OpenWorkspace"
                </a>
                <SignupForm />
            </div>
        </div>
    }
}

/// The signup card: name, email, and a two-column password/confirm pair.
///
/// Self-service registration is not yet wired to a server endpoint (accounts are
/// admin-provisioned or created via SSO just-in-time); the form is presentational
/// until that flow exists.
#[component]
fn SignupForm() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6">
            <Card>
                <CardHeader class="text-center">
                    <CardTitle class="text-xl">"Create your account"</CardTitle>
                    <CardDescription>
                        "Enter your email below to create your account"
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <form>
                        <FieldGroup>
                            <Field>
                                <FieldLabel attr:r#for="name">"Full Name"</FieldLabel>
                                <Input
                                    attr:id="name"
                                    attr:r#type="text"
                                    attr:placeholder="John Doe"
                                    attr:required=true
                                />
                            </Field>
                            <Field>
                                <FieldLabel attr:r#for="email">"Email"</FieldLabel>
                                <Input
                                    attr:id="email"
                                    attr:r#type="email"
                                    attr:placeholder="m@example.com"
                                    attr:required=true
                                />
                            </Field>
                            <Field>
                                <Field class="grid grid-cols-2 gap-4">
                                    <Field>
                                        <FieldLabel attr:r#for="password">"Password"</FieldLabel>
                                        <Input
                                            attr:id="password"
                                            attr:r#type="password"
                                            attr:required=true
                                        />
                                    </Field>
                                    <Field>
                                        <FieldLabel attr:r#for="confirm-password">
                                            "Confirm Password"
                                        </FieldLabel>
                                        <Input
                                            attr:id="confirm-password"
                                            attr:r#type="password"
                                            attr:required=true
                                        />
                                    </Field>
                                </Field>
                                <FieldDescription>
                                    "Must be at least 8 characters long."
                                </FieldDescription>
                            </Field>
                            <Field>
                                <Button attr:r#type="submit">"Create Account"</Button>
                                <FieldDescription class="text-center">
                                    "Already have an account? " <a href="/login">"Sign in"</a>
                                </FieldDescription>
                            </Field>
                        </FieldGroup>
                    </form>
                </CardContent>
            </Card>
            <FieldDescription class="px-6 text-center">
                "By continuing, you agree to our " <a href="#">"Terms of Service"</a> " and "
                <a href="#">"Privacy Policy"</a> "."
            </FieldDescription>
        </div>
    }
}
