//! Credential and bootstrap queries for local password authentication.
//!
//! This module owns the `sqlx`-mapped [`UserStatusRow`] and the compile-time
//! `query!`/`query_as!` the `auth` facade calls. Keeping the queries here (not in
//! `auth`) centralises the `.sqlx` offline cache in this crate. All error mapping
//! goes through the shared [`DbError`].

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Db, DbError};

/// Persistence-mapped mirror of the `user_status` enum. The `auth` facade
/// converts this to its own domain status; the vendor enum never leaks upward.
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatusRow {
    /// Active; may authenticate.
    Active,
    /// Suspended; rejected at login.
    Suspended,
    /// Deactivated; rejected at login.
    Deactivated,
}

/// A user's authentication record: the join of `users` and
/// `password_credentials` needed to verify a password and gate by status.
///
/// `Debug` is hand-written to redact the PHC hash so it never reaches logs.
#[derive(Clone)]
pub struct CredentialRow {
    /// `users.id`.
    pub id: Uuid,
    /// Lower-cased email (the login identifier).
    pub email: String,
    /// Account status — only [`UserStatusRow::Active`] may log in.
    pub status: UserStatusRow,
    /// Last successful login, if any.
    pub last_login_at: Option<DateTime<Utc>>,
    /// Whether the credential is flagged to force a password change before use.
    /// Surfaced to the `auth` facade; enforcement at login is deferred to the
    /// password-change phase.
    pub must_change: bool,
    /// Stored Argon2 PHC hash.
    pub password_hash: String,
}

impl std::fmt::Debug for CredentialRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialRow")
            .field("id", &self.id)
            .field("email", &self.email)
            .field("status", &self.status)
            .field("last_login_at", &self.last_login_at)
            .field("must_change", &self.must_change)
            .field("password_hash", &"<redacted>")
            .finish()
    }
}

/// Loads the credential record for `email` (case-insensitive via `citext`).
/// Returns `None` when no user, or the user has no password credential.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_credential_by_email(
    pool: &Db,
    email: &str,
) -> Result<Option<CredentialRow>, DbError> {
    let row = sqlx::query_as!(
        CredentialRow,
        r#"
        SELECT
            u.id,
            u.email::text AS "email!",
            u.status AS "status: UserStatusRow",
            u.last_login_at,
            pc.must_change,
            pc.password_hash
        FROM users u
        JOIN password_credentials pc ON pc.user_id = u.id
        WHERE u.email = $1::citext
        "#,
        email,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Loads the credential record for `user_id`. Returns `None` when no user, or the
/// user has no password credential.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_credential_by_id(
    pool: &Db,
    user_id: Uuid,
) -> Result<Option<CredentialRow>, DbError> {
    let row = sqlx::query_as!(
        CredentialRow,
        r#"
        SELECT
            u.id,
            u.email::text AS "email!",
            u.status AS "status: UserStatusRow",
            u.last_login_at,
            pc.must_change,
            pc.password_hash
        FROM users u
        JOIN password_credentials pc ON pc.user_id = u.id
        WHERE u.id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Replaces a user's stored password hash and stamps `password_changed_at = now()`
/// (used by rehash-on-login when the stored cost is below the active suite).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn update_password_hash(pool: &Db, user_id: Uuid, new_hash: &str) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        UPDATE password_credentials
        SET password_hash = $2, password_changed_at = now()
        WHERE user_id = $1
        "#,
        user_id,
        new_hash,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Stamps `users.last_login_at = now()` after a successful authentication.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn touch_last_login(pool: &Db, user_id: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE users SET last_login_at = now() WHERE id = $1"#,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Whether any instance admin already exists. Used to make bootstrap idempotent.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn instance_admin_exists(pool: &Db) -> Result<bool, DbError> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM users WHERE is_instance_admin) AS "exists!""#,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// Inserts a first-boot instance admin (`is_instance_admin = true`) together with
/// its password credential (`must_change = true`) in one transaction. A random
/// 16-byte `webauthn_user_handle` is generated to satisfy the NOT NULL/UNIQUE
/// column (passkeys land in P6). Returns the new user's id.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error (including a uniqueness clash on email
/// or handle).
pub async fn insert_bootstrap_admin(
    pool: &Db,
    email: &str,
    display_name: &str,
    password_hash: &str,
    webauthn_user_handle: &[u8],
) -> Result<Uuid, DbError> {
    let mut tx = pool.begin().await?;

    let user_id = sqlx::query_scalar!(
        r#"
        INSERT INTO users (email, display_name, webauthn_user_handle, is_instance_admin)
        VALUES ($1::citext, $2, $3, true)
        RETURNING id
        "#,
        email,
        display_name,
        webauthn_user_handle,
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO password_credentials (user_id, password_hash, must_change)
        VALUES ($1, $2, true)
        "#,
        user_id,
        password_hash,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(user_id)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::{CredentialRow, UserStatusRow};

    #[test]
    fn credential_row_debug_redacts_hash() {
        let secret_hash = "$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHQ$ZGlnZXN0ZGlnZXN0";
        let row = CredentialRow {
            id: Uuid::new_v4(),
            email: "person@example.test".to_owned(),
            status: UserStatusRow::Active,
            last_login_at: Some(Utc::now()),
            must_change: false,
            password_hash: secret_hash.to_owned(),
        };
        let dbg = format!("{row:?}");
        assert!(
            !dbg.contains(secret_hash),
            "Debug must not print the PHC hash"
        );
        assert!(
            dbg.contains("redacted"),
            "Debug must mark the field redacted"
        );
    }
}
