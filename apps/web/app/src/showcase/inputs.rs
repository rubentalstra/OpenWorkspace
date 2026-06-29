//! Form inputs.

use leptos::prelude::*;
use ui::{
    Checkbox, Field, FieldDescription, FieldLabel, Input, InputGroup, InputGroupAddon,
    InputGroupInput, InputGroupText, Label, NativeSelect, NativeSelectOption, RadioGroup,
    RadioGroupItem, Slider, Switch, Textarea,
};

use super::{Demo, PageShell};

/// Form inputs.
#[component]
pub fn InputsPage() -> impl IntoView {
    let checked = RwSignal::new(true);
    let switch_on = RwSignal::new(true);
    let vol = RwSignal::new(40.0);
    let plan = RwSignal::new("map".to_owned());
    view! {
        <PageShell title="Inputs" subtitle="Text, choice, and range controls.">
            <Demo title="Text field">
                <div class="flex w-full flex-col gap-2">
                    <Label attr:r#for="email">"Email"</Label>
                    <Input attr:id="email" attr:r#type="email" attr:placeholder="you@example.com" />
                    <Textarea attr:placeholder="Notes…" />
                </div>
            </Demo>
            <Demo title="Checkbox / Switch">
                <Checkbox checked=checked on_change=Callback::new(move |v| checked.set(v)) />
                <Switch checked=switch_on on_change=Callback::new(move |v| switch_on.set(v)) />
            </Demo>
            <Demo title="Slider">
                <Slider value=vol on_change=Callback::new(move |v| vol.set(v)) class="w-full" />
                <span class="text-muted-foreground text-sm tabular-nums">
                    {move || format!("{:.0}", vol.get())}
                </span>
            </Demo>
            <Demo title="Native select">
                <NativeSelect>
                    <NativeSelectOption>"Map"</NativeSelectOption>
                    <NativeSelectOption>"List"</NativeSelectOption>
                    <NativeSelectOption>"Calendar"</NativeSelectOption>
                </NativeSelect>
            </Demo>
            <Demo title="Radio group">
                <RadioGroup
                    value=plan
                    on_change=Callback::new(move |v| plan.set(v))
                    class="flex flex-col gap-2"
                >
                    <label class="flex items-center gap-2 text-sm">
                        <RadioGroupItem value="map" />
                        "Map"
                    </label>
                    <label class="flex items-center gap-2 text-sm">
                        <RadioGroupItem value="list" />
                        "List"
                    </label>
                </RadioGroup>
            </Demo>
            <Demo title="Field">
                <Field>
                    <FieldLabel>"Display name"</FieldLabel>
                    <Input attr:placeholder="Ada Lovelace" />
                    <FieldDescription>"Shown to your teammates."</FieldDescription>
                </Field>
            </Demo>
            <Demo title="Input group">
                <InputGroup>
                    <InputGroupAddon>
                        <InputGroupText>"@"</InputGroupText>
                    </InputGroupAddon>
                    <InputGroupInput attr:placeholder="username" />
                </InputGroup>
            </Demo>
        </PageShell>
    }
}
