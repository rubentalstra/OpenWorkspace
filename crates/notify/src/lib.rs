//! Notification rendering — localized email strings over Mozilla Fluent.
//!
//! Server-side counterpart to the `i18n` crate (which serves the Leptos UI):
//! this loads the `locales/email` resources behind a first-party facade so call
//! sites never touch `fluent-templates` types. Languages and the `en` fallback
//! match the UI; the seven first-release languages live under `locales/email`.
//!
//! notify — Notifications: email rendering (askama), the one-way .ics (icalendar) and the outbox.

use std::borrow::Cow;
use std::collections::HashMap;

use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{Loader, static_loader};
use unic_langid::LanguageIdentifier;

static_loader! {
    static EMAIL_TRANSLATIONS = {
        locales: "../../locales/email",
        fallback_language: "en",
    };
}

/// An error producing a localized notification string.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The locale code was not a well-formed BCP-47 language identifier.
    #[error("invalid locale `{0}`")]
    InvalidLocale(String),
    /// No message with this key exists in the requested locale or the fallback.
    #[error("unknown message `{0}`")]
    UnknownMessage(String),
}

/// Resolves the email string `key` for `lang`, substituting `args` (Fluent
/// placeables, e.g. `("resource", "Desk A1")`).
///
/// `lang` is a BCP-47 code (`"en"`, `"nl"`, …). A well-formed but unsupported
/// locale falls back to English; a malformed one is an [`Error::InvalidLocale`].
/// A key missing from both the locale and the fallback is an
/// [`Error::UnknownMessage`].
pub fn translate(lang: &str, key: &str, args: &[(&str, &str)]) -> Result<String, Error> {
    let langid: LanguageIdentifier = lang
        .parse()
        .map_err(|_| Error::InvalidLocale(lang.to_owned()))?;

    let fluent_args: HashMap<Cow<'static, str>, FluentValue<'static>> = args
        .iter()
        .map(|(name, value)| {
            (
                Cow::Owned((*name).to_owned()),
                FluentValue::from((*value).to_owned()),
            )
        })
        .collect();

    EMAIL_TRANSLATIONS
        .try_lookup_with_args(&langid, key, &fluent_args)
        .ok_or_else(|| Error::UnknownMessage(key.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_localized_subject_with_args() {
        let subject = translate(
            "nl",
            "booking-confirmed-subject",
            &[("resource", "Desk A1")],
        )
        .expect("nl subject should render");

        assert!(subject.contains("Je reservering voor"), "got: {subject}");
        assert!(subject.contains("Desk A1"), "got: {subject}");
    }

    #[test]
    fn falls_back_to_english_for_unsupported_locale() {
        let subject = translate(
            "zh",
            "booking-confirmed-subject",
            &[("resource", "Desk A1")],
        )
        .expect("should fall back to en");
        assert!(subject.contains("Your booking for"), "got: {subject}");
    }

    #[test]
    fn unknown_message_is_an_error() {
        let result = translate("en", "no-such-key", &[]);
        assert!(matches!(result, Err(Error::UnknownMessage(_))));
    }
}
