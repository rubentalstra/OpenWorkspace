use std::collections::HashSet;

use leptos::prelude::*;

/// A design parameter that can be locked to exclude it from randomization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockableParam {
    Style,
    BaseColor,
    Theme,
    IconLibrary,
    Font,
    MenuAccent,
    MenuColor,
    Radius,
}

impl LockableParam {
    /// Every param in canonical display order.
    pub const ALL: &'static [Self] = &[
        Self::Style,
        Self::BaseColor,
        Self::Theme,
        Self::IconLibrary,
        Self::Font,
        Self::MenuAccent,
        Self::MenuColor,
        Self::Radius,
    ];

    /// Human-readable label for the param.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Style => "Style",
            Self::BaseColor => "Base Color",
            Self::Theme => "Theme",
            Self::IconLibrary => "Icon Library",
            Self::Font => "Font",
            Self::MenuAccent => "Menu Accent",
            Self::MenuColor => "Menu Color",
            Self::Radius => "Radius",
        }
    }
}

/// Reactive context tracking which design params are locked against
/// randomization. Lives entirely in client memory; no browser APIs are touched,
/// so it is safe to construct during SSR.
///
/// Call [`UseLocks::init`] once at the page root, then read it with
/// [`use_locks`] in any descendant.
///
/// ```ignore
/// _ = UseLocks::init();
///
/// let locks = use_locks();
/// let is_locked = locks.is_locked(LockableParam::Font);
/// view! {
///     <button on:click=move |_| locks.toggle_lock(LockableParam::Font)>
///         {move || if is_locked.get() { "Locked" } else { "Unlocked" }}
///     </button>
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct UseLocks {
    locks: RwSignal<HashSet<LockableParam>>,
}

impl UseLocks {
    /// Construct the context with nothing locked and provide it to descendants.
    /// Returns the handle for immediate use at the call site.
    #[must_use]
    pub fn init() -> Self {
        let hook = Self {
            locks: RwSignal::new(HashSet::new()),
        };
        provide_context(hook);
        hook
    }

    /// Reactive signal that is `true` while `param` is locked.
    #[must_use]
    pub fn is_locked(&self, param: LockableParam) -> Signal<bool> {
        let locks = self.locks;
        Signal::derive(move || locks.with(|set| set.contains(&param)))
    }

    /// Reactive signal that is `true` while `param` is unlocked and therefore
    /// safe to randomize.
    #[must_use]
    pub fn can_randomize(&self, param: LockableParam) -> Signal<bool> {
        let locks = self.locks;
        Signal::derive(move || locks.with(|set| !set.contains(&param)))
    }

    /// Flip the lock state of `param`.
    pub fn toggle_lock(&self, param: LockableParam) {
        self.locks.update(|set| {
            if !set.remove(&param) {
                set.insert(param);
            }
        });
    }

    /// Lock `param`, excluding it from randomization.
    pub fn lock(&self, param: LockableParam) {
        self.locks.update(|set| {
            set.insert(param);
        });
    }

    /// Unlock `param`, allowing it to be randomized again.
    pub fn unlock(&self, param: LockableParam) {
        self.locks.update(|set| {
            set.remove(&param);
        });
    }

    /// Snapshot of every currently locked param. Subscribes the caller when
    /// invoked inside a reactive scope.
    #[must_use]
    pub fn locked_params(&self) -> HashSet<LockableParam> {
        self.locks.get()
    }
}

/// Access the [`UseLocks`] context established by [`UseLocks::init`].
///
/// # Panics
///
/// Panics if no ancestor called [`UseLocks::init`] first.
#[must_use]
pub fn use_locks() -> UseLocks {
    expect_context::<UseLocks>()
}
