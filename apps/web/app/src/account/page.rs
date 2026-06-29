//! `/account` — the signed-in user's security settings: password, two-factor
//! (authenticator app), passkeys, and recovery codes. Pure composition of the
//! `crates/ui` kit; every mutation calls a server fn in [`super`].

use leptos::prelude::*;
use leptos::task::spawn_local;
use ui::{
    Alert, AlertDescription, AlertTitle, AlertVariant, Badge, Button, ButtonSize, ButtonVariant,
    Card, CardContent, CardDescription, CardHeader, CardTitle, Field, FieldDescription, FieldLabel,
    Input, InputOtp, InputOtpGroup, InputOtpSeparator, InputOtpSlot, Tabs, TabsContent, TabsList,
    TabsTrigger,
};

use super::{
    PasskeyDto, change_password, delete_passkey, list_passkeys, mfa_status,
    passkey_register_finish, passkey_register_start, regenerate_recovery_codes, totp_disable,
    totp_enroll_confirm, totp_enroll_start,
};

/// Strip the server-fn wrapper from an error so the user sees the plain message.
fn msg(err: &ServerFnError) -> String {
    err.to_string()
        .rsplit(": ")
        .next()
        .unwrap_or("Something went wrong.")
        .to_owned()
}

/// A six-slot one-time-code input bound to `code`.
#[component]
fn CodeInput(code: RwSignal<String>) -> impl IntoView {
    view! {
        <InputOtp max_length=6 value=code>
            <InputOtpGroup>
                <InputOtpSlot index=0 />
                <InputOtpSlot index=1 />
                <InputOtpSlot index=2 />
            </InputOtpGroup>
            <InputOtpSeparator />
            <InputOtpGroup>
                <InputOtpSlot index=3 />
                <InputOtpSlot index=4 />
                <InputOtpSlot index=5 />
            </InputOtpGroup>
        </InputOtp>
    }
}

/// A success/error banner driven by an optional `Result<message, message>`.
fn status_banner(status: RwSignal<Option<Result<String, String>>>) -> impl IntoView {
    move || {
        status.get().map(|s| match s {
            Ok(m) => view! {
                <Alert>
                    <AlertTitle>"Done"</AlertTitle>
                    <AlertDescription>{m}</AlertDescription>
                </Alert>
            }
            .into_any(),
            Err(m) => view! {
                <Alert variant=AlertVariant::Destructive>
                    <AlertTitle>"That didn't work"</AlertTitle>
                    <AlertDescription>{m}</AlertDescription>
                </Alert>
            }
            .into_any(),
        })
    }
}

/// Render a freshly issued set of recovery codes (shown once).
fn codes_panel(codes: Vec<String>) -> impl IntoView {
    view! {
        <Alert>
            <AlertTitle>"Save your recovery codes"</AlertTitle>
            <AlertDescription>
                "Each code works once. Store them somewhere safe — they are shown only now."
                <ul class="mt-2 grid grid-cols-2 gap-1 font-mono text-sm">
                    {codes.into_iter().map(|c| view! { <li>{c}</li> }).collect_view()}
                </ul>
            </AlertDescription>
        </Alert>
    }
}

/// `/account` — security settings, tabbed.
#[component]
pub fn AccountPage() -> impl IntoView {
    view! {
        <div class="container max-w-2xl py-8">
            <header class="mb-6 flex items-center justify-between">
                <h1 class="cn-font-heading text-2xl font-semibold tracking-tight">"Security"</h1>
                <a href="/dashboard" class="text-muted-foreground text-sm">
                    "← Back"
                </a>
            </header>
            <Tabs default_value="password" class="gap-6">
                <TabsList>
                    <TabsTrigger value="password">"Password"</TabsTrigger>
                    <TabsTrigger value="2fa">"Two-factor"</TabsTrigger>
                    <TabsTrigger value="passkeys">"Passkeys"</TabsTrigger>
                    <TabsTrigger value="recovery">"Recovery"</TabsTrigger>
                </TabsList>
                <TabsContent value="password">
                    <PasswordCard />
                </TabsContent>
                <TabsContent value="2fa">
                    <TwoFactorCard />
                </TabsContent>
                <TabsContent value="passkeys">
                    <PasskeysCard />
                </TabsContent>
                <TabsContent value="recovery">
                    <RecoveryCard />
                </TabsContent>
            </Tabs>
        </div>
    }
}

#[component]
fn PasswordCard() -> impl IntoView {
    let current = RwSignal::new(String::new());
    let next = RwSignal::new(String::new());
    let confirm = RwSignal::new(String::new());
    let status = RwSignal::new(None::<Result<String, String>>);
    let pending = RwSignal::new(false);

    let submit = move |_| {
        if pending.get_untracked() {
            return;
        }
        let (c, n, cf) = (
            current.get_untracked(),
            next.get_untracked(),
            confirm.get_untracked(),
        );
        if n != cf {
            status.set(Some(Err("The new passwords do not match.".to_owned())));
            return;
        }
        pending.set(true);
        status.set(None);
        spawn_local(async move {
            let result = change_password(c, n).await;
            pending.set(false);
            match result {
                Ok(()) => {
                    status.set(Some(Ok("Your password has been changed.".to_owned())));
                    current.set(String::new());
                    next.set(String::new());
                    confirm.set(String::new());
                }
                Err(e) => status.set(Some(Err(msg(&e)))),
            }
        });
    };

    view! {
        <Card>
            <CardHeader>
                <CardTitle>"Change password"</CardTitle>
                <CardDescription>"Other signed-in sessions will be signed out."</CardDescription>
            </CardHeader>
            <CardContent class="flex flex-col gap-4">
                {status_banner(status)} <Field>
                    <FieldLabel attr:r#for="current_pw">"Current password"</FieldLabel>
                    <Input
                        attr:id="current_pw"
                        attr:r#type="password"
                        attr:autocomplete="current-password"
                        prop:value=move || current.get()
                        on:input=move |ev| current.set(event_target_value(&ev))
                    />
                </Field> <Field>
                    <FieldLabel attr:r#for="new_pw">"New password"</FieldLabel>
                    <Input
                        attr:id="new_pw"
                        attr:r#type="password"
                        attr:autocomplete="new-password"
                        prop:value=move || next.get()
                        on:input=move |ev| next.set(event_target_value(&ev))
                    />
                </Field> <Field>
                    <FieldLabel attr:r#for="confirm_pw">"Confirm new password"</FieldLabel>
                    <Input
                        attr:id="confirm_pw"
                        attr:r#type="password"
                        attr:autocomplete="new-password"
                        prop:value=move || confirm.get()
                        on:input=move |ev| confirm.set(event_target_value(&ev))
                    />
                </Field> <div>
                    <Button on:click=submit attr:disabled=move || pending.get()>
                        "Change password"
                    </Button>
                </div>
            </CardContent>
        </Card>
    }
}

#[component]
fn TwoFactorCard() -> impl IntoView {
    let status = Resource::new(|| (), |()| mfa_status());
    // Local enrolment state.
    let enrolling = RwSignal::new(Option::<super::TotpEnrollDto>::None);
    let code = RwSignal::new(String::new());
    let codes = RwSignal::new(Option::<Vec<String>>::None);
    let banner = RwSignal::new(None::<Result<String, String>>);
    let pending = RwSignal::new(false);
    let armed = RwSignal::new(false);

    let begin = move |_| {
        pending.set(true);
        banner.set(None);
        spawn_local(async move {
            match totp_enroll_start().await {
                Ok(dto) => enrolling.set(Some(dto)),
                Err(e) => banner.set(Some(Err(msg(&e)))),
            }
            pending.set(false);
        });
    };
    let confirm = move |_| {
        let c = code.get_untracked();
        pending.set(true);
        spawn_local(async move {
            match totp_enroll_confirm(c).await {
                Ok(rc) => {
                    codes.set(Some(rc.codes));
                    enrolling.set(None);
                    banner.set(None);
                    status.refetch();
                }
                Err(e) => banner.set(Some(Err(msg(&e)))),
            }
            pending.set(false);
        });
    };
    let disable = move |_| {
        if !armed.get_untracked() {
            armed.set(true);
            return;
        }
        spawn_local(async move {
            match totp_disable().await {
                Ok(()) => {
                    armed.set(false);
                    codes.set(None);
                    status.refetch();
                }
                Err(e) => banner.set(Some(Err(msg(&e)))),
            }
        });
    };

    view! {
        <Card>
            <CardHeader>
                <CardTitle>"Authenticator app"</CardTitle>
                <CardDescription>
                    "Use a TOTP app (1Password, Authy, Google Authenticator) as a second factor."
                </CardDescription>
            </CardHeader>
            <CardContent class="flex flex-col gap-4">
                {status_banner(banner)} {move || codes.get().map(codes_panel)}
                <Suspense fallback=|| {
                    view! { <p class="text-muted-foreground text-sm">"Loading…"</p> }
                }>
                    {move || Suspend::new(async move {
                        let enabled = status.await.map(|s| s.totp_enabled).unwrap_or(false);
                        if enabled {
                            view! {
                                <div class="flex items-center gap-3">
                                    <Badge>"Enabled"</Badge>
                                    <Button
                                        variant=ButtonVariant::Destructive
                                        size=ButtonSize::Sm
                                        on:click=disable
                                    >
                                        {move || {
                                            if armed.get() {
                                                "Click again to disable"
                                            } else {
                                                "Disable"
                                            }
                                        }}
                                    </Button>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                {move || {
                                    enrolling
                                        .get()
                                        .map_or_else(
                                            || {
                                                view! {
                                                    <div>
                                                        <Button on:click=begin attr:disabled=move || pending.get()>
                                                            "Set up authenticator"
                                                        </Button>
                                                    </div>
                                                }
                                                    .into_any()
                                            },
                                            |dto| {
                                                let qr = format!(
                                                    "data:image/png;base64,{}",
                                                    dto.qr_png_base64,
                                                );
                                                view! {
                                                    <div class="flex flex-col items-start gap-3">
                                                        <img
                                                            src=qr
                                                            alt="Authenticator QR code"
                                                            class="size-44 rounded-md border"
                                                        />
                                                        <p class="text-muted-foreground text-sm">
                                                            "Or enter this secret manually:"
                                                        </p>
                                                        <code class="bg-muted rounded px-2 py-1 font-mono text-sm">
                                                            {dto.secret_base32}
                                                        </code>
                                                        <CodeInput code=code />
                                                        <Button
                                                            on:click=confirm
                                                            attr:disabled=move || pending.get()
                                                        >
                                                            "Confirm"
                                                        </Button>
                                                    </div>
                                                }
                                                    .into_any()
                                            },
                                        )
                                }}
                            }
                                .into_any()
                        }
                    })}
                </Suspense>
            </CardContent>
        </Card>
    }
}

#[component]
fn PasskeysCard() -> impl IntoView {
    let passkeys = Resource::new(|| (), |()| list_passkeys());
    let banner = RwSignal::new(None::<Result<String, String>>);
    let pending = RwSignal::new(false);
    let arming = RwSignal::new(Option::<String>::None);

    let add = move |_| {
        if pending.get_untracked() {
            return;
        }
        pending.set(true);
        banner.set(None);
        spawn_local(async move {
            let result: Result<(), String> = async {
                let options = passkey_register_start().await.map_err(|e| msg(&e))?;
                let credential = crate::webauthn::create(options).await?;
                passkey_register_finish(credential, None)
                    .await
                    .map_err(|e| msg(&e))
            }
            .await;
            match result {
                Ok(()) => {
                    banner.set(Some(Ok("Passkey added.".to_owned())));
                    passkeys.refetch();
                }
                Err(e) => banner.set(Some(Err(e))),
            }
            pending.set(false);
        });
    };

    let remove = move |id: String| {
        if arming.get_untracked().as_deref() != Some(id.as_str()) {
            arming.set(Some(id));
            return;
        }
        spawn_local(async move {
            match delete_passkey(id).await {
                Ok(()) => {
                    arming.set(None);
                    passkeys.refetch();
                }
                Err(e) => banner.set(Some(Err(msg(&e)))),
            }
        });
    };

    view! {
        <Card>
            <CardHeader>
                <CardTitle>"Passkeys"</CardTitle>
                <CardDescription>
                    "Sign in with your device's biometrics or a security key."
                </CardDescription>
            </CardHeader>
            <CardContent class="flex flex-col gap-4">
                {status_banner(banner)} <div>
                    <Button on:click=add attr:disabled=move || pending.get()>
                        "Add a passkey"
                    </Button>
                </div>
                <Suspense fallback=|| {
                    view! { <p class="text-muted-foreground text-sm">"Loading…"</p> }
                }>
                    {move || Suspend::new(async move {
                        match passkeys.await {
                            Ok(list) if list.is_empty() => {
                                view! {
                                    <p class="text-muted-foreground text-sm">"No passkeys yet."</p>
                                }
                                    .into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <ul class="divide-border divide-y rounded-md border">
                                        {list
                                            .into_iter()
                                            .map(|p| passkey_row(p, remove))
                                            .collect_view()}
                                    </ul>
                                }
                                    .into_any()
                            }
                            Err(_) => {
                                view! {
                                    <p class="text-destructive text-sm">
                                        "Sign in to manage passkeys."
                                    </p>
                                }
                                    .into_any()
                            }
                        }
                    })}
                </Suspense>
            </CardContent>
        </Card>
    }
}

fn passkey_row(p: PasskeyDto, remove: impl Fn(String) + Copy + 'static) -> impl IntoView {
    let id = p.id.clone();
    let label = p.label.unwrap_or_else(|| "Passkey".to_owned());
    view! {
        <li class="flex items-center justify-between gap-3 px-3 py-2">
            <div class="min-w-0">
                <p class="truncate text-sm font-medium">{label}</p>
                <p class="text-muted-foreground text-xs">"Added " {p.created_at}</p>
            </div>
            <Button
                variant=ButtonVariant::Destructive
                size=ButtonSize::Sm
                on:click=move |_| remove(id.clone())
            >
                "Remove"
            </Button>
        </li>
    }
}

#[component]
fn RecoveryCard() -> impl IntoView {
    let status = Resource::new(|| (), |()| mfa_status());
    let codes = RwSignal::new(Option::<Vec<String>>::None);
    let banner = RwSignal::new(None::<Result<String, String>>);
    let armed = RwSignal::new(false);

    let regenerate = move |_| {
        if !armed.get_untracked() {
            armed.set(true);
            return;
        }
        spawn_local(async move {
            match regenerate_recovery_codes().await {
                Ok(rc) => {
                    codes.set(Some(rc.codes));
                    armed.set(false);
                    status.refetch();
                }
                Err(e) => banner.set(Some(Err(msg(&e)))),
            }
        });
    };

    view! {
        <Card>
            <CardHeader>
                <CardTitle>"Recovery codes"</CardTitle>
                <CardDescription>
                    "One-time codes to sign in if you lose your authenticator."
                </CardDescription>
            </CardHeader>
            <CardContent class="flex flex-col gap-4">
                {status_banner(banner)} {move || codes.get().map(codes_panel)}
                <Suspense fallback=|| ()>
                    {move || Suspend::new(async move {
                        let remaining = status.await.map(|s| s.recovery_remaining).unwrap_or(0);
                        view! {
                            <p class="text-muted-foreground text-sm">
                                {remaining} " unused codes remaining."
                            </p>
                        }
                    })}
                </Suspense>
                <FieldDescription>"Regenerating replaces any existing codes."</FieldDescription>
                <div>
                    <Button variant=ButtonVariant::Outline on:click=regenerate>
                        {move || {
                            if armed.get() {
                                "Click again to regenerate"
                            } else {
                                "Regenerate codes"
                            }
                        }}
                    </Button>
                </div>
            </CardContent>
        </Card>
    }
}
