//! Buttons, badges, toggles, kbd.

use leptos::prelude::*;
use ui::{
    Badge, BadgeVariant, Button, ButtonGroup, ButtonGroupText, ButtonSize, ButtonVariant, Kbd,
    KbdGroup, Toggle, ToggleGroup, ToggleGroupItem,
};

use super::{Demo, PageShell};

/// Buttons, badges, toggles, kbd.
#[component]
pub fn ButtonsPage() -> impl IntoView {
    let bold = RwSignal::new(false);
    let marks = RwSignal::new(vec!["bold".to_owned()]);
    view! {
        <PageShell title="Buttons" subtitle="Actions, badges, toggles.">
            <Demo title="Variants">
                <Button>"Default"</Button>
                <Button variant=ButtonVariant::Secondary>"Secondary"</Button>
                <Button variant=ButtonVariant::Outline>"Outline"</Button>
                <Button variant=ButtonVariant::Ghost>"Ghost"</Button>
                <Button variant=ButtonVariant::Destructive>"Destructive"</Button>
                <Button variant=ButtonVariant::Link>"Link"</Button>
            </Demo>
            <Demo title="Sizes">
                <Button size=ButtonSize::Xs>"Xs"</Button>
                <Button size=ButtonSize::Sm>"Sm"</Button>
                <Button size=ButtonSize::Default>"Default"</Button>
                <Button size=ButtonSize::Lg>"Lg"</Button>
            </Demo>
            <Demo title="Badges">
                <Badge>"Default"</Badge>
                <Badge variant=BadgeVariant::Secondary>"Secondary"</Badge>
                <Badge variant=BadgeVariant::Outline>"Outline"</Badge>
                <Badge variant=BadgeVariant::Destructive>"Destructive"</Badge>
            </Demo>
            <Demo title="Toggle + Kbd">
                <Toggle pressed=bold on_change=Callback::new(move |v| bold.set(v))>
                    "Bold"
                </Toggle>
                <KbdGroup>
                    <Kbd>"⌘"</Kbd>
                    <Kbd>"K"</Kbd>
                </KbdGroup>
            </Demo>
            <Demo title="Button group">
                <ButtonGroup>
                    <ButtonGroupText>"View"</ButtonGroupText>
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                        "Day"
                    </Button>
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                        "Week"
                    </Button>
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                        "Month"
                    </Button>
                </ButtonGroup>
            </Demo>
            <Demo title="Toggle group">
                <ToggleGroup value=marks on_change=Callback::new(move |v| marks.set(v))>
                    <ToggleGroupItem value="bold">"B"</ToggleGroupItem>
                    <ToggleGroupItem value="italic">"I"</ToggleGroupItem>
                    <ToggleGroupItem value="underline">"U"</ToggleGroupItem>
                </ToggleGroup>
            </Demo>
        </PageShell>
    }
}
