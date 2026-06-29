//! The `AuthzBackend`: load → decide → audit.

use chrono::Utc;

use domain::UserId;
use domain::authz::{Action, Delegation, ManagementTarget};

use crate::authz::error::AuthzError;
use crate::authz::target::Target;

/// The single place permissions are decided. Clone-cheap (holds the pooled `Db`).
/// Supplies the real clock; the pure `domain` decision stays clock-injected.
#[derive(Clone)]
pub struct AuthzBackend {
    db: db::Db,
}

impl AuthzBackend {
    /// Builds the backend over the runtime database pool.
    #[must_use]
    pub fn new(db: db::Db) -> Self {
        Self { db }
    }

    /// Decides whether `actor` may perform `action` on `target`, optionally acting
    /// on behalf of a principal who named `actor` a booking delegate. Every call
    /// records exactly one audit row (success or denied) once a decision is made.
    ///
    /// Delegated authority is clamped to the booking family
    /// ([`Delegation::as_principal`]); a missing or inactive delegation, or one not
    /// issued to `actor`, is denied.
    ///
    /// # Errors
    ///
    /// [`AuthzError::Denied`] when the action is not permitted,
    /// [`AuthzError::NotFound`] when the target does not exist, or
    /// [`AuthzError::Db`] on a database error.
    pub async fn authorize(
        &self,
        actor: UserId,
        action: Action,
        target: Target,
        on_behalf_of: Option<UserId>,
    ) -> Result<(), AuthzError> {
        let now = Utc::now();
        let actor_ctx = db::load_authz_context(&self.db, actor).await?;

        let effective_ctx = match on_behalf_of {
            None => actor_ctx,
            Some(principal) => {
                let delegation =
                    db::load_active_delegation(&self.db, actor, principal, now).await?;
                let allowed = delegation
                    .as_ref()
                    .and_then(|d| Delegation::as_principal(actor_ctx, Some(d), now));
                let Some(ctx) = allowed else {
                    self.record(actor, on_behalf_of, action, target, false)
                        .await?;
                    return Err(AuthzError::Denied);
                };
                ctx
            }
        };

        let management = self.resolve_target(target).await?;
        let decision = domain::authz::authorize(&effective_ctx, action, &management, "/", now);
        self.record(actor, on_behalf_of, action, target, decision.allowed)
            .await?;
        if decision.allowed {
            Ok(())
        } else {
            Err(AuthzError::Denied)
        }
    }

    /// Whether `viewer` may see `resource` under the instance segmentation mode.
    /// Authoritative companion to the resource RLS policy.
    ///
    /// # Errors
    ///
    /// [`AuthzError::NotFound`] when the resource does not exist, or
    /// [`AuthzError::Db`] on a database error.
    pub async fn visible_resource(
        &self,
        viewer: UserId,
        resource: domain::ResourceId,
    ) -> Result<bool, AuthzError> {
        let viewer_seg = db::load_viewer_segmentation(&self.db, viewer).await?;
        let Some(resource_seg) = db::load_resource_segmentation(&self.db, resource).await? else {
            return Err(AuthzError::NotFound);
        };
        let mode = db::load_segmentation_mode(&self.db).await?;
        Ok(domain::segmentation::visible(
            resource_seg.effective(),
            &viewer_seg,
            mode,
        ))
    }

    async fn resolve_target(&self, target: Target) -> Result<ManagementTarget, AuthzError> {
        match target {
            Target::Resource(id) => db::load_resource_target(&self.db, id)
                .await?
                .ok_or(AuthzError::NotFound),
            Target::Location(id) => db::load_location_target(&self.db, id)
                .await?
                .ok_or(AuthzError::NotFound),
            Target::Organization(id) => Ok(ManagementTarget {
                location: None,
                organization: Some(id),
            }),
            Target::Instance => Ok(ManagementTarget {
                location: None,
                organization: None,
            }),
        }
    }

    async fn record(
        &self,
        actor: UserId,
        on_behalf_of: Option<UserId>,
        action: Action,
        target: Target,
        allowed: bool,
    ) -> Result<(), AuthzError> {
        let (target_type, target_id) = target.audit_parts();
        let entry = db::NewAuditEntry {
            actor_kind: db::ActorKindRow::User,
            actor_user_id: Some(actor.as_uuid()),
            on_behalf_of_user_id: on_behalf_of.map(UserId::as_uuid),
            action: action.token(),
            outcome: if allowed {
                db::AuditOutcomeRow::Success
            } else {
                db::AuditOutcomeRow::Denied
            },
            target_type,
            target_id,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        };
        db::record_audit(&self.db, &entry).await?;
        Ok(())
    }
}
