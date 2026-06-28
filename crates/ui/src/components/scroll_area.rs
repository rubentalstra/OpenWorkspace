use crate::cn;
use leptos::prelude::*;

/// ScrollArea — shadcn Base UI `scroll-area`. A scroll container with a thin custom
/// scrollbar overlaying the content; the native scrollbar is hidden. The vertical
/// thumb tracks the viewport's scroll position (size and offset). Give the area a
/// bounded height via `class` (e.g. `h-72`) so its content can scroll.
#[component]
pub fn ScrollArea(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let viewport = NodeRef::<leptos::html::Div>::new();
    // Thumb geometry as percentages of the track (= viewport) height.
    let thumb_height = RwSignal::new(0.0_f64);
    let thumb_top = RwSignal::new(0.0_f64);
    let visible = RwSignal::new(false);

    let measure = move || {
        if let Some(el) = viewport.get_untracked() {
            let client = f64::from(el.client_height());
            let scroll = f64::from(el.scroll_height());
            let top = f64::from(el.scroll_top());
            if scroll > client && client > 0.0 {
                visible.set(true);
                thumb_height.set(client / scroll * 100.0);
                thumb_top.set(top / scroll * 100.0);
            } else {
                visible.set(false);
            }
        }
    };

    Effect::new(move |_| measure());

    view! {
        <div
            data-slot="scroll-area"
            class=move || cn!("cn-scroll-area relative overflow-hidden", class.get())
        >
            <div
                node_ref=viewport
                data-slot="scroll-area-viewport"
                class="cn-scroll-area-viewport size-full overflow-y-auto rounded-[inherit] outline-none [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
                on:scroll=move |_| measure()
            >
                {children()}
            </div>
            <div
                data-slot="scroll-area-scrollbar"
                data-orientation="vertical"
                class:hidden=move || !visible.get()
                class="cn-scroll-area-scrollbar absolute top-0 right-0 flex touch-none p-px transition-colors select-none"
            >
                <div
                    data-slot="scroll-area-thumb"
                    class="cn-scroll-area-thumb absolute right-px w-2 bg-border"
                    style:height=move || format!("{}%", thumb_height.get())
                    style:top=move || format!("{}%", thumb_top.get())
                ></div>
            </div>
        </div>
    }
}
