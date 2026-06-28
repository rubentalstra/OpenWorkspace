use crate::slot;
use leptos::prelude::*;

slot! {
    /// Label — shadcn Base UI `label`. Associate with a control via `attr:for` at
    /// the call site.
    Label, label, "label",
    "cn-label flex items-center select-none group-data-[disabled=true]:pointer-events-none peer-disabled:cursor-not-allowed"
}
