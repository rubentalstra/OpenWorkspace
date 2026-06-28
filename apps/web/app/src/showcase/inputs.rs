use std::collections::HashSet;

use leptos::prelude::*;
use leptos_icons::Icon;
use ui::{
    Checkbox, Field, FieldContent, FieldDescription, FieldLabel, FieldLegend, FieldSet, FieldTitle,
    FieldVariant, Input, InputGroup, InputGroupAddon, InputGroupAddonAlign, InputGroupButton,
    InputGroupButtonSize, InputGroupInput, InputGroupText, InputGroupTextarea, Label, MultiSelect,
    MultiSelectContent, MultiSelectOption, MultiSelectTrigger, MultiSelectValue, RadioButton,
    RadioButtonGroup, RadioButtonText, RadioGroup, RadioGroupItem, Select, SelectContent,
    SelectGroup, SelectLabel, SelectNative, SelectOption, SelectTrigger, SelectValue, Slider,
    Switch, SwitchLabel, Textarea,
};

use super::{Demo, Page, Section};

/// Text fields, selects, checkboxes, switches, radios and sliders.
#[component]
pub fn InputsPage() -> impl IntoView {
    view! {
        <Page
            title="Inputs"
            subtitle="Text fields, selects, checkboxes, switches, radios and sliders."
        >
            <TextFieldSection />
            <InputGroupSection />
            <TextareaSection />
            <CheckboxSection />
            <SwitchSection />
            <RadioSection />
            <SliderSection />
            <SelectSection />
            <SelectNativeSection />
            <MultiSelectSection />
            <FieldSection />
        </Page>
    }
}

#[component]
fn TextFieldSection() -> impl IntoView {
    view! {
        <Section
            title="Text fields"
            description="The Input is a thin styled wrapper — set type, placeholder and bindings at the call site."
        >
            <Demo col=true>
                <div class="flex flex-col gap-2 w-full max-w-sm">
                    <Label r#for="email".to_string()>"Email"</Label>
                    <Input attr:id="email" attr:r#type="email" attr:placeholder="you@example.com" />
                </div>
                <div class="flex flex-col gap-2 w-full max-w-sm">
                    <Label r#for="password".to_string()>"Password"</Label>
                    <Input
                        attr:id="password"
                        attr:r#type="password"
                        attr:placeholder="\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}"
                    />
                </div>
            </Demo>
            <Demo label="States">
                <Input attr:placeholder="Default" class="max-w-[12rem]" />
                <Input attr:value="Disabled" attr:disabled=true class="max-w-[12rem]" />
                <Input attr:value="Read only" attr:readonly=true class="max-w-[12rem]" />
                <Input attr:placeholder="Invalid" attr:aria-invalid="true" class="max-w-[12rem]" />
            </Demo>
        </Section>
    }
}

#[component]
fn InputGroupSection() -> impl IntoView {
    view! {
        <Section
            title="Input groups"
            description="Fuse a control with leading or trailing icons, text addons and inline buttons."
        >
            <Demo label="Leading icon" col=true>
                <InputGroup class="max-w-sm">
                    <InputGroupAddon>
                        <Icon icon=icondata::LuSearch attr:class="size-4" />
                    </InputGroupAddon>
                    <InputGroupInput attr:placeholder="Search desks\u{2026}" />
                </InputGroup>
            </Demo>
            <Demo label="Text addons" col=true>
                <InputGroup class="max-w-sm">
                    <InputGroupAddon>
                        <InputGroupText>"https://"</InputGroupText>
                    </InputGroupAddon>
                    <InputGroupInput attr:placeholder="openworkspace.dev" />
                </InputGroup>
                <InputGroup class="max-w-sm">
                    <InputGroupInput attr:r#type="number" attr:placeholder="0.00" />
                    <InputGroupAddon align=InputGroupAddonAlign::InlineEnd>
                        <InputGroupText>"USD"</InputGroupText>
                    </InputGroupAddon>
                </InputGroup>
            </Demo>
            <Demo label="Inline button" col=true>
                <InputGroup class="max-w-sm">
                    <InputGroupInput attr:r#type="password" attr:value="ow_sk_live_8f2c9a" />
                    <InputGroupAddon align=InputGroupAddonAlign::InlineEnd>
                        <InputGroupButton
                            size=InputGroupButtonSize::IconXs
                            attr:r#type="button"
                            attr:aria-label="Reveal token"
                        >
                            <Icon icon=icondata::LuEyeOff attr:class="size-4" />
                        </InputGroupButton>
                        <InputGroupButton
                            size=InputGroupButtonSize::IconXs
                            attr:r#type="button"
                            attr:aria-label="Copy token"
                        >
                            <Icon icon=icondata::LuCopy attr:class="size-4" />
                        </InputGroupButton>
                    </InputGroupAddon>
                </InputGroup>
            </Demo>
            <Demo label="Block addon with textarea" col=true>
                <InputGroup class="max-w-sm">
                    <InputGroupTextarea attr:placeholder="Add a note\u{2026}" attr:rows=3 />
                    <InputGroupAddon align=InputGroupAddonAlign::BlockEnd>
                        <InputGroupText>"Markdown supported"</InputGroupText>
                        <InputGroupButton
                            size=InputGroupButtonSize::Sm
                            attr:r#type="button"
                            class="ml-auto"
                        >
                            "Send"
                        </InputGroupButton>
                    </InputGroupAddon>
                </InputGroup>
            </Demo>
        </Section>
    }
}

#[component]
fn TextareaSection() -> impl IntoView {
    view! {
        <Section title="Textarea" description="Multi-line free text, auto-sizing to its content.">
            <Demo col=true>
                <div class="flex flex-col gap-2 w-full max-w-sm">
                    <Label r#for="bio".to_string()>"Bio"</Label>
                    <Textarea
                        attr:id="bio"
                        attr:rows=4
                        attr:placeholder="Tell the team a little about yourself\u{2026}"
                    />
                </div>
                <Textarea
                    attr:value="This field is disabled."
                    attr:disabled=true
                    class="max-w-sm"
                />
            </Demo>
        </Section>
    }
}

#[component]
fn CheckboxSection() -> impl IntoView {
    let terms = RwSignal::new(true);
    let newsletter = RwSignal::new(false);

    view! {
        <Section
            title="Checkboxes"
            description="Controlled via a boolean signal; on_checked_change fires with the toggled value."
        >
            <Demo col=true>
                <Label class="gap-2">
                    <Checkbox
                        checked=terms
                        on_checked_change=Callback::new(move |v| terms.set(v))
                        aria_label="Accept terms"
                    />
                    "Accept terms and conditions"
                </Label>
                <Label class="gap-2">
                    <Checkbox
                        checked=newsletter
                        on_checked_change=Callback::new(move |v| newsletter.set(v))
                        aria_label="Subscribe to newsletter"
                    />
                    "Send me the monthly newsletter"
                </Label>
            </Demo>
            <Demo label="States">
                <Checkbox checked=Signal::derive(|| true) aria_label="Checked" />
                <Checkbox checked=Signal::derive(|| false) aria_label="Unchecked" />
                <Checkbox
                    checked=Signal::derive(|| true)
                    aria_label="Disabled checked"
                    attr:disabled=true
                />
                <Checkbox
                    checked=Signal::derive(|| false)
                    aria_label="Disabled unchecked"
                    attr:disabled=true
                />
            </Demo>
        </Section>
    }
}

#[component]
fn SwitchSection() -> impl IntoView {
    let wifi = RwSignal::new(true);
    let bluetooth = RwSignal::new(false);

    view! {
        <Section
            title="Switches"
            description="A button[role=switch] driven by aria-checked; toggle the state at the call site."
        >
            <Demo col=true>
                <Label class="gap-3">
                    <Switch
                        attr:aria-checked=move || wifi.get().to_string()
                        attr:aria-label="Wi-Fi"
                        on:click=move |_| {
                            wifi.update(|on| *on = !*on);
                        }
                    />
                    <SwitchLabel>"Wi-Fi"</SwitchLabel>
                </Label>
                <Label class="gap-3">
                    <Switch
                        attr:aria-checked=move || bluetooth.get().to_string()
                        attr:aria-label="Bluetooth"
                        on:click=move |_| {
                            bluetooth.update(|on| *on = !*on);
                        }
                    />
                    <SwitchLabel>"Bluetooth"</SwitchLabel>
                </Label>
            </Demo>
            <Demo label="States">
                <Switch attr:aria-checked="true" attr:aria-label="On" />
                <Switch attr:aria-checked="false" attr:aria-label="Off" />
                <Switch attr:aria-checked="true" attr:aria-label="Disabled on" attr:disabled=true />
                <Switch
                    attr:aria-checked="false"
                    attr:aria-label="Disabled off"
                    attr:disabled=true
                />
            </Demo>
        </Section>
    }
}

#[component]
fn RadioSection() -> impl IntoView {
    let plan = RwSignal::new("team".to_string());

    view! {
        <Section
            title="Radios"
            description="A single-select RadioGroup bound to a string signal, and the segmented RadioButtonGroup bar."
        >
            <Demo label="Radio group" col=true>
                <RadioGroup value=plan>
                    <Label class="gap-2">
                        <RadioGroupItem value="free" />
                        "Free"
                    </Label>
                    <Label class="gap-2">
                        <RadioGroupItem value="team" />
                        "Team"
                    </Label>
                    <Label class="gap-2">
                        <RadioGroupItem value="enterprise" />
                        "Enterprise"
                    </Label>
                </RadioGroup>
                <span class="text-sm text-muted-foreground">
                    {move || format!("Selected: {}", plan.get())}
                </span>
            </Demo>
            <Demo label="Radio button bar">
                <RadioButtonGroup>
                    <RadioButton>
                        <RadioButtonText>"Compact"</RadioButtonText>
                    </RadioButton>
                    <RadioButton>
                        <RadioButtonText>"Comfortable"</RadioButtonText>
                    </RadioButton>
                    <RadioButton>
                        <RadioButtonText>"Spacious"</RadioButtonText>
                    </RadioButton>
                </RadioButtonGroup>
            </Demo>
        </Section>
    }
}

#[component]
fn SliderSection() -> impl IntoView {
    let volume = RwSignal::new(40.0_f64);
    let zoom = RwSignal::new(2.0_f64);

    view! {
        <Section
            title="Slider"
            description="A native range input with a styled track; the position lives in an f64 signal."
        >
            <Demo col=true>
                <div class="flex flex-col gap-2 w-full max-w-sm">
                    <Label>{move || format!("Volume: {:.0}%", volume.get())}</Label>
                    <Slider value=volume />
                </div>
                <div class="flex flex-col gap-2 w-full max-w-sm">
                    <Label>{move || format!("Zoom: {:.1}x", zoom.get())}</Label>
                    <Slider value=zoom min=1.0 max=5.0 step=0.5 />
                </div>
                <div class="flex flex-col gap-2 w-full max-w-sm">
                    <Label>"Disabled"</Label>
                    <Slider value=RwSignal::new(70.0_f64) disabled=true />
                </div>
            </Demo>
        </Section>
    }
}

#[component]
fn SelectSection() -> impl IntoView {
    let choice = RwSignal::new(Option::<String>::None);
    let on_change = Callback::new(move |v: Option<String>| choice.set(v));

    view! {
        <Section
            title="Select"
            description="A custom JS-free listbox with keyboard navigation, grouped options and a controlled value."
        >
            <Demo col=true>
                <Select on_change=on_change class="w-[220px]">
                    <SelectTrigger>
                        <SelectValue placeholder="Pick a workspace" />
                    </SelectTrigger>
                    <SelectContent class="w-[220px]">
                        <SelectGroup>
                            <SelectLabel>"North America"</SelectLabel>
                            <SelectOption value="sf">"San Francisco"</SelectOption>
                            <SelectOption value="nyc">"New York"</SelectOption>
                        </SelectGroup>
                        <SelectGroup>
                            <SelectLabel>"Europe"</SelectLabel>
                            <SelectOption value="ams">"Amsterdam"</SelectOption>
                            <SelectOption value="lon">"London"</SelectOption>
                            <SelectOption value="ber">"Berlin"</SelectOption>
                        </SelectGroup>
                    </SelectContent>
                </Select>
                <span class="text-sm text-muted-foreground">
                    {move || match choice.get() {
                        Some(v) => format!("Selected: {v}"),
                        None => "Nothing selected".to_string(),
                    }}
                </span>
            </Demo>
            <Demo label="With default value">
                <Select default_value="team".to_string() class="w-[180px]">
                    <SelectTrigger>
                        <SelectValue placeholder="Plan" />
                    </SelectTrigger>
                    <SelectContent class="w-[180px]">
                        <SelectOption value="free">"Free"</SelectOption>
                        <SelectOption value="team">"Team"</SelectOption>
                        <SelectOption value="enterprise">"Enterprise"</SelectOption>
                    </SelectContent>
                </Select>
            </Demo>
        </Section>
    }
}

#[component]
fn SelectNativeSection() -> impl IntoView {
    let fruit = RwSignal::new("banana".to_string());

    view! {
        <Section
            title="Native select"
            description="A styled native <select> with a trailing chevron; bind the value at the call site."
        >
            <Demo col=true>
                <div class="flex flex-col gap-2 w-full max-w-xs">
                    <Label r#for="fruit".to_string()>"Favourite fruit"</Label>
                    <SelectNative
                        attr:id="fruit"
                        prop:value=move || fruit.get()
                        on:change=move |ev| fruit.set(event_target_value(&ev))
                    >
                        <option value="apple">"Apple"</option>
                        <option value="banana">"Banana"</option>
                        <option value="cherry">"Cherry"</option>
                        <option value="durian">"Durian"</option>
                    </SelectNative>
                    <span class="text-sm text-muted-foreground">
                        {move || format!("Chosen: {}", fruit.get())}
                    </span>
                </div>
                <SelectNative attr:disabled=true class="max-w-xs">
                    <option>"Disabled select"</option>
                </SelectNative>
            </Demo>
        </Section>
    }
}

#[component]
fn MultiSelectSection() -> impl IntoView {
    let amenities = RwSignal::new(HashSet::<String>::new());

    view! {
        <Section
            title="Multi-select"
            description="The same listbox parts, but each option toggles its membership in a shared set."
        >
            <Demo col=true>
                <MultiSelect values=amenities class="w-[240px]">
                    <MultiSelectTrigger>
                        <MultiSelectValue placeholder="Pick amenities" />
                    </MultiSelectTrigger>
                    <MultiSelectContent class="w-[240px]">
                        <MultiSelectOption value="monitor">"External monitor"</MultiSelectOption>
                        <MultiSelectOption value="dock">"USB-C dock"</MultiSelectOption>
                        <MultiSelectOption value="standing">"Standing desk"</MultiSelectOption>
                        <MultiSelectOption value="window">"Window seat"</MultiSelectOption>
                        <MultiSelectOption value="quiet">"Quiet zone"</MultiSelectOption>
                    </MultiSelectContent>
                </MultiSelect>
                <span class="text-sm text-muted-foreground">
                    {move || format!("{} selected", amenities.with(HashSet::len))}
                </span>
            </Demo>
        </Section>
    }
}

#[component]
fn FieldSection() -> impl IntoView {
    let notifications = RwSignal::new(true);

    view! {
        <Section
            title="Fields"
            description="FieldSet, FieldLegend, Field, FieldLabel, FieldContent and FieldDescription compose labelled controls."
        >
            <Demo col=true>
                <FieldSet class="max-w-md">
                    <FieldLegend>"Profile"</FieldLegend>
                    <Field variant=FieldVariant::Vertical>
                        <FieldLabel>
                            "Display name" <Input attr:placeholder="Ada Lovelace" />
                        </FieldLabel>
                        <FieldDescription>"Shown to everyone on your team."</FieldDescription>
                    </Field>
                    <Field variant=FieldVariant::Horizontal>
                        <Checkbox
                            checked=notifications
                            on_checked_change=Callback::new(move |v| notifications.set(v))
                            aria_label="Enable notifications"
                        />
                        <FieldContent>
                            <FieldTitle>"Email notifications"</FieldTitle>
                            <FieldDescription>
                                "Receive a summary when a booking changes."
                            </FieldDescription>
                        </FieldContent>
                    </Field>
                </FieldSet>
            </Demo>
        </Section>
    }
}
