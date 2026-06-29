//! The `axum-login` authentication backend over local password credentials.

use axum_login::{AuthUser, AuthnBackend, UserId};
use crypto::{PasswordHashString, VerifyOutcome};
use db::Db;
use secrecy::SecretString;
use uuid::Uuid;

use crate::{Credentials, User};

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id.as_uuid()
    }

    fn session_auth_hash(&self) -> &[u8] {
        // Tying the session to the stored hash means a password change rotates
        // `session_auth_hash` and invalidates every existing session.
        self.password_hash.as_bytes()
    }
}

/// Authentication backend: looks credentials up in Postgres and verifies the
/// password (off the async runtime) with the optional server pepper.
#[derive(Clone)]
pub struct Backend {
    db: Db,
    pepper: Option<SecretString>,
}

impl Backend {
    /// Builds a backend over `db`, optionally peppering password verification.
    #[must_use]
    pub fn new(db: Db, pepper: Option<SecretString>) -> Self {
        Self { db, pepper }
    }
}

/// Errors raised by the authentication backend. A bad password or unknown user
/// is **not** an error — it is `Ok(None)` (uniform failure, no enumeration).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AuthError {
    /// A database error.
    #[error(transparent)]
    Db(#[from] db::DbError),
    /// A password-hashing or field-encryption error (e.g. a malformed stored hash).
    #[error(transparent)]
    Crypto(#[from] crypto::CryptoError),
    /// The blocking verification task failed to join.
    #[error("password verification task failed")]
    Task,
    /// The supplied current password did not match the stored hash. Distinct from
    /// the `authenticate` flow (where a mismatch is `Ok(None)`): on a deliberate
    /// password change the caller already knows who the user is, so a wrong
    /// current password is a typed error the handler maps to 403 — never revealing
    /// which field was wrong.
    #[error("current password does not match")]
    WrongPassword,
    /// The targeted user has no password credential to change.
    #[error("no password credential for user")]
    NoCredential,
    /// A WebAuthn ceremony failed. The vendor cause is logged inside the facade
    /// and never surfaced, so the client learns nothing it can exploit.
    #[error("webauthn ceremony failed")]
    Webauthn,
    /// A TOTP operation failed (secret generation, parsing, or the system clock).
    #[error("totp operation failed")]
    Totp,
    /// The signature counter did not advance past the stored value — a possible
    /// cloned authenticator. The credential is rejected.
    #[error("credential counter regressed; possible cloned authenticator")]
    ClonedCredential,
    /// Required in-progress ceremony state was absent or expired from the session.
    #[error("no in-progress ceremony state")]
    CeremonyState,
    /// A stored credential blob failed to serialize or deserialize.
    #[error("credential serialization failed")]
    Serialization,
    /// Field-encryption or relying-party configuration was missing or invalid.
    #[error("authentication configuration is invalid")]
    Config,
    /// A session-store operation failed (id cycle, sign-in, CSRF rotation, or the
    /// pending-MFA marker). The vendor cause is logged inside the facade.
    #[error("session operation failed")]
    Session,
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let Some(row) = db::load_credential_by_email(&self.db, &creds.email).await? else {
            // Row-absent path: spend one Argon2 verify against an internal dummy
            // hash so the latency matches the password-present path, closing the
            // user-enumeration timing oracle. The result is discarded.
            let pepper = self.pepper.clone();
            tokio::task::spawn_blocking(move || {
                let _ = crypto::verify_dummy(pepper.as_ref());
            })
            .await
            .map_err(|_| AuthError::Task)?;
            return Ok(None);
        };

        let stored = PasswordHashString::from(row.password_hash.clone());
        let verify_password = creds.password.clone();
        let pepper = self.pepper.clone();

        // Argon2 verification is CPU-bound; keep it off the async runtime.
        let outcome = tokio::task::spawn_blocking(move || {
            crypto::verify_password(&verify_password, &stored, pepper.as_ref())
        })
        .await
        .map_err(|_| AuthError::Task)??;

        let user_hash = match outcome {
            VerifyOutcome::Mismatch => return Ok(None),
            VerifyOutcome::OkNeedsRehash => {
                self.rehash(row.id, &creds.password, &row.password_hash)
                    .await
            }
            VerifyOutcome::Ok => row.password_hash.clone(),
        };

        // Build the User from the hash actually persisted so `session_auth_hash`
        // matches what `get_user` reloads (see `rehash`).
        let user = User::from_row_with_hash(row, user_hash);
        // Status gating: only active accounts may authenticate. Checked after a
        // successful password match so the failure path stays uniform.
        if !user.is_active() {
            return Ok(None);
        }

        db::touch_last_login(&self.db, user.id.as_uuid()).await?;
        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let row = db::load_credential_by_id(&self.db, *user_id).await?;
        Ok(row.map(User::from))
    }
}

impl Backend {
    /// Changes a user's password after verifying the current one.
    ///
    /// Loads the stored hash, verifies `current` against it off the async runtime
    /// (peppered), and on a match hashes `new` off-runtime and persists it via
    /// [`db::change_password`] (which also clears `must_change`). Both Argon2
    /// operations run in `spawn_blocking` so the CPU-bound work never blocks the
    /// reactor.
    ///
    /// A wrong `current` password is [`AuthError::WrongPassword`] (not silent), so
    /// the handler can answer `403` without disclosing which field was wrong.
    /// Changing the stored hash rotates `session_auth_hash`, which invalidates
    /// every *other* live session on its next request; the caller is expected to
    /// re-bind the current session (cycle its id) so the active session survives.
    ///
    /// # Errors
    ///
    /// - [`AuthError::WrongPassword`] if `current` does not match.
    /// - [`AuthError::NoCredential`] if the user has no password credential.
    /// - [`AuthError::Crypto`] if the stored hash is malformed or hashing fails.
    /// - [`AuthError::Db`] on a database error.
    /// - [`AuthError::Task`] if a blocking crypto task fails to join.
    pub async fn change_password(
        &self,
        user_id: Uuid,
        current: &SecretString,
        new: &SecretString,
    ) -> Result<(), AuthError> {
        let Some(row) = db::load_credential_by_id(&self.db, user_id).await? else {
            return Err(AuthError::NoCredential);
        };

        let stored = PasswordHashString::from(row.password_hash);
        let verify_password = current.clone();
        let pepper = self.pepper.clone();
        let outcome = tokio::task::spawn_blocking(move || {
            crypto::verify_password(&verify_password, &stored, pepper.as_ref())
        })
        .await
        .map_err(|_| AuthError::Task)??;

        if outcome == VerifyOutcome::Mismatch {
            return Err(AuthError::WrongPassword);
        }

        let new_password = new.clone();
        let pepper = self.pepper.clone();
        let new_hash = tokio::task::spawn_blocking(move || {
            crypto::hash_password(&new_password, pepper.as_ref())
        })
        .await
        .map_err(|_| AuthError::Task)??;

        db::change_password(&self.db, user_id, new_hash.as_str()).await?;
        Ok(())
    }

    /// Best-effort rehash-on-login: re-hash the verified password at the active
    /// cost and persist it, returning the hash that is *actually* the DB state.
    ///
    /// On success that is the NEW hash; on any failure (hashing or the DB write)
    /// it is the unchanged `old_hash`. Returning the persisted hash lets the
    /// caller build the [`User`] — and therefore `session_auth_hash` — from a
    /// value that matches what a subsequent `get_user` reload will see, so the
    /// session is not silently invalidated on the next request. A failure here is
    /// logged but never fails the login.
    async fn rehash(&self, user_id: Uuid, password: &SecretString, old_hash: &str) -> String {
        let password = password.clone();
        let pepper = self.pepper.clone();
        let hashed =
            tokio::task::spawn_blocking(move || crypto::hash_password(&password, pepper.as_ref()))
                .await;

        let new_hash = match hashed {
            Ok(Ok(h)) => h,
            Ok(Err(err)) => {
                tracing::warn!(error = %err, "rehash-on-login: hashing failed; keeping old hash");
                return old_hash.to_owned();
            }
            Err(_) => {
                tracing::warn!("rehash-on-login: hashing task failed to join; keeping old hash");
                return old_hash.to_owned();
            }
        };

        if let Err(err) = db::update_password_hash(&self.db, user_id, new_hash.as_str()).await {
            tracing::warn!(error = %err, "rehash-on-login: persisting new hash failed; keeping old hash");
            return old_hash.to_owned();
        }
        new_hash.into_string()
    }
}

#[cfg(test)]
mod tests {
    use axum_login::AuthnBackend as _;
    use secrecy::SecretString;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::{AuthError, Backend};
    use crate::Credentials;

    /// Seed a user plus a password credential. `weak` selects a deliberately
    /// under-strength stored hash to exercise rehash-on-login. Returns the email.
    async fn seed_user(pool: &PgPool, status: &str, password: &str, weak: bool) -> String {
        let tag = Uuid::new_v4().simple().to_string();
        let email = format!("{tag}@example.test");
        let secret = SecretString::from(password.to_owned());
        let hash = if weak {
            weak_hash(&secret)
        } else {
            crypto::hash_password(&secret, None).unwrap().into_string()
        };

        let user_id: Uuid = sqlx::query_scalar(
            "INSERT INTO users (email, display_name, status, webauthn_user_handle) \
             VALUES ($1::citext, 'Test User', $2::user_status, $3) RETURNING id",
        )
        .bind(&email)
        .bind(status)
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .fetch_one(pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO password_credentials (user_id, password_hash) VALUES ($1, $2)")
            .bind(user_id)
            .bind(&hash)
            .execute(pool)
            .await
            .unwrap();
        email
    }

    /// A deliberately weak Argon2id hash (params below the active suite).
    fn weak_hash(password: &SecretString) -> String {
        use argon2::password_hash::rand_core::OsRng;
        use argon2::password_hash::{PasswordHasher as _, SaltString};
        use argon2::{Algorithm, Argon2, Params, Version};
        use secrecy::ExposeSecret as _;

        let weak = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(8 * 1024, 1, 1, None).unwrap(),
        );
        let salt = SaltString::generate(&mut OsRng);
        weak.hash_password(password.expose_secret().as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    async fn user_id_for(pool: &PgPool, email: &str) -> Uuid {
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE email = $1::citext")
            .bind(email)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn login_success_sets_last_login(pool: PgPool) {
        let email = seed_user(&pool, "active", "correct horse", false).await;
        let user_id = user_id_for(&pool, &email).await;
        let backend = Backend::new(pool.clone(), None);

        let user = backend
            .authenticate(Credentials {
                email,
                password: SecretString::from("correct horse".to_owned()),
            })
            .await
            .unwrap()
            .expect("active user with correct password authenticates");
        assert_eq!(user.id.as_uuid(), user_id);

        let last_login: Option<chrono::DateTime<chrono::Utc>> =
            sqlx::query_scalar("SELECT last_login_at FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(last_login.is_some(), "last_login_at must be stamped");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn wrong_password_is_none(pool: PgPool) {
        let email = seed_user(&pool, "active", "correct horse", false).await;
        let backend = Backend::new(pool, None);
        let result = backend
            .authenticate(Credentials {
                email,
                password: SecretString::from("wrong".to_owned()),
            })
            .await
            .unwrap();
        assert!(result.is_none(), "wrong password must be Ok(None)");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn unknown_user_is_none(pool: PgPool) {
        let backend = Backend::new(pool, None);
        let result = backend
            .authenticate(Credentials {
                email: "nobody@example.test".to_owned(),
                password: SecretString::from("whatever".to_owned()),
            })
            .await
            .unwrap();
        assert!(result.is_none(), "unknown user must be Ok(None)");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn rehash_on_login_upgrades_hash(pool: PgPool) {
        let email = seed_user(&pool, "active", "weak pw", true).await;
        let user_id = user_id_for(&pool, &email).await;

        let before: (String, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
            "SELECT password_hash, password_changed_at FROM password_credentials WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        let backend = Backend::new(pool.clone(), None);
        let user = backend
            .authenticate(Credentials {
                email,
                password: SecretString::from("weak pw".to_owned()),
            })
            .await
            .unwrap()
            .expect("weak-hash user authenticates");
        assert_eq!(user.id.as_uuid(), user_id);

        let after: (String, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
            "SELECT password_hash, password_changed_at FROM password_credentials WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_ne!(before.0, after.0, "password hash must be upgraded");
        assert!(
            after.1 > before.1,
            "password_changed_at must advance on rehash"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn suspended_user_is_none(pool: PgPool) {
        let email = seed_user(&pool, "suspended", "correct horse", false).await;
        let backend = Backend::new(pool, None);
        let result = backend
            .authenticate(Credentials {
                email,
                password: SecretString::from("correct horse".to_owned()),
            })
            .await
            .unwrap();
        assert!(result.is_none(), "suspended user must be Ok(None)");
    }

    /// Median wall-clock latency (ns) of `n` `authenticate()` calls, used by the
    /// timing-oracle test.
    async fn median_authenticate_ns(backend: &Backend, email: &str, password: &str) -> u128 {
        let n = 7;
        let mut samples = Vec::with_capacity(n);
        for _ in 0..n {
            let start = std::time::Instant::now();
            let _ = backend
                .authenticate(Credentials {
                    email: email.to_owned(),
                    password: SecretString::from(password.to_owned()),
                })
                .await
                .unwrap();
            samples.push(start.elapsed().as_nanos());
        }
        samples.sort_unstable();
        samples[n / 2]
    }

    /// M0: the unknown-email path must spend Argon2 verify work comparable to the
    /// wrong-password path, so it cannot be used as a user-enumeration timing
    /// oracle. Asserts unknown-email median >= 50% of wrong-password median.
    #[sqlx::test(migrations = "../db/migrations")]
    async fn unknown_email_spends_verify_time(pool: PgPool) {
        let email = seed_user(&pool, "active", "correct horse", false).await;
        let backend = Backend::new(pool, None);

        // Warm any lazy state (the dummy hash) so it is not charged to the first
        // unknown-email sample.
        let _ = backend
            .authenticate(Credentials {
                email: "warmup@example.test".to_owned(),
                password: SecretString::from("x".to_owned()),
            })
            .await
            .unwrap();

        let wrong_pw = median_authenticate_ns(&backend, &email, "wrong password").await;
        let unknown =
            median_authenticate_ns(&backend, "ghost@example.test", "wrong password").await;

        assert!(
            unknown * 2 >= wrong_pw,
            "unknown-email median ({unknown} ns) must be >= 50% of wrong-password median \
             ({wrong_pw} ns) — the absent-row path must perform the dummy Argon2 verify"
        );
    }

    /// M1 unit: after rehash-on-login the session auth hash must equal the hash
    /// freshly loaded from the DB (constant-time compare), so the next request
    /// does not flush the session.
    #[sqlx::test(migrations = "../db/migrations")]
    async fn rehash_session_hash_matches_persisted(pool: PgPool) {
        use axum_login::AuthUser as _;
        use subtle::ConstantTimeEq as _;

        let email = seed_user(&pool, "active", "weak pw", true).await;
        let backend = Backend::new(pool.clone(), None);

        let user = backend
            .authenticate(Credentials {
                email,
                password: SecretString::from("weak pw".to_owned()),
            })
            .await
            .unwrap()
            .expect("weak-hash user authenticates");

        let reloaded = backend
            .get_user(&user.id.as_uuid())
            .await
            .unwrap()
            .expect("user reloads");

        let eq: bool = user
            .session_auth_hash()
            .ct_eq(reloaded.session_auth_hash())
            .into();
        assert!(
            eq,
            "post-rehash session auth hash must match the freshly-loaded DB hash"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn change_password_succeeds_and_clears_must_change(pool: PgPool) {
        let email = seed_user(&pool, "active", "old password", false).await;
        let user_id = user_id_for(&pool, &email).await;
        // Flag must_change so the success path is shown to clear it.
        sqlx::query("UPDATE password_credentials SET must_change = true WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

        let before: (String, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
            "SELECT password_hash, password_changed_at FROM password_credentials WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        let backend = Backend::new(pool.clone(), None);
        backend
            .change_password(
                user_id,
                &SecretString::from("old password".to_owned()),
                &SecretString::from("a brand new long password".to_owned()),
            )
            .await
            .expect("correct current password changes the password");

        let after: (String, chrono::DateTime<chrono::Utc>, bool) = sqlx::query_as(
            "SELECT password_hash, password_changed_at, must_change \
             FROM password_credentials WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_ne!(before.0, after.0, "stored hash must change");
        assert!(after.1 > before.1, "password_changed_at must advance");
        assert!(!after.2, "must_change must be cleared");

        // The new password authenticates; the old one no longer does.
        assert!(
            backend
                .authenticate(Credentials {
                    email: email.clone(),
                    password: SecretString::from("a brand new long password".to_owned()),
                })
                .await
                .unwrap()
                .is_some(),
            "new password authenticates"
        );
        assert!(
            backend
                .authenticate(Credentials {
                    email,
                    password: SecretString::from("old password".to_owned()),
                })
                .await
                .unwrap()
                .is_none(),
            "old password no longer authenticates"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn change_password_wrong_current_is_wrong_password(pool: PgPool) {
        let email = seed_user(&pool, "active", "old password", false).await;
        let user_id = user_id_for(&pool, &email).await;

        let before: String =
            sqlx::query_scalar("SELECT password_hash FROM password_credentials WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        let backend = Backend::new(pool.clone(), None);
        let err = backend
            .change_password(
                user_id,
                &SecretString::from("not the password".to_owned()),
                &SecretString::from("a brand new long password".to_owned()),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AuthError::WrongPassword), "got {err:?}");

        let after: String =
            sqlx::query_scalar("SELECT password_hash FROM password_credentials WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            before, after,
            "a wrong current password must not change the hash"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn malformed_hash_surfaces_crypto_error(pool: PgPool) {
        let tag = Uuid::new_v4().simple().to_string();
        let email = format!("{tag}@example.test");
        let user_id: Uuid = sqlx::query_scalar(
            "INSERT INTO users (email, display_name, webauthn_user_handle) \
             VALUES ($1::citext, 'Broken', $2) RETURNING id",
        )
        .bind(&email)
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO password_credentials (user_id, password_hash) VALUES ($1, 'not-a-phc')",
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        let backend = Backend::new(pool, None);
        let err = backend
            .authenticate(Credentials {
                email,
                password: SecretString::from("whatever".to_owned()),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, AuthError::Crypto(_)), "got {err:?}");
    }
}
