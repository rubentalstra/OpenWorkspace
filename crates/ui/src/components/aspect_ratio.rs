use crate::cn;
use leptos::prelude::*;

/// AspectRatio — shadcn Base UI `aspect-ratio`. Constrains children to the given
/// width/height `ratio` (e.g. `16.0 / 9.0`).
#[component]
pub fn AspectRatio(
    #[prop(into)] ratio: f64,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            data-slot="aspect-ratio"
            style=format!("--ratio: {ratio};")
            class=move || cn!("relative aspect-(--ratio)", class.get())
        >
            {children.map(|children| children())}
        </div>
    }
}
