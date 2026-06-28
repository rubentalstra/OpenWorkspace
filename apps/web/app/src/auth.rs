//! login-03 and signup-03 — faithful Leptos ports of shadcn's auth blocks. The login
//! card offers Apple/Google buttons, a separator, and email/password fields; the
//! signup card collects name, email, and a two-column password pair. Both are wrapped
//! in the centered, muted-background page shell from the blocks' `page.tsx`.

use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{
    Button, ButtonVariant, Card, CardContent, CardDescription, CardHeader, CardTitle, Field,
    FieldDescription, FieldGroup, FieldLabel, FieldSeparator, Input,
};

/// login-03 page: brand mark over the login card, centered on a muted background.
#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="bg-muted flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10">
            <div class="flex w-full max-w-sm flex-col gap-6">
                <a href="#" class="flex items-center gap-2 self-center font-medium">
                    <div class="bg-primary text-primary-foreground flex size-6 items-center justify-center rounded-md">
                        <Icon icon=icondata::LuGalleryVerticalEnd attr:class="size-4" />
                    </div>
                    "Acme Inc."
                </a>
                <LoginForm />
            </div>
        </div>
    }
}

/// The login card with social buttons, a separator, and the credential fields.
#[component]
fn LoginForm() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6">
            <Card>
                <CardHeader class="text-center">
                    <CardTitle class="text-xl">"Welcome back"</CardTitle>
                    <CardDescription>"Login with your Apple or Google account"</CardDescription>
                </CardHeader>
                <CardContent>
                    <form>
                        <FieldGroup>
                            <Field>
                                <Button variant=ButtonVariant::Outline attr:r#type="button">
                                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                        <path
                                            d="M12.152 6.896c-.948 0-2.415-1.078-3.96-1.04-2.04.027-3.91 1.183-4.961 3.014-2.117 3.675-.546 9.103 1.519 12.09 1.013 1.454 2.208 3.09 3.792 3.039 1.52-.065 2.09-.987 3.935-.987 1.831 0 2.35.987 3.96.948 1.637-.026 2.676-1.48 3.676-2.948 1.156-1.688 1.636-3.325 1.662-3.415-.039-.013-3.182-1.221-3.22-4.857-.026-3.04 2.48-4.494 2.597-4.559-1.429-2.09-3.623-2.324-4.39-2.376-2-.156-3.675 1.09-4.61 1.09zM15.53 3.83c.843-1.012 1.4-2.427 1.245-3.83-1.207.052-2.662.805-3.532 1.818-.78.896-1.454 2.338-1.273 3.714 1.338.104 2.715-.688 3.559-1.701"
                                            fill="currentColor"
                                        ></path>
                                    </svg>
                                    "Login with Apple"
                                </Button>
                                <Button variant=ButtonVariant::Outline attr:r#type="button">
                                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                        <path
                                            d="M12.48 10.92v3.28h7.84c-.24 1.84-.853 3.187-1.787 4.133-1.147 1.147-2.933 2.4-6.053 2.4-4.827 0-8.6-3.893-8.6-8.72s3.773-8.72 8.6-8.72c2.6 0 4.507 1.027 5.907 2.347l2.307-2.307C18.747 1.44 16.133 0 12.48 0 5.867 0 .307 5.387.307 12s5.56 12 12.173 12c3.573 0 6.267-1.173 8.373-3.36 2.16-2.16 2.84-5.213 2.84-7.667 0-.76-.053-1.467-.173-2.053H12.48z"
                                            fill="currentColor"
                                        ></path>
                                    </svg>
                                    "Login with Google"
                                </Button>
                            </Field>
                            <FieldSeparator class="*:data-[slot=field-separator-content]:bg-card">
                                "Or continue with"
                            </FieldSeparator>
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
                                <div class="flex items-center">
                                    <FieldLabel attr:r#for="password">"Password"</FieldLabel>
                                    <a
                                        href="#"
                                        class="ml-auto text-sm underline-offset-4 hover:underline"
                                    >
                                        "Forgot your password?"
                                    </a>
                                </div>
                                <Input
                                    attr:id="password"
                                    attr:r#type="password"
                                    attr:required=true
                                />
                            </Field>
                            <Field>
                                <Button attr:r#type="submit">"Login"</Button>
                                <FieldDescription class="text-center">
                                    "Don't have an account? " <a href="/signup">"Sign up"</a>
                                </FieldDescription>
                            </Field>
                        </FieldGroup>
                    </form>
                </CardContent>
            </Card>
            <FieldDescription class="px-6 text-center">
                "By clicking continue, you agree to our " <a href="#">"Terms of Service"</a> " and "
                <a href="#">"Privacy Policy"</a> "."
            </FieldDescription>
        </div>
    }
}

/// signup-03 page: brand mark over the signup card, centered on a muted background.
#[component]
pub fn SignupPage() -> impl IntoView {
    view! {
        <div class="bg-muted flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10">
            <div class="flex w-full max-w-sm flex-col gap-6">
                <a href="#" class="flex items-center gap-2 self-center font-medium">
                    <div class="bg-primary text-primary-foreground flex size-6 items-center justify-center rounded-md">
                        <Icon icon=icondata::LuGalleryVerticalEnd attr:class="size-4" />
                    </div>
                    "Acme Inc."
                </a>
                <SignupForm />
            </div>
        </div>
    }
}

/// The signup card: name, email, and a two-column password/confirm pair.
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
                "By clicking continue, you agree to our " <a href="#">"Terms of Service"</a> " and "
                <a href="#">"Privacy Policy"</a> "."
            </FieldDescription>
        </div>
    }
}
