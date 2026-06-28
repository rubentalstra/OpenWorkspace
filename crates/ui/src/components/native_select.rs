use crate::{cn, slot};
use leptos::prelude::*;
use leptos_icons::Icon;

/// Native-select size.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum NativeSelectSize {
    #[default]
    Default,
    Sm,
}

impl NativeSelectSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
        }
    }
}

/// NativeSelect — shadcn Base UI `native-select`. A styled native `<select>` with
/// a chevron. Provide `<NativeSelectOption>`s as children; set `value`/`on:change`
/// at the call site.
#[component]
pub fn NativeSelect(
    #[prop(into, optional)] size: Signal<NativeSelectSize>,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Select>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            data-slot="native-select-wrapper"
            data-size=move || size.get().as_str()
            class=move || {
                cn!(
                    "cn-native-select-wrapper group/native-select relative w-fit has-[select:disabled]:opacity-50",
                    class.get(),
                )
            }
        >
            <select
                node_ref=node_ref
                data-slot="native-select"
                data-size=move || size.get().as_str()
                class="cn-native-select outline-none disabled:pointer-events-none disabled:cursor-not-allowed"
            >
                {children()}
            </select>
            <Icon
                icon=icondata::LuChevronDown
                attr:class="cn-native-select-icon pointer-events-none absolute select-none"
                attr:aria-hidden="true"
                attr:data-slot="native-select-icon"
            />
        </div>
    }
}

slot! { NativeSelectOption, option, "native-select-option", "bg-[Canvas] text-[CanvasText]" }
slot! {
    NativeSelectOptGroup, optgroup, "native-select-optgroup", "bg-[Canvas] text-[CanvasText]"
}
