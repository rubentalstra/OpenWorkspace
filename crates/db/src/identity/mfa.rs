//! Multi-factor credential queries: passkeys, TOTP, recovery codes, plus the
//! instance MFA toggles the login and enrolment flows read.
//!
//! Vendor types never appear here. A passkey's serialized webauthn-rs `Passkey`
//! is carried as an opaque `serde_json::Value` (the `passkeys.passkey` jsonb
//! column); the `auth` facade owns the (de)serialization. TOTP secrets arrive
//! already sealed by the `crypto` facade and are stored as opaque bytes.

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::{Db, DbError, classify};

/// The instance-wide MFA toggles the login/enrolment flows consult, mirroring the
/// `instance_settings` singleton. Falls back to the column defaults when the
/// singleton row has not been created yet.
#[derive(Clone, Copy, Debug)]
pub struct MfaSettings {
    /// Whether passkeys may be registered and used.
    pub passkeys_enabled: bool,
    /// Whether TOTP may be enrolled and used.
    pub totp_enabled: bool,
    /// Whether every user must clear a second factor to sign in.
    pub require_mfa: bool,
}

impl Default for MfaSettings {
    fn default() -> Self {
        Self {
            passkeys_enabled: true,
            totp_enabled: true,
            require_mfa: false,
        }
    }
}

/// Load the instance MFA settings, or [`MfaSettings::default`] if unset.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_mfa_settings(pool: &Db) -> Result<MfaSettings, DbError> {
    let row = sqlx::query!(
        r#"
        SELECT passkeys_enabled, totp_enabled, require_mfa
        FROM instance_settings
        WHERE id = true
        "#,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map_or_else(MfaSettings::default, |r| MfaSettings {
        passkeys_enabled: r.passkeys_enabled,
        totp_enabled: r.totp_enabled,
        require_mfa: r.require_mfa,
    }))
}

/// The user fields the WebAuthn registration ceremony needs.
#[derive(Clone, Debug)]
pub struct WebauthnIdentity {
    /// Random opaque user handle bound into the credential (never the email).
    pub webauthn_user_handle: Vec<u8>,
    /// Login identifier, shown as the WebAuthn `user.name`.
    pub email: String,
    /// Shown as the WebAuthn `user.displayName`.
    pub display_name: String,
}

/// Load the WebAuthn identity for `user_id`. `None` if the user does not exist.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_webauthn_identity(
    pool: &Db,
    user_id: Uuid,
) -> Result<Option<WebauthnIdentity>, DbError> {
    let row = sqlx::query_as!(
        WebauthnIdentity,
        r#"
        SELECT webauthn_user_handle, email::text AS "email!", display_name
        FROM users
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Resolve the user owning a WebAuthn user handle — the discoverable-login
/// "identify" step. `None` if no user carries that handle.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_user_id_by_webauthn_handle(
    pool: &Db,
    handle: &[u8],
) -> Result<Option<Uuid>, DbError> {
    let id = sqlx::query_scalar!(
        r#"SELECT id FROM users WHERE webauthn_user_handle = $1"#,
        handle,
    )
    .fetch_optional(pool)
    .await?;
    Ok(id)
}

/// Resolve a user id by email (case-insensitive) without requiring a password
/// credential — the lookup for username-first passkey login, where the account
/// may be passwordless. `None` if no such user.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_user_id_by_email(pool: &Db, email: &str) -> Result<Option<Uuid>, DbError> {
    let id = sqlx::query_scalar!(r#"SELECT id FROM users WHERE email = $1::citext"#, email)
        .fetch_optional(pool)
        .await?;
    Ok(id)
}

/// A stored passkey with its serialized webauthn-rs `Passkey` and live counter.
#[derive(Clone, Debug)]
pub struct PasskeyRow {
    /// `passkeys.id`.
    pub id: Uuid,
    /// Owning user.
    pub user_id: Uuid,
    /// Raw credential id (globally unique).
    pub credential_id: Vec<u8>,
    /// Serialized webauthn-rs `Passkey` (COSE public key, flags, counter).
    pub passkey: JsonValue,
    /// Last persisted signature counter (clone-detection baseline).
    pub sign_count: i64,
}

/// Display metadata for a passkey — no key material.
#[derive(Clone, Debug)]
pub struct PasskeySummary {
    /// `passkeys.id`.
    pub id: Uuid,
    /// User-given label, if any.
    pub label: Option<String>,
    /// Live signature counter.
    pub sign_count: i64,
    /// Whether the authenticator reports the credential as backed up (synced).
    pub backup_state: bool,
    /// Registration time.
    pub created_at: DateTime<Utc>,
    /// Last successful authentication, if any.
    pub last_used_at: Option<DateTime<Utc>>,
}

/// The values needed to persist a freshly registered passkey.
#[derive(Clone, Debug)]
pub struct NewPasskey {
    /// Owning user.
    pub user_id: Uuid,
    /// Raw credential id (globally unique across all accounts).
    pub credential_id: Vec<u8>,
    /// Serialized webauthn-rs `Passkey`.
    pub passkey: JsonValue,
    /// Initial signature counter.
    pub sign_count: i64,
    /// Authenticator AAGUID, if attested.
    pub aaguid: Option<Uuid>,
    /// Reported transports (`internal`, `usb`, …).
    pub transports: Vec<String>,
    /// Whether the credential is eligible to be backed up.
    pub backup_eligible: bool,
    /// Whether the credential is currently backed up.
    pub backup_state: bool,
    /// Optional user-given label.
    pub label: Option<String>,
}

/// Insert a registered passkey. The globally-unique `credential_id` makes a
/// duplicate surface as [`DbError::Conflict`].
///
/// # Errors
///
/// [`DbError::Conflict`] on a duplicate credential id; [`DbError::Sqlx`] otherwise.
pub async fn insert_passkey(pool: &Db, new: &NewPasskey) -> Result<Uuid, DbError> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO passkeys
            (user_id, credential_id, passkey, sign_count, aaguid, transports, backup_eligible, backup_state, label)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
        new.user_id,
        new.credential_id,
        new.passkey,
        new.sign_count,
        new.aaguid,
        &new.transports,
        new.backup_eligible,
        new.backup_state,
        new.label.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(classify)?;
    Ok(id)
}

/// Load all of a user's passkeys (with key material) — the candidate set for
/// username-first authentication.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_passkeys_for_user(pool: &Db, user_id: Uuid) -> Result<Vec<PasskeyRow>, DbError> {
    let rows = sqlx::query_as!(
        PasskeyRow,
        r#"
        SELECT id, user_id, credential_id, passkey, sign_count
        FROM passkeys
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Load a single passkey by its credential id (discoverable login + counter update).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_passkey_by_credential_id(
    pool: &Db,
    credential_id: &[u8],
) -> Result<Option<PasskeyRow>, DbError> {
    let row = sqlx::query_as!(
        PasskeyRow,
        r#"
        SELECT id, user_id, credential_id, passkey, sign_count
        FROM passkeys
        WHERE credential_id = $1
        "#,
        credential_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Persist a passkey's updated state after a successful authentication: the
/// re-serialized credential, the advanced counter, and `last_used_at`.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn update_passkey_after_auth(
    pool: &Db,
    id: Uuid,
    passkey: &JsonValue,
    sign_count: i64,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        UPDATE passkeys
        SET passkey = $2, sign_count = $3, last_used_at = now()
        WHERE id = $1
        "#,
        id,
        passkey,
        sign_count,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// List a user's passkeys as display metadata, oldest first.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn list_passkeys(pool: &Db, user_id: Uuid) -> Result<Vec<PasskeySummary>, DbError> {
    let rows = sqlx::query_as!(
        PasskeySummary,
        r#"
        SELECT id, label, sign_count, backup_state, created_at, last_used_at
        FROM passkeys
        WHERE user_id = $1
        ORDER BY created_at
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Count a user's passkeys.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn count_passkeys(pool: &Db, user_id: Uuid) -> Result<i64, DbError> {
    let n = sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM passkeys WHERE user_id = $1"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(n)
}

/// Delete a passkey, scoped to its owner. Returns whether a row was removed.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn delete_passkey(pool: &Db, id: Uuid, user_id: Uuid) -> Result<bool, DbError> {
    let res = sqlx::query!(
        r#"DELETE FROM passkeys WHERE id = $1 AND user_id = $2"#,
        id,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// A stored TOTP enrolment. The secret is sealed by the `crypto` facade.
#[derive(Clone, Debug)]
pub struct TotpRow {
    /// Owning user.
    pub user_id: Uuid,
    /// AEAD-sealed shared secret.
    pub secret_encrypted: Vec<u8>,
    /// Code length.
    pub digits: i16,
    /// Time step in seconds.
    pub period_seconds: i16,
    /// HMAC algorithm name (`SHA1`/`SHA256`/`SHA512`).
    pub algorithm: String,
    /// When enrolment was confirmed; `None` while pending.
    pub confirmed_at: Option<DateTime<Utc>>,
}

/// Upsert a pending TOTP enrolment, replacing any prior one and resetting
/// confirmation (a re-enrolment must be re-verified).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn upsert_totp_pending(
    pool: &Db,
    user_id: Uuid,
    secret_encrypted: &[u8],
    digits: i16,
    period_seconds: i16,
    algorithm: &str,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        INSERT INTO totp_credentials (user_id, secret_encrypted, digits, period_seconds, algorithm, confirmed_at)
        VALUES ($1, $2, $3, $4, $5, NULL)
        ON CONFLICT (user_id) DO UPDATE SET
            secret_encrypted = EXCLUDED.secret_encrypted,
            digits           = EXCLUDED.digits,
            period_seconds   = EXCLUDED.period_seconds,
            algorithm        = EXCLUDED.algorithm,
            confirmed_at     = NULL,
            created_at       = now()
        "#,
        user_id,
        secret_encrypted,
        digits,
        period_seconds,
        algorithm,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Confirm a pending TOTP enrolment. Returns whether a pending row was confirmed.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn confirm_totp(pool: &Db, user_id: Uuid) -> Result<bool, DbError> {
    let res = sqlx::query!(
        r#"
        UPDATE totp_credentials
        SET confirmed_at = now()
        WHERE user_id = $1 AND confirmed_at IS NULL
        "#,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Load a user's TOTP enrolment (confirmed or pending).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_totp(pool: &Db, user_id: Uuid) -> Result<Option<TotpRow>, DbError> {
    let row = sqlx::query_as!(
        TotpRow,
        r#"
        SELECT user_id, secret_encrypted, digits, period_seconds, algorithm, confirmed_at
        FROM totp_credentials
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Load a user's *confirmed* TOTP enrolment, used at the login challenge.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_confirmed_totp(pool: &Db, user_id: Uuid) -> Result<Option<TotpRow>, DbError> {
    let row = sqlx::query_as!(
        TotpRow,
        r#"
        SELECT user_id, secret_encrypted, digits, period_seconds, algorithm, confirmed_at
        FROM totp_credentials
        WHERE user_id = $1 AND confirmed_at IS NOT NULL
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Whether the user has a confirmed TOTP enrolment (drives the MFA challenge).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn totp_is_confirmed(pool: &Db, user_id: Uuid) -> Result<bool, DbError> {
    let exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM totp_credentials WHERE user_id = $1 AND confirmed_at IS NOT NULL
        ) AS "exists!"
        "#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// Delete a user's TOTP enrolment.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn delete_totp(pool: &Db, user_id: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"DELETE FROM totp_credentials WHERE user_id = $1"#,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Replace a user's recovery codes with a fresh set of hashes (single transaction).
///
/// # Errors
///
/// [`DbError::Conflict`] on the (astronomically unlikely) hash collision with an
/// existing code; [`DbError::Sqlx`] otherwise.
pub async fn replace_recovery_codes(
    pool: &Db,
    user_id: Uuid,
    code_hashes: &[Vec<u8>],
) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query!(r#"DELETE FROM recovery_codes WHERE user_id = $1"#, user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        r#"
        INSERT INTO recovery_codes (user_id, code_hash)
        SELECT $1, h FROM UNNEST($2::bytea[]) AS h
        "#,
        user_id,
        code_hashes,
    )
    .execute(&mut *tx)
    .await
    .map_err(classify)?;
    tx.commit().await?;
    Ok(())
}

/// Consume one unused recovery code by its hash. Returns whether a code matched
/// and was marked used (single-use).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn consume_recovery_code(
    pool: &Db,
    user_id: Uuid,
    code_hash: &[u8],
) -> Result<bool, DbError> {
    let res = sqlx::query!(
        r#"
        UPDATE recovery_codes
        SET used_at = now()
        WHERE user_id = $1 AND code_hash = $2 AND used_at IS NULL
        "#,
        user_id,
        code_hash,
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Count a user's unused recovery codes.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn count_unused_recovery_codes(pool: &Db, user_id: Uuid) -> Result<i64, DbError> {
    let n = sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM recovery_codes WHERE user_id = $1 AND used_at IS NULL"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Runtime (non-macro) query: test-only seed data stays out of the offline
    // `.sqlx` cache, which `cargo sqlx prepare` builds from non-test targets only.
    async fn seed_user(pool: &Db, handle: &[u8]) -> Uuid {
        let email = format!("u{}@example.test", Uuid::new_v4().simple());
        sqlx::query_scalar(
            "INSERT INTO users (email, display_name, webauthn_user_handle) \
             VALUES ($1::citext, 'Test User', $2) RETURNING id",
        )
        .bind(email)
        .bind(handle)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn sample_passkey(user_id: Uuid, cred: &[u8]) -> NewPasskey {
        NewPasskey {
            user_id,
            credential_id: cred.to_vec(),
            passkey: serde_json::json!({"cred": "opaque"}),
            sign_count: 0,
            aaguid: Some(Uuid::nil()),
            transports: vec!["internal".to_owned()],
            backup_eligible: true,
            backup_state: false,
            label: Some("Phone".to_owned()),
        }
    }

    #[sqlx::test]
    async fn passkey_insert_load_update_delete(pool: Db) -> Result<(), DbError> {
        let user = seed_user(&pool, b"handle-aaaaaaaaa1").await;
        let id = insert_passkey(&pool, &sample_passkey(user, b"cred-1")).await?;

        let for_user = load_passkeys_for_user(&pool, user).await?;
        assert_eq!(for_user.len(), 1);
        assert_eq!(for_user[0].sign_count, 0);

        let by_cred = load_passkey_by_credential_id(&pool, b"cred-1")
            .await?
            .expect("found by credential id");
        assert_eq!(by_cred.id, id);

        update_passkey_after_auth(&pool, id, &serde_json::json!({"cred": "v2"}), 5).await?;
        let bumped = load_passkey_by_credential_id(&pool, b"cred-1")
            .await?
            .unwrap();
        assert_eq!(bumped.sign_count, 5);

        assert_eq!(count_passkeys(&pool, user).await?, 1);
        assert_eq!(list_passkeys(&pool, user).await?.len(), 1);

        assert!(delete_passkey(&pool, id, user).await?);
        assert!(!delete_passkey(&pool, id, user).await?);
        assert_eq!(count_passkeys(&pool, user).await?, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn duplicate_credential_id_conflicts(pool: Db) -> Result<(), DbError> {
        let a = seed_user(&pool, b"handle-aaaaaaaaa2").await;
        let b = seed_user(&pool, b"handle-bbbbbbbbb2").await;
        insert_passkey(&pool, &sample_passkey(a, b"shared-cred")).await?;
        let err = insert_passkey(&pool, &sample_passkey(b, b"shared-cred"))
            .await
            .unwrap_err();
        assert!(
            matches!(err, DbError::Conflict),
            "global credential id must be unique"
        );
        Ok(())
    }

    #[sqlx::test]
    async fn discoverable_handle_resolves_user(pool: Db) -> Result<(), DbError> {
        let handle = b"handle-disco-00001";
        let user = seed_user(&pool, handle).await;
        assert_eq!(
            load_user_id_by_webauthn_handle(&pool, handle).await?,
            Some(user)
        );
        assert_eq!(
            load_user_id_by_webauthn_handle(&pool, b"absent").await?,
            None
        );

        let ident = load_webauthn_identity(&pool, user)
            .await?
            .expect("identity");
        assert_eq!(ident.webauthn_user_handle, handle);
        assert!(ident.email.contains('@'));
        Ok(())
    }

    #[sqlx::test]
    async fn totp_enroll_confirm_lifecycle(pool: Db) -> Result<(), DbError> {
        let user = seed_user(&pool, b"handle-totp-000001").await;
        assert!(!totp_is_confirmed(&pool, user).await?);

        upsert_totp_pending(&pool, user, b"sealed-secret", 6, 30, "SHA1").await?;
        assert!(load_confirmed_totp(&pool, user).await?.is_none());
        assert!(load_totp(&pool, user).await?.is_some());

        assert!(confirm_totp(&pool, user).await?);
        assert!(
            !confirm_totp(&pool, user).await?,
            "second confirm is a no-op"
        );
        assert!(totp_is_confirmed(&pool, user).await?);

        let confirmed = load_confirmed_totp(&pool, user).await?.expect("confirmed");
        assert_eq!(confirmed.secret_encrypted, b"sealed-secret");
        assert_eq!(confirmed.digits, 6);

        delete_totp(&pool, user).await?;
        assert!(!totp_is_confirmed(&pool, user).await?);
        Ok(())
    }

    #[sqlx::test]
    async fn recovery_codes_replace_and_single_use(pool: Db) -> Result<(), DbError> {
        let user = seed_user(&pool, b"handle-recov-00001").await;
        let hashes = vec![vec![1u8; 32], vec![2u8; 32], vec![3u8; 32]];
        replace_recovery_codes(&pool, user, &hashes).await?;
        assert_eq!(count_unused_recovery_codes(&pool, user).await?, 3);

        assert!(consume_recovery_code(&pool, user, &[2u8; 32]).await?);
        assert!(
            !consume_recovery_code(&pool, user, &[2u8; 32]).await?,
            "single use"
        );
        assert!(
            !consume_recovery_code(&pool, user, &[9u8; 32]).await?,
            "unknown code"
        );
        assert_eq!(count_unused_recovery_codes(&pool, user).await?, 2);

        // Re-issuing replaces the prior set wholesale.
        replace_recovery_codes(&pool, user, &[vec![7u8; 32]]).await?;
        assert_eq!(count_unused_recovery_codes(&pool, user).await?, 1);
        Ok(())
    }

    #[sqlx::test]
    async fn recovery_code_hash_is_globally_unique(pool: Db) -> Result<(), DbError> {
        let a = seed_user(&pool, b"handle-uniq-000001").await;
        let b = seed_user(&pool, b"handle-uniq-000002").await;
        replace_recovery_codes(&pool, a, &[vec![5u8; 32]]).await?;
        let err = replace_recovery_codes(&pool, b, &[vec![5u8; 32]])
            .await
            .unwrap_err();
        assert!(
            matches!(err, DbError::Conflict),
            "code_hash unique index must hold"
        );
        Ok(())
    }

    #[sqlx::test]
    async fn mfa_settings_default_without_singleton(pool: Db) -> Result<(), DbError> {
        let s = load_mfa_settings(&pool).await?;
        assert!(s.passkeys_enabled && s.totp_enabled && !s.require_mfa);
        Ok(())
    }
}
