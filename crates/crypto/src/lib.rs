//! Crypto-agile facade. P1 surfaces Argon2id password hashing behind intent-named
//! operations; argon2/password-hash types never appear in the public API.
//!
//! The stored value is a PHC string that self-describes its algorithm and cost,
//! so raising [`active_hasher`]'s parameters later still verifies old hashes —
//! that is the versioned-suite identifier (no custom header needed).

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString};
use argon2::{Argon2, Params};
use secrecy::{ExposeSecret, SecretString};

/// A PHC-format password hash (e.g. `$argon2id$v=19$m=19456,t=2,p=1$…`). Store this.
#[derive(Debug, Clone)]
pub struct PasswordHashString(String);

impl PasswordHashString {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for PasswordHashString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Result of verifying a password against a stored hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyOutcome {
    /// Matches; the stored hash already uses the active suite.
    Ok,
    /// Matches, but the stored hash is weaker than the active suite — the caller
    /// should re-hash the plaintext and store the new value (rehash-on-login).
    OkNeedsRehash,
    /// Does not match.
    Mismatch,
}

/// Crypto facade errors — never carries or leaks an argon2 type.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("password hashing failed")]
    Hash,
    #[error("stored password hash is malformed")]
    Parse,
}

/// The active Argon2 suite — the single place hashing cost is defined.
/// `Argon2::default()` is the OWASP minimum for Argon2id (m = 19 MiB, t = 2, p = 1).
fn active_hasher() -> Argon2<'static> {
    Argon2::default()
}

/// Hash a plaintext password into a storable PHC string.
pub fn hash_password(plaintext: &SecretString) -> Result<PasswordHashString, CryptoError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = active_hasher()
        .hash_password(plaintext.expose_secret().as_bytes(), &salt)
        .map_err(|_| CryptoError::Hash)?;
    Ok(PasswordHashString(hash.to_string()))
}

/// Verify a plaintext password against a stored PHC string.
pub fn verify_password(
    plaintext: &SecretString,
    stored: &PasswordHashString,
) -> Result<VerifyOutcome, CryptoError> {
    let parsed = PasswordHash::new(stored.as_str()).map_err(|_| CryptoError::Parse)?;
    match active_hasher().verify_password(plaintext.expose_secret().as_bytes(), &parsed) {
        Ok(()) if needs_rehash(&parsed) => Ok(VerifyOutcome::OkNeedsRehash),
        Ok(()) => Ok(VerifyOutcome::Ok),
        Err(argon2::password_hash::Error::Password) => Ok(VerifyOutcome::Mismatch),
        Err(_) => Err(CryptoError::Parse),
    }
}

/// Whether a stored hash's parameters are weaker than the active suite.
fn needs_rehash(parsed: &PasswordHash<'_>) -> bool {
    let Ok(stored) = Params::try_from(parsed) else {
        return true;
    };
    let active = Params::DEFAULT;
    stored.m_cost() < active.m_cost()
        || stored.t_cost() < active.t_cost()
        || stored.p_cost() < active.p_cost()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_ok() {
        let pw = SecretString::from("correct horse battery staple".to_owned());
        let hash = hash_password(&pw).unwrap();
        assert_eq!(verify_password(&pw, &hash).unwrap(), VerifyOutcome::Ok);
    }

    #[test]
    fn wrong_password_is_mismatch() {
        let pw = SecretString::from("right".to_owned());
        let wrong = SecretString::from("wrong".to_owned());
        let hash = hash_password(&pw).unwrap();
        assert_eq!(
            verify_password(&wrong, &hash).unwrap(),
            VerifyOutcome::Mismatch
        );
    }

    #[test]
    fn weaker_params_need_rehash() {
        let pw = SecretString::from("pw".to_owned());
        let weak = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(8 * 1024, 1, 1, None).unwrap(),
        );
        let salt = SaltString::generate(&mut OsRng);
        let phc = PasswordHashString(
            weak.hash_password(pw.expose_secret().as_bytes(), &salt)
                .unwrap()
                .to_string(),
        );
        assert_eq!(
            verify_password(&pw, &phc).unwrap(),
            VerifyOutcome::OkNeedsRehash
        );
    }
}
