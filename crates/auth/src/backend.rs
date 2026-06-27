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
pub enum AuthError {
    /// A database error.
    #[error(transparent)]
    Db(#[from] db::DbError),
    /// A password-hashing error (e.g. a malformed stored hash).
    #[error(transparent)]
    Crypto(#[from] crypto::CryptoError),
    /// The blocking verification task failed to join.
    #[error("password verification task failed")]
    Task,
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
