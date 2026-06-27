//! First-party internationalisation facade over `leptos-fluent` (Mozilla Fluent).
//!
//! Centralises the translation provider, the supported language set, and the
//! locale-resolution strategy, and re-exports the [`I18n`]/[`Language`] context
//! types. Components read the active locale via those types and translate with
//! `leptos_fluent`'s `tr!`/`move_tr!` macros.
//!
//! Those two macros are imported directly from `leptos_fluent` at the call site
//! rather than re-exported here: the compile-time `check_translations` pass (see
//! [`I18nProvider`]) does textual analysis of `tr!`/`move_tr!` calls and rejects
//! any re-export or alias of them. The swappable surface — provider, strategy,
//! language set — stays behind this facade regardless.

use fluent_templates::static_loader;
use leptos::prelude::*;
use leptos_fluent::leptos_fluent;

pub use leptos_fluent::{I18n, Language};

static_loader! {
    // Fluent resources for every supported language, embedded at compile time.
    pub static TRANSLATIONS = {
        locales: "../../locales/app",
        fallback_language: "en",
    };
}

/// Provides the [`I18n`] context to all descendants and wires locale resolution
/// and persistence.
///
/// Initial language is taken from the `ow-lang` cookie, then negotiated from the
/// `Accept-Language` request header (falling back to `en`). The active language
/// is persisted to the cookie and mirrored onto the `<html lang>` attribute. The
/// URL is never used for language selection.
#[component]
pub fn I18nProvider(children: Children) -> impl IntoView {
    leptos_fluent! {
        children: children(),
        translations: [TRANSLATIONS],
        locales: "../../locales/app",
        languages: "../../locales/languages.json5",
        default_language: "en",
        #[cfg(not(feature = "ssr"))]
        check_translations: "../../{apps/web/app,crates/ui}/src/**/*.rs",
        sync_html_tag_lang: true,
        cookie_name: "ow-lang",
        initial_language_from_cookie: true,
        initial_language_from_accept_language_header: true,
        set_language_to_cookie: true,
    }
}
