//! Password policy.
//!
//! The master spec (architecture plan §, `tenant_settings.min_password_length
//! integer NOT NULL DEFAULT 12 -- length over composition, breach-list checked`)
//! mandates a minimum length of 12 with **no** forced composition, following
//! current NIST guidance (favour length, do not require character classes).
//!
//! Two parts of the spec'd policy are deferred:
//!
//! - the configurable per-tenant `min_password_length` lives in a settings table
//!   that P5 does not yet wire up, so the floor is the constant
//!   [`MIN_PASSWORD_LENGTH`] (= the spec default of 12);
//! - the breach-list (have-i-been-pwned style) check is a later phase — it needs
//!   the breach corpus/integration that does not exist in P5.
//!
//! Enforcing length-only here is the safe subset: it never *weakens* the eventual
//! policy, and adding the breach check later only tightens it.

use secrecy::{ExposeSecret as _, SecretString};

/// Minimum password length. The spec's `min_password_length` default; used as a
/// fixed floor until the per-tenant setting is wired up.
pub const MIN_PASSWORD_LENGTH: usize = 12;

/// Why a candidate password was rejected. Carries no composition rules by design
/// (length over composition).
#[derive(Debug, thiserror::Error)]
pub enum PasswordPolicyError {
    /// The password is shorter than [`MIN_PASSWORD_LENGTH`] characters.
    #[error("password must be at least {MIN_PASSWORD_LENGTH} characters")]
    TooShort,
}

/// Validates a candidate password against the policy: length only, no forced
/// composition. Counts Unicode scalar values (`chars`), not bytes, so a password
/// of multi-byte characters is judged by visible length.
///
/// # Errors
///
/// Returns [`PasswordPolicyError::TooShort`] when the password has fewer than
/// [`MIN_PASSWORD_LENGTH`] characters.
pub fn validate_password(password: &SecretString) -> Result<(), PasswordPolicyError> {
    if password.expose_secret().chars().count() < MIN_PASSWORD_LENGTH {
        return Err(PasswordPolicyError::TooShort);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::{MIN_PASSWORD_LENGTH, PasswordPolicyError, validate_password};

    #[test]
    fn rejects_short_password() {
        let pw = SecretString::from("short".to_owned());
        assert!(matches!(
            validate_password(&pw).unwrap_err(),
            PasswordPolicyError::TooShort
        ));
    }

    #[test]
    fn accepts_long_password_without_composition() {
        // All lowercase, no digits/symbols — length over composition.
        let pw = SecretString::from("a".repeat(MIN_PASSWORD_LENGTH));
        assert!(validate_password(&pw).is_ok());
    }

    #[test]
    fn boundary_one_short_is_rejected() {
        let pw = SecretString::from("a".repeat(MIN_PASSWORD_LENGTH - 1));
        assert!(validate_password(&pw).is_err());
    }
}
