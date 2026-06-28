use leptos::prelude::*;

/// Toggles a scroll lock on `<body>` driven by the returned signal.
///
/// Returns an [`RwSignal<bool>`] whose value mirrors the lock state: set it to
/// `true` to freeze page scrolling (sets `overflow: hidden` on `<body>`) and
/// `false` to release it. The reaction runs in an effect, so it is inert during
/// server rendering and only touches the DOM on the client. The inline
/// `overflow` value present before the first lock is captured once and restored
/// on release and on cleanup, so the hook leaves no trace.
pub fn use_lock_body_scroll(initial_locked: bool) -> RwSignal<bool> {
    let locked = RwSignal::new(initial_locked);
    let original = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        let Some(body) = document().body() else {
            return;
        };
        let style = body.style();

        if original.get_untracked().is_none() {
            original.set(Some(
                style.get_property_value("overflow").unwrap_or_default(),
            ));
        }

        if locked.get() {
            _ = style.set_property("overflow", "hidden");
        } else {
            restore_overflow(&style, original.get_untracked().as_deref());
        }
    });

    on_cleanup(move || {
        // The reactive owner is torn down during server rendering too, where the
        // DOM globals panic on the non-wasm target. Restoring `<body>` only makes
        // sense on the client, so the access is gated to the wasm target.
        #[cfg(target_arch = "wasm32")]
        if let Some(body) = document().body() {
            restore_overflow(&body.style(), original.get_untracked().as_deref());
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = original;
    });

    locked
}

/// Restores the captured inline `overflow`, removing the property when the
/// original was unset or empty.
fn restore_overflow(style: &web_sys::CssStyleDeclaration, original: Option<&str>) {
    match original {
        Some(value) if !value.is_empty() => {
            _ = style.set_property("overflow", value);
        }
        _ => {
            _ = style.remove_property("overflow");
        }
    }
}
