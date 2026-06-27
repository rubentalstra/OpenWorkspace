//! Crypto-agile facade. P1 surfaces Argon2id password hashing behind intent-named
//! operations; argon2/password-hash types never appear in the public API.
//!
//! The stored value is a PHC string that self-describes its algorithm and cost,
//! so raising [`active_hasher`]'s parameters later still verifies old hashes —
//! that is the versioned-suite identifier (no custom header needed).

use std::sync::LazyLock;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use secrecy::{ExposeSecret, SecretString};

mod field;

pub use field::{
    DataKey, RootKey, SUITE_ID_AES_256_GCM_V1, TokenHash, decrypt_field, encrypt_field,
    generate_data_key, hash_token, unwrap_key, wrap_key,
};

/// A PHC-format password hash (e.g. `$argon2id$v=19$m=19456,t=2,p=1$…`). Store this.
///
/// `Debug` is redacted so the PHC string (salt + digest) never reaches logs or
/// panic messages.
#[derive(Clone)]
pub struct PasswordHashString(String);

impl std::fmt::Debug for PasswordHashString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PasswordHashString")
            .field(&"<redacted>")
            .finish()
    }
}

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

/// Crypto facade errors — never carries or leaks a vendor crypto type.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CryptoError {
    #[error("password hashing failed")]
    Hash,
    #[error("stored password hash is malformed")]
    Parse,
    #[error("field encryption failed")]
    Encrypt,
    #[error("field decryption failed")]
    Decrypt,
    #[error("key wrap or unwrap failed")]
    KeyWrap,
    #[error("key has an invalid length")]
    KeyLength,
    #[error("secure random generation failed")]
    Rng,
}

/// The active Argon2 suite — the single place hashing cost is defined.
///
/// With no pepper this is `Argon2::default()`, the OWASP minimum for Argon2id
/// (m = 19 MiB, t = 2, p = 1). When a `pepper` is supplied it is bound in as the
/// keyed secret via [`Argon2::new_with_secret`], so a stolen database without the
/// pepper cannot be brute-forced offline. The pepper is *not* recorded in the PHC
/// string, so the same pepper must be supplied at verify time.
fn active_hasher(pepper: Option<&SecretString>) -> Result<Argon2<'_>, CryptoError> {
    match pepper {
        None => Ok(Argon2::default()),
        Some(secret) => Argon2::new_with_secret(
            secret.expose_secret().as_bytes(),
            Algorithm::Argon2id,
            Version::V0x13,
            Params::default(),
        )
        .map_err(|_| CryptoError::Hash),
    }
}

/// Hash a plaintext password into a storable PHC string, optionally peppered.
pub fn hash_password(
    plaintext: &SecretString,
    pepper: Option<&SecretString>,
) -> Result<PasswordHashString, CryptoError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = active_hasher(pepper)?
        .hash_password(plaintext.expose_secret().as_bytes(), &salt)
        .map_err(|_| CryptoError::Hash)?;
    Ok(PasswordHashString(hash.to_string()))
}

/// Verify a plaintext password against a stored PHC string, optionally peppered.
///
/// The same `pepper` (if any) used at hash time must be supplied; otherwise a
/// correct password reads as [`VerifyOutcome::Mismatch`].
pub fn verify_password(
    plaintext: &SecretString,
    stored: &PasswordHashString,
    pepper: Option<&SecretString>,
) -> Result<VerifyOutcome, CryptoError> {
    let parsed = PasswordHash::new(stored.as_str()).map_err(|_| CryptoError::Parse)?;
    match active_hasher(pepper)?.verify_password(plaintext.expose_secret().as_bytes(), &parsed) {
        Ok(()) if needs_rehash(&parsed) => Ok(VerifyOutcome::OkNeedsRehash),
        Ok(()) => Ok(VerifyOutcome::Ok),
        Err(argon2::password_hash::Error::Password) => Ok(VerifyOutcome::Mismatch),
        Err(_) => Err(CryptoError::Parse),
    }
}

/// A fixed plaintext fed into [`verify_dummy`]. Its value is irrelevant — the
/// result is always discarded; it exists only so the dummy verify does real work.
const DUMMY_PLAINTEXT: &[u8] = b"owk-dummy-verify-input";

/// A lazily-computed dummy PHC string at the active (default) suite parameters.
///
/// Computed once via [`LazyLock`]. Construction cannot panic: a default-params
/// Argon2id hash of a fixed input is infallible, but should the unreachable
/// failure path ever fire we fall back to a hand-written PHC at the same
/// parameters so [`verify_dummy`] still parses it and spends comparable time.
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    let salt = SaltString::generate(&mut OsRng);
    if let Ok(hash) = Argon2::default().hash_password(DUMMY_PLAINTEXT, &salt) {
        return hash.to_string();
    }
    // Unreachable in practice (default-params hashing of a fixed input is
    // infallible); a hand-written PHC at the same parameters keeps `verify_dummy`
    // parseable so it still spends comparable time.
    let p = Params::DEFAULT;
    format!(
        "$argon2id$v=19$m={},t={},p={}$\
         c29tZS1maXhlZC1zYWx0$Zm9yY2VkLWZhbGxiYWNrLWR1bW15LWRpZ2VzdA",
        p.m_cost(),
        p.t_cost(),
        p.p_cost(),
    )
});

/// Perform one Argon2 verify against an internal dummy hash at the active suite
/// parameters, discarding the result.
///
/// Callers on a credential-absent path (no user, no stored hash) invoke this so
/// that the work — and therefore the latency — matches the password-present
/// path, closing the user-enumeration timing oracle. The `pepper` is threaded
/// through so the hasher construction matches the real path's cost. The boolean
/// return exists only to defeat optimisation; it carries no security meaning.
pub fn verify_dummy(pepper: Option<&SecretString>) -> bool {
    let Ok(hasher) = active_hasher(pepper) else {
        return false;
    };
    let Ok(parsed) = PasswordHash::new(DUMMY_HASH.as_str()) else {
        return false;
    };
    hasher.verify_password(DUMMY_PLAINTEXT, &parsed).is_ok()
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
        let hash = hash_password(&pw, None).unwrap();
        assert_eq!(
            verify_password(&pw, &hash, None).unwrap(),
            VerifyOutcome::Ok
        );
    }

    #[test]
    fn wrong_password_is_mismatch() {
        let pw = SecretString::from("right".to_owned());
        let wrong = SecretString::from("wrong".to_owned());
        let hash = hash_password(&pw, None).unwrap();
        assert_eq!(
            verify_password(&wrong, &hash, None).unwrap(),
            VerifyOutcome::Mismatch
        );
    }

    #[test]
    fn weaker_params_need_rehash() {
        let pw = SecretString::from("pw".to_owned());
        let weak = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(8 * 1024, 1, 1, None).unwrap(),
        );
        let salt = SaltString::generate(&mut OsRng);
        let phc = PasswordHashString(
            weak.hash_password(pw.expose_secret().as_bytes(), &salt)
                .unwrap()
                .to_string(),
        );
        assert_eq!(
            verify_password(&pw, &phc, None).unwrap(),
            VerifyOutcome::OkNeedsRehash
        );
    }

    #[test]
    fn verify_dummy_does_real_argon2_work() {
        // The dummy verify must take on the order of a real Argon2 verify, not
        // return in ~no time — that is what closes the enumeration timing oracle.
        // Compare it against a real wrong-password verify at the same params.
        let pw = SecretString::from("anchor".to_owned());
        let hash = hash_password(&pw, None).unwrap();
        let wrong = SecretString::from("nope".to_owned());

        // Warm the lazy dummy hash so its one-time construction is not timed.
        let _ = verify_dummy(None);

        let n = 8;
        let real = {
            let start = std::time::Instant::now();
            for _ in 0..n {
                let _ = verify_password(&wrong, &hash, None).unwrap();
            }
            start.elapsed()
        };
        let dummy = {
            let start = std::time::Instant::now();
            for _ in 0..n {
                let _ = verify_dummy(None);
            }
            start.elapsed()
        };
        assert!(
            dummy.as_nanos() * 2 >= real.as_nanos(),
            "dummy verify ({dummy:?}) must do comparable work to a real verify ({real:?})"
        );
    }

    #[test]
    fn password_hash_string_debug_is_redacted() {
        let pw = SecretString::from("secret pw".to_owned());
        let hash = hash_password(&pw, None).unwrap();
        let raw = hash.as_str().to_owned();
        let dbg = format!("{hash:?}");
        assert!(!dbg.contains(&raw), "Debug must not print the PHC string");
        assert!(
            dbg.contains("redacted"),
            "Debug must mark the field redacted"
        );
    }

    #[test]
    fn peppered_hash_verifies_only_with_same_pepper() {
        let pw = SecretString::from("pw".to_owned());
        let pepper = SecretString::from("server-pepper".to_owned());
        let hash = hash_password(&pw, Some(&pepper)).unwrap();
        assert_eq!(
            verify_password(&pw, &hash, Some(&pepper)).unwrap(),
            VerifyOutcome::Ok
        );
        // The correct password without the pepper does not verify.
        assert_eq!(
            verify_password(&pw, &hash, None).unwrap(),
            VerifyOutcome::Mismatch
        );
        let wrong_pepper = SecretString::from("other-pepper".to_owned());
        assert_eq!(
            verify_password(&pw, &hash, Some(&wrong_pepper)).unwrap(),
            VerifyOutcome::Mismatch
        );
    }
}
