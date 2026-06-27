//! First-party Postgres [`SessionStore`] over the workspace's sqlx 0.9 pool.
//!
//! This replaces `tower-sessions-sqlx-store` (which pins sqlx 0.8) with a small
//! store that runs session-row CRUD against `tower_sessions.session` using the
//! existing [`db::Db`] pool, so the workspace keeps a single `sqlx`. The wire
//! format matches the upstream store exactly so the existing migration and any
//! previously-written rows are reused unchanged:
//!
//! - the `id text` primary key holds [`Id::to_string()`] (URL-safe base64, no
//!   padding, of the `i128` little-endian bytes — 22 chars);
//! - the `data bytea` column holds `rmp_serde::to_vec(&record)` — the *whole*
//!   [`Record`] (id, data, `expiry_date`), not just the data map;
//! - the `expiry_date timestamptz` column mirrors `record.expiry_date` for the
//!   server-side expiry filter on load.
//!
//! No `sqlx` or `tower_sessions` type appears in this module's public surface
//! beyond the implemented trait; the store is constructed from a [`db::Db`].

use chrono::{DateTime, Utc};
use db::Db;
use tower_sessions::SessionStore;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store;

/// Bound on the create-collision retry loop. A 128-bit random id collision is
/// astronomically unlikely; a handful of attempts removes any realistic chance
/// while guaranteeing termination (upstream loops unbounded).
const MAX_CREATE_ATTEMPTS: u32 = 8;

/// A Postgres-backed [`SessionStore`] over the workspace's sqlx 0.9 pool.
#[derive(Clone, Debug)]
pub struct PgSessionStore {
    db: Db,
}

impl PgSessionStore {
    /// Builds a store over the shared [`db::Db`] pool. The schema is owned by the
    /// reversible `tower_sessions_session` migration, so there is nothing to
    /// migrate at construction.
    #[must_use]
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

/// Maps a sqlx error to the trait's [`session_store::Error::Backend`] without
/// leaking the sqlx type outward (its `Display` carries the cause).
fn backend(err: &sqlx::Error) -> session_store::Error {
    session_store::Error::Backend(err.to_string())
}

/// `rmp_serde`-encodes the whole record, matching the upstream wire format.
fn encode(record: &Record) -> session_store::Result<Vec<u8>> {
    rmp_serde::to_vec(record).map_err(|err| session_store::Error::Encode(err.to_string()))
}

/// Decodes a record body written by [`encode`].
fn decode(data: &[u8]) -> session_store::Result<Record> {
    rmp_serde::from_slice(data).map_err(|err| session_store::Error::Decode(err.to_string()))
}

/// Converts the record's `time::OffsetDateTime` expiry to the `chrono` type the
/// `db` pool maps `timestamptz` to (a saturating instant; the value is only used
/// for the server-side expiry comparison, the canonical copy rides in the body).
fn expiry_to_chrono(record: &Record) -> DateTime<Utc> {
    let nanos = record.expiry_date.unix_timestamp_nanos();
    DateTime::from_timestamp_nanos(i64::try_from(nanos).unwrap_or(i64::MAX))
}

#[async_trait::async_trait]
impl SessionStore for PgSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        // Insert, regenerating the id on a primary-key collision and retrying.
        // `ON CONFLICT (id) DO NOTHING` plus `rows_affected()` detects the clash
        // race-safely under the PK constraint, so no separate existence probe is
        // needed. Bounded so the loop always terminates.
        for _ in 0..MAX_CREATE_ATTEMPTS {
            let id = record.id.to_string();
            let data = encode(record)?;
            let expiry = expiry_to_chrono(record);
            let inserted = sqlx::query!(
                r#"
                INSERT INTO tower_sessions.session (id, data, expiry_date)
                VALUES ($1, $2, $3)
                ON CONFLICT (id) DO NOTHING
                "#,
                id,
                data,
                expiry,
            )
            .execute(&self.db)
            .await
            .map_err(|e| backend(&e))?
            .rows_affected();

            if inserted == 1 {
                return Ok(());
            }
            record.id = Id::default();
        }
        Err(session_store::Error::Backend(
            "exhausted session id collision retries".to_owned(),
        ))
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let id = record.id.to_string();
        let data = encode(record)?;
        let expiry = expiry_to_chrono(record);
        sqlx::query!(
            r#"
            INSERT INTO tower_sessions.session (id, data, expiry_date)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET data = excluded.data, expiry_date = excluded.expiry_date
            "#,
            id,
            data,
            expiry,
        )
        .execute(&self.db)
        .await
        .map_err(|e| backend(&e))?;
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let id = session_id.to_string();
        let row = sqlx::query!(
            r#"
            SELECT data FROM tower_sessions.session
            WHERE id = $1 AND expiry_date > now()
            "#,
            id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| backend(&e))?;

        row.map(|r| decode(&r.data)).transpose()
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let id = session_id.to_string();
        sqlx::query!(r#"DELETE FROM tower_sessions.session WHERE id = $1"#, id)
            .execute(&self.db)
            .await
            .map_err(|e| backend(&e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use sqlx::PgPool;
    use time::{Duration, OffsetDateTime};
    use tower_sessions::SessionStore as _;
    use tower_sessions::session::{Id, Record};

    use super::PgSessionStore;

    /// A record expiring `secs` from now, with one data entry.
    fn record(secs: i64) -> Record {
        let mut data = HashMap::new();
        data.insert("k".to_owned(), serde_json::json!("v"));
        Record {
            id: Id::default(),
            data,
            expiry_date: OffsetDateTime::now_utc() + Duration::seconds(secs),
        }
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn save_then_load_round_trips_data(pool: PgPool) {
        let store = PgSessionStore::new(pool);
        let rec = record(3600);
        store.save(&rec).await.unwrap();

        let loaded = store.load(&rec.id).await.unwrap().expect("record loads");
        assert_eq!(loaded, rec, "load must round-trip the full record");
        assert_eq!(loaded.data.get("k"), Some(&serde_json::json!("v")));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn expired_record_loads_as_none(pool: PgPool) {
        let store = PgSessionStore::new(pool);
        let rec = record(-1);
        store.save(&rec).await.unwrap();
        assert!(
            store.load(&rec.id).await.unwrap().is_none(),
            "an expired record must load as None"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn delete_removes_record(pool: PgPool) {
        let store = PgSessionStore::new(pool);
        let rec = record(3600);
        store.save(&rec).await.unwrap();
        store.delete(&rec.id).await.unwrap();
        assert!(
            store.load(&rec.id).await.unwrap().is_none(),
            "delete must remove the record"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn create_assigns_fresh_id_on_collision(pool: PgPool) {
        let store = PgSessionStore::new(pool);

        // Occupy an id, then create with that same id: create must regenerate it
        // and persist under a new id, leaving the original row untouched.
        let occupied = record(3600);
        store.save(&occupied).await.unwrap();

        let mut clash = record(3600);
        clash.id = occupied.id;
        store.create(&mut clash).await.unwrap();

        assert_ne!(
            clash.id, occupied.id,
            "create must regenerate a clashing id"
        );
        assert!(
            store.load(&clash.id).await.unwrap().is_some(),
            "the created record must be retrievable under its fresh id"
        );
        assert!(
            store.load(&occupied.id).await.unwrap().is_some(),
            "the pre-existing record must be left intact"
        );
    }
}
