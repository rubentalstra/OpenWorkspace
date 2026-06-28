use leptos::prelude::*;
use ui::{
    Alert, AlertDescription, AlertTitle, Avatar, Badge, BadgeVariant, Button, ButtonSize,
    ButtonVariant, Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Checkbox,
    Input, Kbd, Label, Progress, Separator, Skeleton, Spinner, Switch, Textarea, ThemeToggle,
};

/// A gallery of the design-system components, exercising light and dark themes.
#[component]
pub fn UiShowcase() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-background text-foreground">
            <div class="flex flex-col gap-10 p-8 mx-auto max-w-4xl">
                <header class="flex justify-between items-center">
                    <h1 class="text-3xl font-bold tracking-tight">"OpenWorkspace UI"</h1>
                    <ThemeToggle />
                </header>

                <Section title="Buttons">
                    <div class="flex flex-wrap gap-3">
                        <Button>"Default"</Button>
                        <Button variant=ButtonVariant::Secondary>"Secondary"</Button>
                        <Button variant=ButtonVariant::Destructive>"Destructive"</Button>
                        <Button variant=ButtonVariant::Outline>"Outline"</Button>
                        <Button variant=ButtonVariant::Ghost>"Ghost"</Button>
                        <Button variant=ButtonVariant::Link>"Link"</Button>
                    </div>
                    <div class="flex flex-wrap gap-3 items-center">
                        <Button size=ButtonSize::Sm>"Small"</Button>
                        <Button size=ButtonSize::Default>"Default"</Button>
                        <Button size=ButtonSize::Lg>"Large"</Button>
                    </div>
                </Section>

                <Section title="Badges">
                    <div class="flex flex-wrap gap-2">
                        <Badge>"Default"</Badge>
                        <Badge variant=BadgeVariant::Secondary>"Secondary"</Badge>
                        <Badge variant=BadgeVariant::Destructive>"Destructive"</Badge>
                        <Badge variant=BadgeVariant::Outline>"Outline"</Badge>
                        <Badge variant=BadgeVariant::Success>"Success"</Badge>
                        <Badge variant=BadgeVariant::Warning>"Warning"</Badge>
                    </div>
                </Section>

                <Section title="Alert">
                    <Alert>
                        <AlertTitle>"Heads up!"</AlertTitle>
                        <AlertDescription>
                            "This is a description inside an alert."
                        </AlertDescription>
                    </Alert>
                </Section>

                <Section title="Card">
                    <Card class="max-w-sm">
                        <CardHeader>
                            <CardTitle>"Project"</CardTitle>
                            <CardDescription>"A short description of the card."</CardDescription>
                        </CardHeader>
                        <CardContent>"Card body content goes here."</CardContent>
                        <CardFooter>
                            <Button size=ButtonSize::Sm>"Action"</Button>
                        </CardFooter>
                    </Card>
                </Section>

                <Section title="Form controls">
                    <div class="flex flex-col gap-4 max-w-sm">
                        <div class="flex flex-col gap-1.5">
                            <Label r#for="email".to_string()>"Email"</Label>
                            <Input
                                attr:r#type="email"
                                attr:id="email"
                                attr:placeholder="you@example.com"
                            />
                        </div>
                        <div class="flex flex-col gap-1.5">
                            <Label>"Message"</Label>
                            <Textarea attr:placeholder="Type your message..." />
                        </div>
                        <label class="flex gap-2 items-center">
                            <Checkbox />
                            <span class="text-sm">"Accept terms"</span>
                        </label>
                        <label class="flex gap-2 items-center">
                            <Switch />
                            <span class="text-sm">"Enable notifications"</span>
                        </label>
                    </div>
                </Section>

                <Section title="Feedback & display">
                    <div class="flex flex-col gap-4">
                        <Progress value=60.0 />
                        <div class="flex gap-4 items-center">
                            <Spinner />
                            <Avatar>"AB"</Avatar>
                            <Kbd>"\u{2318}K"</Kbd>
                        </div>
                        <Separator />
                        <div class="flex flex-col gap-2">
                            <Skeleton class="w-48 h-4" />
                            <Skeleton class="w-32 h-4" />
                        </div>
                    </div>
                </Section>
            </div>
        </div>
    }
}

/// A titled section wrapper for the showcase.
#[component]
fn Section(#[prop(into)] title: String, children: Children) -> impl IntoView {
    view! {
        <section class="flex flex-col gap-4">
            <h2 class="text-sm font-medium tracking-wide uppercase text-muted-foreground">
                {title}
            </h2>
            {children()}
        </section>
    }
}
