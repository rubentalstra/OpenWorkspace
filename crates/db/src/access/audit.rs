//! Append-only audit-log writer.
//!
//! Every privileged decision records one row: the actor, the principal it was on
//! behalf of (delegation), the action token, the outcome, and the target. The
//! runtime role may only `INSERT`/`SELECT` here — `UPDATE`/`DELETE` are revoked
//! (see the `p8_access_enforcement` migration) and a trigger blocks them too.
//! `metadata` must carry IDs only, never names or emails (schema rule M12), so
//! anonymizing the `users` row suffices for erasure.

use uuid::Uuid;

use crate::{Db, DbError};

/// Persistence-mapped mirror of the `actor_kind` enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "actor_kind", rename_all = "snake_case")]
pub enum ActorKindRow {
    /// A human user acted.
    User,
    /// An API key acted.
    ApiKey,
    /// The platform itself acted (a job, a system task).
    System,
}

/// Persistence-mapped mirror of the `audit_outcome` enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "audit_outcome", rename_all = "lowercase")]
pub enum AuditOutcomeRow {
    /// The action was permitted and applied.
    Success,
    /// The action was attempted but failed (infrastructure / conflict).
    Failure,
    /// The action was refused by authorization.
    Denied,
}

/// A row to append to the audit log. Borrows its string fields for cheap
/// construction at the call site; `metadata` must contain only identifiers.
pub struct NewAuditEntry<'a> {
    /// What kind of actor performed the action.
    pub actor_kind: ActorKindRow,
    /// The acting user, if any (the delegate in a delegated action).
    pub actor_user_id: Option<Uuid>,
    /// The principal the action was performed for, if delegated.
    pub on_behalf_of_user_id: Option<Uuid>,
    /// The action token (e.g. `booking.create`).
    pub action: &'a str,
    /// The outcome (success / failure / denied).
    pub outcome: AuditOutcomeRow,
    /// The kind of target (free-form, e.g. `resource`), if any.
    pub target_type: Option<&'a str>,
    /// The target's id, if any.
    pub target_id: Option<Uuid>,
    /// Structured detail — **identifiers only**, never names or emails.
    pub metadata: serde_json::Value,
}

/// Appends one entry to the audit log.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn record_audit(pool: &Db, entry: &NewAuditEntry<'_>) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        INSERT INTO audit_log
            (actor_kind, actor_user_id, on_behalf_of_user_id, action, outcome, target_type, target_id, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        entry.actor_kind as _,
        entry.actor_user_id,
        entry.on_behalf_of_user_id,
        entry.action,
        entry.outcome as _,
        entry.target_type,
        entry.target_id,
        &entry.metadata,
    )
    .execute(pool)
    .await?;
    Ok(())
}
