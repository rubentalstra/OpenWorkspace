//! First-boot instance-admin bootstrap.

use config::AuthConfig;
use db::Db;
use rand::TryRngCore as _;

use crate::AuthError;

/// Bootstraps the first-boot instance admin from operator-supplied credentials.
///
/// Bootstrap runs only when **both** [`AuthConfig::bootstrap_admin_email`] and
/// [`AuthConfig::bootstrap_admin_password`] are set **and** no instance admin yet
/// exists: it hashes the operator-supplied password (peppered when configured),
/// inserts the admin, and flags the credential `must_change = true`. It is
/// idempotent — a second call (or a configured email when an admin already
/// exists) is a no-op.
///
/// If the email is set but the password is **not**, bootstrap is skipped and a
/// single `warn` is emitted telling the operator to set
/// `APP_AUTH__BOOTSTRAP_ADMIN_PASSWORD`. The password (and every other secret) is
/// **never** logged.
///
/// The `must_change` flag is persisted, but forced-change *enforcement* at login
/// is deferred to the password-change phase — see [`crate::User`].
///
/// # Errors
///
/// - [`AuthError::Db`] on a database error.
/// - [`AuthError::Crypto`] if hashing the password fails.
/// - [`AuthError::Task`] if generating the WebAuthn handle's randomness fails.
pub async fn bootstrap_admin(db: &Db, cfg: &AuthConfig) -> Result<(), AuthError> {
    let Some(email) = cfg.bootstrap_admin_email.as_deref() else {
        return Ok(());
    };
    let Some(password) = cfg.bootstrap_admin_password.as_ref() else {
        tracing::warn!(
            %email,
            "bootstrap_admin_email is set but no password; skipping bootstrap. \
             Set APP_AUTH__BOOTSTRAP_ADMIN_PASSWORD to create the instance admin"
        );
        return Ok(());
    };
    if db::instance_admin_exists(db).await? {
        return Ok(());
    }

    let password = password.clone();
    let pepper = cfg.argon2_pepper.clone();
    let hashed =
        tokio::task::spawn_blocking(move || crypto::hash_password(&password, pepper.as_ref()))
            .await
            .map_err(|_| AuthError::Task)??;

    let mut handle = [0u8; 16];
    rand::rngs::OsRng
        .try_fill_bytes(&mut handle)
        .map_err(|_| AuthError::Task)?;

    db::insert_bootstrap_admin(
        db,
        email,
        "Instance Administrator",
        hashed.as_str(),
        &handle,
    )
    .await?;

    tracing::warn!(
        %email,
        "bootstrapped instance admin; log in with the configured password and change it immediately"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use config::AuthConfig;
    use secrecy::SecretString;
    use sqlx::PgPool;

    use super::bootstrap_admin;

    fn cfg_with_credentials(email: &str, password: &str) -> AuthConfig {
        AuthConfig {
            bootstrap_admin_email: Some(email.to_owned()),
            bootstrap_admin_password: Some(SecretString::from(password.to_owned())),
            ..AuthConfig::default()
        }
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn bootstrap_with_password_is_idempotent(pool: PgPool) {
        let cfg = cfg_with_credentials("admin@example.test", "operator-chosen-pw");

        bootstrap_admin(&pool, &cfg).await.unwrap();
        let after_first: i64 =
            sqlx::query_scalar("SELECT count(*) FROM users WHERE is_instance_admin")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(after_first, 1, "first bootstrap creates exactly one admin");

        bootstrap_admin(&pool, &cfg).await.unwrap();
        let after_second: i64 =
            sqlx::query_scalar("SELECT count(*) FROM users WHERE is_instance_admin")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(after_second, 1, "second bootstrap is a no-op");

        let must_change: bool = sqlx::query_scalar(
            "SELECT pc.must_change FROM password_credentials pc \
             JOIN users u ON u.id = pc.user_id WHERE u.is_instance_admin",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(
            must_change,
            "bootstrapped admin must require a password change"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn bootstrap_noop_without_email(pool: PgPool) {
        bootstrap_admin(&pool, &AuthConfig::default())
            .await
            .unwrap();
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE is_instance_admin")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "no email configured ⇒ no admin created");
    }

    /// A `MakeWriter` over a shared buffer so a test can read back captured logs.
    #[derive(Clone)]
    struct BufWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl std::io::Write for BufWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if let Ok(mut guard) = self.0.lock() {
                guard.extend_from_slice(buf);
            }
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for BufWriter {
        type Writer = BufWriter;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn bootstrap_logs_no_secret(pool: PgPool) {
        use std::sync::{Arc, Mutex};

        let secret = "super-secret-operator-password-1234567890";
        let cfg = cfg_with_credentials("admin@example.test", secret);

        let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_writer(BufWriter(Arc::clone(&buf)))
            .with_max_level(tracing::Level::TRACE)
            .finish();

        {
            let _guard = tracing::subscriber::set_default(subscriber);
            bootstrap_admin(&pool, &cfg).await.unwrap();
        }

        let logged = String::from_utf8(buf.lock().expect("log buffer lock").clone()).unwrap();
        assert!(
            !logged.contains(secret),
            "the bootstrap password must never appear in logs; got: {logged}"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn bootstrap_noop_with_email_but_no_password(pool: PgPool) {
        let cfg = AuthConfig {
            bootstrap_admin_email: Some("admin@example.test".to_owned()),
            bootstrap_admin_password: None,
            ..AuthConfig::default()
        };
        bootstrap_admin(&pool, &cfg).await.unwrap();
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE is_instance_admin")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "email without password must not create an admin");
    }
}
