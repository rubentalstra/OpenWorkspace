//! Field-encryption keyring: the unwrapped data keys held in memory for the
//! process lifetime, bootstrapped from `crypto_keys` under the configured root KEK.
//!
//! The root key-encryption key is supplied (base64) from configuration; it never
//! touches the database. Each per-purpose data key is generated once, wrapped
//! under the KEK, and stored in `crypto_keys`; on every later boot it is loaded
//! and unwrapped. Concurrent first-boot replicas race on the partial-unique index
//! and the loser re-reads the winner's key.

use base64::Engine as _;
use crypto::{DataKey, RootKey};
use db::Db;
use secrecy::{ExposeSecret, SecretString};

use crate::AuthError;

/// Logical purpose tag for the TOTP-secret data key in `crypto_keys`.
const TOTP_SECRET_PURPOSE: &str = "totp_secret";
/// Logical purpose tag for the OIDC-client-secret data key in `crypto_keys`.
const OIDC_CLIENT_SECRET_PURPOSE: &str = "oidc_client_secret";

/// In-memory field-encryption keys, unwrapped once at startup.
#[derive(Clone, Debug)]
pub struct FieldKeyring {
    totp_dek: DataKey,
    oidc_dek: DataKey,
}

impl FieldKeyring {
    /// Load — or, on first boot, create — the field keys, unwrapping them under
    /// the base64 root KEK from configuration.
    ///
    /// # Errors
    ///
    /// [`AuthError::Config`] if the root KEK is absent or not 32 bytes of base64;
    /// [`AuthError::Db`]/[`AuthError::Crypto`] on storage or unwrap failure.
    pub async fn load(db: &Db, root_key_base64: &SecretString) -> Result<Self, AuthError> {
        let kek = decode_root_key(root_key_base64)?;
        let totp_dek = load_or_create_dek(db, &kek, TOTP_SECRET_PURPOSE).await?;
        let oidc_dek = load_or_create_dek(db, &kek, OIDC_CLIENT_SECRET_PURPOSE).await?;
        Ok(Self { totp_dek, oidc_dek })
    }

    /// The data key that seals TOTP secrets.
    #[must_use]
    pub fn totp_dek(&self) -> &DataKey {
        &self.totp_dek
    }

    /// The data key that seals OIDC client secrets (`oidc_providers.client_secret_encrypted`).
    #[must_use]
    pub fn oidc_dek(&self) -> &DataKey {
        &self.oidc_dek
    }
}

fn decode_root_key(base64_key: &SecretString) -> Result<RootKey, AuthError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_key.expose_secret())
        .map_err(|_| AuthError::Config)?;
    RootKey::from_bytes(&bytes).map_err(|_| AuthError::Config)
}

async fn load_or_create_dek(db: &Db, kek: &RootKey, purpose: &str) -> Result<DataKey, AuthError> {
    if let Some(row) = db::load_active_crypto_key(db, purpose).await? {
        return Ok(crypto::unwrap_key(kek, &row.wrapped_key)?);
    }

    let dek = crypto::generate_data_key()?;
    let wrapped = crypto::wrap_key(kek, &dek)?;
    match db::insert_crypto_key(db, purpose, crypto::SUITE_ID_AES_256_GCM_V1, &wrapped, None).await
    {
        Ok(_) => Ok(dek),
        Err(db::DbError::Conflict) => {
            // A concurrent replica created the key first; adopt the winner.
            let row = db::load_active_crypto_key(db, purpose)
                .await?
                .ok_or(AuthError::Config)?;
            Ok(crypto::unwrap_key(kek, &row.wrapped_key)?)
        }
        Err(other) => Err(AuthError::from(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev_kek_base64() -> SecretString {
        // 32 zero bytes, base64 — deterministic for the test only.
        SecretString::from(base64::engine::general_purpose::STANDARD.encode([0u8; 32]))
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn first_load_creates_then_reload_reuses(pool: Db) -> Result<(), AuthError> {
        let kek = dev_kek_base64();
        // First load creates and stores the wrapped DEK.
        let first = FieldKeyring::load(&pool, &kek).await?;
        // A TOTP secret sealed under the first DEK...
        let sealed = crypto::encrypt_field(first.totp_dek(), b"seed", b"totp_secret")?;

        // ...must decrypt under the DEK a second load unwraps from storage.
        let second = FieldKeyring::load(&pool, &kek).await?;
        let opened = crypto::decrypt_field(second.totp_dek(), &sealed, b"totp_secret")?;
        assert_eq!(opened, b"seed");
        Ok(())
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn empty_root_key_is_config_error(pool: Db) {
        let err = FieldKeyring::load(&pool, &SecretString::from(String::new()))
            .await
            .unwrap_err();
        assert!(matches!(err, AuthError::Config));
    }
}
