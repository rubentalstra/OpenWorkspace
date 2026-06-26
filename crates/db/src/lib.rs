//! PostgreSQL access and migrations (sqlx), behind a thin facade.

use secrecy::{ExposeSecret, SecretString};
use sqlx::postgres::PgPoolOptions;

/// Connection pool handle. Clone-cheap (`Arc` inside); share into app state and the worker.
pub type Db = sqlx::PgPool;

/// Embedded, reversible migrations. Path is relative to this crate's root.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Database facade errors.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// A booking exclusion-constraint violation (Postgres `23P01`). The HTTP layer
    /// maps this to 409 Conflict — the single chokepoint for no-double-booking.
    #[error("resource is already booked for that time")]
    Conflict,
    /// A serialization failure (`40001`); the operation may be retried.
    #[error("transaction conflict; retry")]
    Retryable,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Connect to PostgreSQL and build a pool.
pub async fn connect(url: &SecretString, max_connections: u32) -> Result<Db, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(url.expose_secret())
        .await?;
    Ok(pool)
}

/// Apply all pending migrations. Advisory-locked, so concurrent web replicas
/// starting together cannot race; idempotent.
pub async fn run_migrations(db: &Db) -> Result<(), DbError> {
    MIGRATOR.run(db).await?;
    Ok(())
}

/// Readiness check: `Ok(())` if the database answers a trivial query.
pub async fn ping(db: &Db) -> Result<(), DbError> {
    sqlx::query("SELECT 1").execute(db).await?;
    Ok(())
}

/// Classify a raw sqlx error into a typed [`DbError`], mapping the Postgres
/// SQLSTATEs the application reacts to. Used at query call sites in later phases.
#[must_use]
pub fn classify(err: sqlx::Error) -> DbError {
    if let sqlx::Error::Database(ref db_err) = err {
        match db_err.code().as_deref() {
            Some("23P01") => return DbError::Conflict,
            Some("40001") => return DbError::Retryable,
            _ => {}
        }
    }
    DbError::Sqlx(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = false)]
    async fn migrations_apply_with_extensions_then_revert(pool: Db) -> Result<(), DbError> {
        run_migrations(&pool).await?;

        let extensions: Vec<String> = sqlx::query_scalar("SELECT extname FROM pg_extension")
            .fetch_all(&pool)
            .await?;
        for required in ["btree_gist", "pg_trgm", "citext"] {
            assert!(
                extensions.iter().any(|e| e == required),
                "missing extension {required}"
            );
        }

        // Roll all migrations back down to version 0.
        MIGRATOR.undo(&pool, 0).await?;
        Ok(())
    }
}
