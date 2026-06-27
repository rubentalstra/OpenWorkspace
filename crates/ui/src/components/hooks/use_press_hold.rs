use std::time::Duration;

use leptos::prelude::*;

/// Frame cadence for the progress animation; ~60 fps.
const FRAME: Duration = Duration::from_millis(16);

/// Press-and-hold interaction handle returned by [`use_press_hold`].
///
/// `progress_signal` ramps from `0.0` to `1.0` while the pointer is held and
/// drains back toward `0.0` once released. `is_holding_signal` mirrors whether
/// a hold is currently active. Call [`UsePressHold::on_pointer_down`] and
/// [`UsePressHold::on_pointer_up`] from the corresponding element events.
#[derive(Clone, Copy)]
pub struct UsePressHold {
    /// Fill level in `0.0..=1.0`; rises while held, drains once released.
    pub progress_signal: RwSignal<f64>,
    /// Whether a hold is currently active.
    pub is_holding_signal: RwSignal<bool>,
    interval: StoredValue<Option<IntervalHandle>>,
    last_update: StoredValue<f64>,
    duration: f64,
    on_complete: Callback<()>,
    disabled: Signal<bool>,
}

impl UsePressHold {
    fn clear_interval(&self) {
        if let Some(handle) = self.interval.try_update_value(Option::take).flatten() {
            handle.clear();
        }
    }

    /// Begins filling progress. Ignored while `disabled` is set.
    pub fn on_pointer_down(&self) {
        if self.disabled.get() {
            return;
        }

        self.clear_interval();
        self.is_holding_signal.set(true);
        self.last_update.set_value(js_sys::Date::now());

        let progress_signal = self.progress_signal;
        let is_holding_signal = self.is_holding_signal;
        let interval = self.interval;
        let last_update = self.last_update;
        let duration = self.duration;
        let on_complete = self.on_complete;

        let tick = move || {
            if !is_holding_signal.get_untracked() {
                return;
            }

            let now = js_sys::Date::now();
            let delta = now - last_update.get_value();
            last_update.set_value(now);

            let next = (progress_signal.get_untracked() + delta / duration).min(1.0);
            progress_signal.set(next);

            if next >= 1.0 {
                on_complete.run(());
                if let Some(handle) = interval.try_update_value(Option::take).flatten() {
                    handle.clear();
                }
                is_holding_signal.set(false);
                progress_signal.set(0.0);
            }
        };

        if let Ok(handle) = set_interval_with_handle(tick, FRAME) {
            self.interval.set_value(Some(handle));
        }
    }

    /// Stops the hold and drains progress back toward zero. A no-op when
    /// progress is already empty.
    pub fn on_pointer_up(&self) {
        self.clear_interval();
        self.is_holding_signal.set(false);

        if self.progress_signal.get() <= 0.0 {
            return;
        }

        self.last_update.set_value(js_sys::Date::now());

        let progress_signal = self.progress_signal;
        let is_holding_signal = self.is_holding_signal;
        let interval = self.interval;
        let last_update = self.last_update;
        let duration = self.duration;

        let tick = move || {
            if is_holding_signal.get_untracked() {
                return;
            }

            let now = js_sys::Date::now();
            let delta = now - last_update.get_value();
            last_update.set_value(now);

            let next = (progress_signal.get_untracked() - delta / duration).max(0.0);
            progress_signal.set(next);

            if next <= 0.0
                && let Some(handle) = interval.try_update_value(Option::take).flatten()
            {
                handle.clear();
            }
        };

        if let Ok(handle) = set_interval_with_handle(tick, FRAME) {
            self.interval.set_value(Some(handle));
        }
    }
}

/// Press-and-hold interaction pattern over `duration_ms`.
///
/// Returns a [`UsePressHold`] whose progress fills while the pointer is held
/// and drains when released; `on_complete` fires once progress reaches `1.0`,
/// after which it resets. Any running timer is torn down on component cleanup,
/// so the interval never outlives the caller.
pub fn use_press_hold(
    duration_ms: u32,
    on_complete: Callback<()>,
    disabled: Signal<bool>,
) -> UsePressHold {
    let hold = UsePressHold {
        progress_signal: RwSignal::new(0.0),
        is_holding_signal: RwSignal::new(false),
        interval: StoredValue::new(None),
        last_update: StoredValue::new(0.0),
        duration: f64::from(duration_ms),
        on_complete,
        disabled,
    };

    let interval = hold.interval;
    on_cleanup(move || {
        if let Some(handle) = interval.try_update_value(Option::take).flatten() {
            handle.clear();
        }
    });

    hold
}
