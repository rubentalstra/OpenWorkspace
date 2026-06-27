//! TOTP second-factor facade over `totp-rs`.
//!
//! The shared secret is generated here, returned once (as a QR image, an otpauth
//! URL and a base32 string) for the user to enrol, and sealed via the `crypto`
//! facade for storage. Verification reconstructs the `TOTP` from the stored,
//! decrypted secret and parameters. The `totp-rs` types never leave this module.

use crypto::DataKey;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::AuthError;

/// Associated data binding the TOTP secret ciphertext to its column purpose.
const TOTP_AAD: &[u8] = b"totp_secret";
/// Verification skew: accept the adjacent ±1 step to tolerate clock drift.
const SKEW: u8 = 1;

/// A fresh TOTP enrolment: the sealed secret to persist plus the artefacts shown
/// to the user exactly once so they can add the account to their authenticator.
///
/// `Debug` is redacted: every field but the parameters reveals the secret.
#[derive(Clone)]
pub struct TotpEnrollment {
    /// AEAD-sealed shared secret to store.
    pub secret_encrypted: Vec<u8>,
    /// QR code as a base64-encoded PNG (`data:image/png;base64,…`).
    pub qr_png_base64: String,
    /// otpauth:// URL for manual/deep-link enrolment.
    pub otpauth_url: String,
    /// Base32 secret for manual key entry.
    pub secret_base32: String,
    /// Code length.
    pub digits: i16,
    /// Time step in seconds.
    pub period_seconds: i16,
    /// HMAC algorithm name.
    pub algorithm: String,
}

impl std::fmt::Debug for TotpEnrollment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TotpEnrollment")
            .field("secret_encrypted", &"<redacted>")
            .field("qr_png_base64", &"<redacted>")
            .field("otpauth_url", &"<redacted>")
            .field("secret_base32", &"<redacted>")
            .field("digits", &self.digits)
            .field("period_seconds", &self.period_seconds)
            .field("algorithm", &self.algorithm)
            .finish()
    }
}

/// A stored TOTP enrolment's fields, borrowed for verification.
#[derive(Clone, Copy)]
pub struct StoredTotp<'a> {
    /// AEAD-sealed shared secret.
    pub secret_encrypted: &'a [u8],
    /// Code length.
    pub digits: i16,
    /// Time step in seconds.
    pub period_seconds: i16,
    /// HMAC algorithm name.
    pub algorithm: &'a str,
}

/// TOTP enrolment and verification, parameterised by the relying issuer name and
/// the data key that seals secrets.
#[derive(Clone)]
pub struct TotpService {
    issuer: String,
    dek: DataKey,
}

impl TotpService {
    /// Build the service with the issuer name shown in authenticators and the
    /// TOTP-secret data key from the keyring.
    #[must_use]
    pub fn new(issuer: String, dek: DataKey) -> Self {
        Self { issuer, dek }
    }

    /// Begin enrolment: generate a secret, render its artefacts, and seal it.
    ///
    /// # Errors
    ///
    /// [`AuthError::Totp`] if secret generation, QR rendering, or `TOTP`
    /// construction fails; [`AuthError::Crypto`] if sealing fails.
    pub fn start_enrollment(&self, account_name: &str) -> Result<TotpEnrollment, AuthError> {
        let secret_bytes = Secret::generate_secret()
            .to_bytes()
            .map_err(|_| AuthError::Totp)?;
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            SKEW,
            30,
            secret_bytes.clone(),
            Some(self.issuer.clone()),
            account_name.to_owned(),
        )
        .map_err(|_| AuthError::Totp)?;

        let qr_png_base64 = totp.get_qr_base64().map_err(|_| AuthError::Totp)?;
        let otpauth_url = totp.get_url();
        let secret_base32 = totp.get_secret_base32();
        let secret_encrypted = crypto::encrypt_field(&self.dek, &secret_bytes, TOTP_AAD)?;

        Ok(TotpEnrollment {
            secret_encrypted,
            qr_png_base64,
            otpauth_url,
            secret_base32,
            digits: 6,
            period_seconds: 30,
            algorithm: "SHA1".to_owned(),
        })
    }

    /// Verify a code against the current system time (±[`SKEW`] steps).
    ///
    /// # Errors
    ///
    /// [`AuthError::Totp`] on a clock error or malformed stored enrolment;
    /// [`AuthError::Crypto`] if the stored secret cannot be opened.
    pub fn verify(&self, stored: &StoredTotp<'_>, code: &str) -> Result<bool, AuthError> {
        self.reconstruct(stored)?
            .check_current(code)
            .map_err(|_| AuthError::Totp)
    }

    /// Verify a code against an explicit Unix timestamp — used by tests for
    /// determinism.
    ///
    /// # Errors
    ///
    /// [`AuthError::Totp`] on a malformed stored enrolment; [`AuthError::Crypto`]
    /// if the stored secret cannot be opened.
    pub fn verify_at(
        &self,
        stored: &StoredTotp<'_>,
        code: &str,
        unix_time: u64,
    ) -> Result<bool, AuthError> {
        Ok(self.reconstruct(stored)?.check(code, unix_time))
    }

    fn reconstruct(&self, stored: &StoredTotp<'_>) -> Result<TOTP, AuthError> {
        let secret = crypto::decrypt_field(&self.dek, stored.secret_encrypted, TOTP_AAD)?;
        let algorithm = parse_algorithm(stored.algorithm)?;
        let digits = usize::try_from(stored.digits).map_err(|_| AuthError::Totp)?;
        let period = u64::try_from(stored.period_seconds).map_err(|_| AuthError::Totp)?;
        TOTP::new(
            algorithm,
            digits,
            SKEW,
            period,
            secret,
            Some(self.issuer.clone()),
            "account".to_owned(),
        )
        .map_err(|_| AuthError::Totp)
    }
}

fn parse_algorithm(name: &str) -> Result<Algorithm, AuthError> {
    match name {
        "SHA1" => Ok(Algorithm::SHA1),
        "SHA256" => Ok(Algorithm::SHA256),
        "SHA512" => Ok(Algorithm::SHA512),
        _ => Err(AuthError::Totp),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> TotpService {
        TotpService::new("OpenWorkspace".to_owned(), crypto::generate_data_key().unwrap())
    }

    #[test]
    fn enroll_then_verify_generated_code() {
        let svc = service();
        let enrollment = svc.start_enrollment("person@example.test").unwrap();
        assert_eq!(enrollment.digits, 6);
        assert!(enrollment.otpauth_url.starts_with("otpauth://totp/"));
        assert!(!enrollment.qr_png_base64.is_empty());

        let stored = StoredTotp {
            secret_encrypted: &enrollment.secret_encrypted,
            digits: enrollment.digits,
            period_seconds: enrollment.period_seconds,
            algorithm: &enrollment.algorithm,
        };

        // Generate the expected code at a fixed time and confirm it verifies there.
        let reconstructed = svc.reconstruct(&stored).unwrap();
        let at = 1_700_000_000u64;
        let code = reconstructed.generate(at);
        assert!(svc.verify_at(&stored, &code, at).unwrap());

        // Within ±1 step is accepted; far outside is not.
        assert!(svc.verify_at(&stored, &code, at + 30).unwrap());
        assert!(!svc.verify_at(&stored, &code, at + 600).unwrap());
        assert!(!svc.verify_at(&stored, "000000", at).unwrap() || code == "000000");
    }

    #[test]
    fn enrollment_debug_redacts_secret() {
        let enrollment = service().start_enrollment("a@b.test").unwrap();
        let dbg = format!("{enrollment:?}");
        assert!(!dbg.contains(&enrollment.secret_base32));
        assert!(dbg.contains("redacted"));
    }

    #[test]
    fn unknown_algorithm_is_rejected() {
        assert!(matches!(parse_algorithm("MD5"), Err(AuthError::Totp)));
    }
}
