//! Envelope key storage: the wrapped data-encryption keys held in `crypto_keys`.
//!
//! Each row is a per-purpose data key sealed under the root key-encryption key.
//! The `auth` facade unwraps it through the `crypto` facade; raw key bytes never
//! flow through this crate.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Db, DbError, classify};

/// A `crypto_keys` row: a wrapped per-purpose data key and its suite tag.
#[derive(Clone, Debug)]
pub struct CryptoKeyRow {
    /// `crypto_keys.id`.
    pub id: Uuid,
    /// Logical purpose (e.g. `totp_secret`).
    pub purpose: String,
    /// Versioned suite identifier of the wrapping/sealing algorithm.
    pub suite_id: String,
    /// The data key, sealed under the root KEK.
    pub wrapped_key: Vec<u8>,
    /// Optional label of the KEK that wrapped this key.
    pub kek_label: Option<String>,
    /// Row creation time.
    pub created_at: DateTime<Utc>,
}

/// Load the active key for `purpose`. At most one row is active per purpose
/// (enforced by the `crypto_keys_active_purpose` partial-unique index).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_active_crypto_key(
    pool: &Db,
    purpose: &str,
) -> Result<Option<CryptoKeyRow>, DbError> {
    let row = sqlx::query_as!(
        CryptoKeyRow,
        r#"
        SELECT id, purpose, suite_id, wrapped_key, kek_label, created_at
        FROM crypto_keys
        WHERE purpose = $1 AND active
        "#,
        purpose,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Insert a new active key for `purpose`. A concurrent second insert for the same
/// purpose loses the race and surfaces as [`DbError::Conflict`] (the partial-unique
/// index), letting the caller re-read the winning row.
///
/// # Errors
///
/// [`DbError::Conflict`] when an active key for `purpose` already exists;
/// [`DbError::Sqlx`] on any other database error.
pub async fn insert_crypto_key(
    pool: &Db,
    purpose: &str,
    suite_id: &str,
    wrapped_key: &[u8],
    kek_label: Option<&str>,
) -> Result<CryptoKeyRow, DbError> {
    let row = sqlx::query_as!(
        CryptoKeyRow,
        r#"
        INSERT INTO crypto_keys (purpose, suite_id, wrapped_key, kek_label)
        VALUES ($1, $2, $3, $4)
        RETURNING id, purpose, suite_id, wrapped_key, kek_label, created_at
        "#,
        purpose,
        suite_id,
        wrapped_key,
        kek_label,
    )
    .fetch_one(pool)
    .await
    .map_err(classify)?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn insert_then_load_active(pool: Db) -> Result<(), DbError> {
        let inserted = insert_crypto_key(
            &pool,
            "totp_secret",
            "aead:aes256-gcm:v1",
            &[1, 2, 3],
            Some("root"),
        )
        .await?;
        let loaded = load_active_crypto_key(&pool, "totp_secret")
            .await?
            .expect("active key present");
        assert_eq!(loaded.id, inserted.id);
        assert_eq!(loaded.wrapped_key, vec![1, 2, 3]);
        assert_eq!(loaded.suite_id, "aead:aes256-gcm:v1");
        Ok(())
    }

    #[sqlx::test]
    async fn second_active_key_for_purpose_conflicts(pool: Db) -> Result<(), DbError> {
        insert_crypto_key(&pool, "totp_secret", "s", &[1], None).await?;
        let err = insert_crypto_key(&pool, "totp_secret", "s", &[2], None)
            .await
            .unwrap_err();
        assert!(
            matches!(err, DbError::Conflict),
            "expected Conflict, got {err:?}"
        );
        Ok(())
    }

    #[sqlx::test]
    async fn missing_purpose_is_none(pool: Db) -> Result<(), DbError> {
        assert!(load_active_crypto_key(&pool, "nope").await?.is_none());
        Ok(())
    }
}
