use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{
    Button, ButtonAction, ButtonGroup, ButtonGroupOrientation, ButtonGroupSeparator,
    ButtonGroupText, ButtonSize, ButtonVariant, ChipItem, ChipsContainer, Kbd, KbdGroup, Pressable,
    ToggleGroup, ToggleGroupItem, ToggleGroupSelection, ToggleGroupVariant, use_copy_clipboard,
};

use super::{Demo, Page, Section};

/// Buttons, segmented controls, press-and-hold confirmation, toggles and keys.
#[component]
pub fn ButtonsPage() -> impl IntoView {
    view! {
        <Page
            title="Buttons & actions"
            subtitle="Every button variant and size, plus the action affordances built on top of them."
        >
            <VariantsSection />
            <SizesSection />
            <StatesSection />
            <GroupSection />
            <ToggleSection />
            <ChipsSection />
            <ActionSection />
            <KeysSection />
        </Page>
    }
}

#[component]
fn VariantsSection() -> impl IntoView {
    view! {
        <Section title="Variants" description="The full palette of button intents.">
            <Demo>
                <Button>"Default"</Button>
                <Button variant=ButtonVariant::Secondary>"Secondary"</Button>
                <Button variant=ButtonVariant::Destructive>"Destructive"</Button>
                <Button variant=ButtonVariant::Outline>"Outline"</Button>
                <Button variant=ButtonVariant::Ghost>"Ghost"</Button>
                <Button variant=ButtonVariant::Accent>"Accent"</Button>
                <Button variant=ButtonVariant::Link>"Link"</Button>
                <Button variant=ButtonVariant::Warning>"Warning"</Button>
                <Button variant=ButtonVariant::Success>"Success"</Button>
                <Button variant=ButtonVariant::Bordered>"Bordered"</Button>
            </Demo>
        </Section>
    }
}

#[component]
fn SizesSection() -> impl IntoView {
    view! {
        <Section title="Sizes" description="Including icon-only and link buttons.">
            <Demo>
                <Button size=ButtonSize::Sm>"Small"</Button>
                <Button size=ButtonSize::Default>"Default"</Button>
                <Button size=ButtonSize::Lg>"Large"</Button>
                <Button size=ButtonSize::Icon attr:aria-label="Settings">
                    <Icon icon=icondata::LuSettings attr:class="size-4" />
                </Button>
                <Button variant=ButtonVariant::Outline>
                    <Icon icon=icondata::LuDownload attr:class="size-4" />
                    "With icon"
                </Button>
                <Button variant=ButtonVariant::Link href="/ui".to_string()>
                    "As a link"
                </Button>
            </Demo>
        </Section>
    }
}

#[component]
fn StatesSection() -> impl IntoView {
    let (copy, copied) = use_copy_clipboard(None);

    view! {
        <Section
            title="States & feedback"
            description="Disabled buttons, a copy-to-clipboard button and the press-feedback wrapper."
        >
            <Demo>
                <Button attr:disabled=true>"Disabled"</Button>
                <Button variant=ButtonVariant::Outline attr:disabled=true>
                    "Disabled outline"
                </Button>
                <Button variant=ButtonVariant::Outline on:click=move |_| copy("ow_sk_live_8f2c9a")>
                    <Icon icon=icondata::LuCopy attr:class="size-4" />
                    {move || if copied.get() { "Copied!" } else { "Copy API token" }}
                </Button>
                <Pressable>
                    <Button variant=ButtonVariant::Secondary>"Pressable (touch scale)"</Button>
                </Pressable>
            </Demo>
        </Section>
    }
}

#[component]
fn GroupSection() -> impl IntoView {
    view! {
        <Section
            title="Button groups"
            description="Segmented controls that fuse neighbouring buttons into one bar."
        >
            <Demo label="Horizontal">
                <ButtonGroup>
                    <Button variant=ButtonVariant::Outline>"Bold"</Button>
                    <Button variant=ButtonVariant::Outline>"Italic"</Button>
                    <Button variant=ButtonVariant::Outline>"Underline"</Button>
                </ButtonGroup>
            </Demo>
            <Demo label="With text & separator">
                <ButtonGroup>
                    <ButtonGroupText>"https://"</ButtonGroupText>
                    <Button variant=ButtonVariant::Outline>"openworkspace.dev"</Button>
                    <ButtonGroupSeparator />
                    <Button variant=ButtonVariant::Outline>
                        <Icon icon=icondata::LuCopy attr:class="size-4" />
                    </Button>
                </ButtonGroup>
            </Demo>
            <Demo label="Vertical">
                <ButtonGroup orientation=ButtonGroupOrientation::Vertical>
                    <Button variant=ButtonVariant::Outline>"Top"</Button>
                    <Button variant=ButtonVariant::Outline>"Middle"</Button>
                    <Button variant=ButtonVariant::Outline>"Bottom"</Button>
                </ButtonGroup>
            </Demo>
        </Section>
    }
}

#[component]
fn ToggleSection() -> impl IntoView {
    let alignment = RwSignal::new(vec!["left".to_string()]);
    let styles = RwSignal::new(vec!["bold".to_string()]);

    view! {
        <Section
            title="Toggle groups"
            description="Single-select (fused) and multi-select (outlined) toggle bars."
        >
            <Demo label="Single select">
                <ToggleGroup value=alignment selection=ToggleGroupSelection::Single spacing=0>
                    <ToggleGroupItem value="left" attr:aria-label="Align left">
                        <Icon icon=icondata::LuAlignLeft attr:class="size-4" />
                    </ToggleGroupItem>
                    <ToggleGroupItem value="center" attr:aria-label="Align center">
                        <Icon icon=icondata::LuAlignCenter attr:class="size-4" />
                    </ToggleGroupItem>
                    <ToggleGroupItem value="right" attr:aria-label="Align right">
                        <Icon icon=icondata::LuAlignRight attr:class="size-4" />
                    </ToggleGroupItem>
                </ToggleGroup>
            </Demo>
            <Demo label="Multi select">
                <ToggleGroup
                    value=styles
                    selection=ToggleGroupSelection::Multiple
                    variant=ToggleGroupVariant::Outline
                    spacing=0
                >
                    <ToggleGroupItem value="bold" attr:aria-label="Bold">
                        <Icon icon=icondata::LuBold attr:class="size-4" />
                    </ToggleGroupItem>
                    <ToggleGroupItem value="italic" attr:aria-label="Italic">
                        <Icon icon=icondata::LuItalic attr:class="size-4" />
                    </ToggleGroupItem>
                    <ToggleGroupItem value="underline" attr:aria-label="Underline">
                        <Icon icon=icondata::LuUnderline attr:class="size-4" />
                    </ToggleGroupItem>
                </ToggleGroup>
            </Demo>
        </Section>
    }
}

#[component]
fn ChipsSection() -> impl IntoView {
    let options = ["Desks", "Rooms", "Parking", "Lockers", "Monitors"];
    let selected = RwSignal::new(vec!["Desks".to_string()]);

    let chips = options
        .into_iter()
        .map(|name| {
            let label = name.to_string();
            let read_name = name.to_string();
            let toggle_name = name.to_string();
            let is_selected = Signal::derive(move || selected.with(|s| s.contains(&read_name)));
            view! {
                <ChipItem
                    label=label
                    selected=is_selected
                    on:click=move |_| {
                        selected
                            .update(|s| {
                                if let Some(i) = s.iter().position(|x| x == &toggle_name) {
                                    s.remove(i);
                                } else {
                                    s.push(toggle_name.clone());
                                }
                            });
                    }
                />
            }
        })
        .collect_view();

    view! {
        <Section
            title="Chips"
            description="Selectable pills with a check mark on the active state."
        >
            <Demo>
                <ChipsContainer>{chips}</ChipsContainer>
            </Demo>
        </Section>
    }
}

#[component]
fn ActionSection() -> impl IntoView {
    let confirmations = RwSignal::new(0_u32);
    let on_complete = Callback::new(move |()| confirmations.update(|n| *n += 1));

    view! {
        <Section
            title="Press and hold"
            description="ButtonAction fires only after a sustained press; releasing early aborts."
        >
            <Demo>
                <ButtonAction on_complete=on_complete duration_ms=1500>
                    <Icon icon=icondata::LuTrash2 attr:class="size-4" />
                    "Hold to delete"
                </ButtonAction>
                <span class="text-sm text-muted-foreground">
                    {move || format!("Confirmed {} time(s)", confirmations.get())}
                </span>
            </Demo>
        </Section>
    }
}

#[component]
fn KeysSection() -> impl IntoView {
    view! {
        <Section title="Keyboard keys" description="Inline key caps and shortcut groups.">
            <Demo>
                <KbdGroup>
                    <Kbd>"Ctrl"</Kbd>
                    <Kbd>"K"</Kbd>
                </KbdGroup>
                <KbdGroup>
                    <Kbd>"\u{2318}"</Kbd>
                    <Kbd>"\u{21E7}"</Kbd>
                    <Kbd>"P"</Kbd>
                </KbdGroup>
                <Kbd>"Esc"</Kbd>
            </Demo>
        </Section>
    }
}
