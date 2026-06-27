//! Just-in-time provisioning, account linking, and group→role mapping.
//!
//! Identity is keyed on `(provider_id, subject)` — the immutable `(issuer, sub)`
//! pair, never the email. An existing local account is auto-linked only when the
//! IdP asserts a verified, allowed-domain email (the nOAuth defence); the same
//! gate guards JIT creation.

use db::Db;
use uuid::Uuid;

use crate::oidc::error::OidcError;
use crate::oidc::flow::VerifiedIdentity;
use crate::oidc::provider::{AccountLinking, DiscoveredProvider, ProviderConfig};

/// Resolve a [`VerifiedIdentity`] to a local user, provisioning or linking as the
/// provider's policy allows, then applying role mappings. Returns the user id to
/// sign in.
///
/// # Errors
///
/// [`OidcError::EmailUnverified`] / [`OidcError::DomainNotAllowed`] when the email
/// gate fails; [`OidcError::ProvisioningDisabled`] when JIT is off and nothing
/// matched; [`OidcError::Provisioning`] on a conflicting local account; or a
/// [`OidcError::Db`] on storage failure.
pub async fn provision_user(
    db: &Db,
    provider: &DiscoveredProvider,
    identity: &VerifiedIdentity,
) -> Result<domain::UserId, OidcError> {
    provision_with_config(db, &provider.config, identity).await
}

/// The provisioning core, taking the validated config directly so it is testable
/// against a database without a live discovery round-trip.
pub(crate) async fn provision_with_config(
    db: &Db,
    config: &ProviderConfig,
    identity: &VerifiedIdentity,
) -> Result<domain::UserId, OidcError> {
    // 1. Returning user: an existing link on (provider, subject).
    if let Some(link) = db::find_oidc_identity(db, config.provider_id, &identity.subject).await? {
        db::touch_oidc_identity(db, link.id).await?;
        refresh_profile_and_roles(db, config, link.user_id, identity).await?;
        return Ok(domain::UserId::new(link.user_id));
    }

    // 2. First sign-in. Auto-link to an existing local account only on a verified,
    //    allowed-domain email (nOAuth defence).
    if config.account_linking == AccountLinking::VerifiedEmail
        && let Some(email) = identity.email.as_deref()
        && identity.email_verified
        && domain_allowed(email, &config.allowed_email_domains)
        && let Some(user_id) = db::load_user_id_by_email(db, email).await?
    {
        db::link_oidc_identity(
            db,
            user_id,
            config.provider_id,
            &identity.subject,
            Some(email),
        )
        .await?;
        refresh_profile_and_roles(db, config, user_id, identity).await?;
        return Ok(domain::UserId::new(user_id));
    }

    // 3. Just-in-time provisioning, gated on the same verified-email check.
    if !config.jit_provisioning {
        return Err(OidcError::ProvisioningDisabled);
    }
    let email = identity
        .email
        .as_deref()
        .ok_or(OidcError::EmailUnverified)?;
    if !identity.email_verified {
        return Err(OidcError::EmailUnverified);
    }
    if !domain_allowed(email, &config.allowed_email_domains) {
        return Err(OidcError::DomainNotAllowed);
    }

    let user_id = match db::jit_create_user(db, email, &identity.display_name).await {
        Ok(id) => id,
        // The email already belongs to a local account that policy will not link
        // (linking disabled, or it lost the verified-email gate above): refuse
        // rather than hijack it.
        Err(db::DbError::Conflict) => return Err(OidcError::Provisioning),
        Err(other) => return Err(other.into()),
    };
    db::link_oidc_identity(
        db,
        user_id,
        config.provider_id,
        &identity.subject,
        Some(email),
    )
    .await?;

    // Assign the default role/org membership when both are configured.
    if let (Some(org), Some(role)) = (config.default_organization_id, config.default_role_id) {
        db::assign_membership(db, user_id, org, role, None).await?;
    }
    if config.sync_roles_on_login {
        apply_role_mappings(db, config, user_id, &identity.groups).await?;
    }
    Ok(domain::UserId::new(user_id))
}

/// Refresh the display name and re-apply role mappings for a known user, per config.
async fn refresh_profile_and_roles(
    db: &Db,
    config: &ProviderConfig,
    user_id: Uuid,
    identity: &VerifiedIdentity,
) -> Result<(), OidcError> {
    if config.update_profile_on_login {
        db::update_user_display_name(db, user_id, &identity.display_name).await?;
    }
    if config.sync_roles_on_login {
        apply_role_mappings(db, config, user_id, &identity.groups).await?;
    }
    Ok(())
}

/// Map asserted group values onto memberships via `oidc_role_mappings`. A mapping
/// applies only when an organization can be resolved (its own scope, else the
/// provider default), since a membership requires one.
async fn apply_role_mappings(
    db: &Db,
    config: &ProviderConfig,
    user_id: Uuid,
    groups: &[String],
) -> Result<(), OidcError> {
    if groups.is_empty() {
        return Ok(());
    }
    for mapping in db::load_role_mappings(db, config.provider_id).await? {
        if groups.iter().any(|g| g == &mapping.external_value)
            && let Some(org) = mapping.organization_id.or(config.default_organization_id)
        {
            db::assign_membership(db, user_id, org, mapping.role_id, mapping.team_id).await?;
        }
    }
    Ok(())
}

/// Whether an email's domain is permitted (an empty allow-list permits all).
fn domain_allowed(email: &str, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        return true;
    }
    match email.rsplit('@').next() {
        Some(domain) => allowed.iter().any(|d| d.eq_ignore_ascii_case(domain)),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::oidc::provider::ClientAuthMethod;

    fn config(provider_id: Uuid) -> ProviderConfig {
        ProviderConfig {
            provider_id,
            issuer_url: "https://issuer.test".to_owned(),
            use_discovery: true,
            client_id: "cid".to_owned(),
            client_secret: None,
            auth_method: ClientAuthMethod::None,
            scopes: vec!["openid".to_owned()],
            acr_values: None,
            id_token_algs: Vec::new(),
            groups_claim: None,
            jit_provisioning: true,
            default_role_id: None,
            default_organization_id: None,
            allowed_email_domains: Vec::new(),
            account_linking: AccountLinking::VerifiedEmail,
            update_profile_on_login: true,
            sync_roles_on_login: true,
            rp_initiated_logout: true,
            metadata_cache: Duration::from_hours(1),
        }
    }

    fn identity(
        subject: &str,
        email: Option<&str>,
        verified: bool,
        groups: &[&str],
    ) -> VerifiedIdentity {
        VerifiedIdentity {
            issuer: "https://issuer.test".to_owned(),
            subject: subject.to_owned(),
            email: email.map(str::to_owned),
            email_verified: verified,
            display_name: "Test User".to_owned(),
            groups: groups.iter().map(|g| (*g).to_owned()).collect(),
            id_token_compact: String::new(),
        }
    }

    /// Insert a minimal `oidc_providers` row (auth method `none` so the
    /// client-secret CHECK is satisfied without a secret) and return its id.
    async fn seed_provider(pool: &PgPool) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO oidc_providers (display_name, slug, issuer_url, client_id, client_auth_method) \
             VALUES ('Test', 'keycloak'::citext, 'https://issuer.test', 'cid', 'none') RETURNING id",
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn seed_org(pool: &PgPool) -> Uuid {
        let tag = Uuid::new_v4().simple().to_string();
        sqlx::query_scalar(
            "INSERT INTO organizations (name, slug) VALUES ('Org', $1::citext) RETURNING id",
        )
        .bind(&tag)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn seed_role(pool: &PgPool, key: &str) -> Uuid {
        let unique = format!("{key}-{}", Uuid::new_v4().simple());
        sqlx::query_scalar("INSERT INTO roles (key, name) VALUES ($1::citext, $2) RETURNING id")
            .bind(&unique)
            .bind(key)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn user_count(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM users")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn membership_role(pool: &PgPool, user_id: Uuid) -> Option<Uuid> {
        sqlx::query_scalar("SELECT role_id FROM memberships WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn jit_creates_user_and_links_identity(pool: PgPool) {
        let provider = seed_provider(&pool).await;
        let cfg = config(provider);
        let id = identity("sub-1", Some("alice@example.test"), true, &[]);

        let user_id = provision_with_config(&pool, &cfg, &id).await.unwrap();

        assert_eq!(
            user_count(&pool).await,
            1,
            "JIT must create exactly one user"
        );
        let linked: Uuid = sqlx::query_scalar(
            "SELECT user_id FROM oidc_identities WHERE provider_id = $1 AND subject = 'sub-1'",
        )
        .bind(provider)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            linked,
            user_id.as_uuid(),
            "identity must link to the new user"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn second_login_reuses_identity_without_duplicating(pool: PgPool) {
        let provider = seed_provider(&pool).await;
        let cfg = config(provider);
        let id = identity("sub-1", Some("alice@example.test"), true, &[]);

        let first = provision_with_config(&pool, &cfg, &id).await.unwrap();
        let second = provision_with_config(&pool, &cfg, &id).await.unwrap();

        assert_eq!(
            first.as_uuid(),
            second.as_uuid(),
            "same subject → same user"
        );
        assert_eq!(user_count(&pool).await, 1, "no duplicate user on re-login");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn links_to_existing_verified_email_account(pool: PgPool) {
        let provider = seed_provider(&pool).await;
        // A pre-existing local account with the same email.
        let existing: Uuid = sqlx::query_scalar(
            "INSERT INTO users (email, display_name, webauthn_user_handle) \
             VALUES ('bob@example.test'::citext, 'Bob', $1) RETURNING id",
        )
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .fetch_one(&pool)
        .await
        .unwrap();

        let cfg = config(provider);
        let id = identity("sub-bob", Some("bob@example.test"), true, &[]);
        let user_id = provision_with_config(&pool, &cfg, &id).await.unwrap();

        assert_eq!(
            user_id.as_uuid(),
            existing,
            "must link to the existing account"
        );
        assert_eq!(
            user_count(&pool).await,
            1,
            "linking must not create a new user"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn unverified_email_is_refused(pool: PgPool) {
        let provider = seed_provider(&pool).await;
        let cfg = config(provider);
        let id = identity("sub-x", Some("eve@example.test"), false, &[]);

        let err = provision_with_config(&pool, &cfg, &id).await.unwrap_err();
        assert!(matches!(err, OidcError::EmailUnverified));
        assert_eq!(
            user_count(&pool).await,
            0,
            "no account on an unverified email"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn disallowed_domain_is_refused(pool: PgPool) {
        let provider = seed_provider(&pool).await;
        let mut cfg = config(provider);
        cfg.allowed_email_domains = vec!["allowed.test".to_owned()];
        let id = identity("sub-x", Some("mallory@other.test"), true, &[]);

        let err = provision_with_config(&pool, &cfg, &id).await.unwrap_err();
        assert!(matches!(err, OidcError::DomainNotAllowed));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn jit_assigns_default_role_then_group_mapping_overrides(pool: PgPool) {
        let provider = seed_provider(&pool).await;
        let org = seed_org(&pool).await;
        let member = seed_role(&pool, "member").await;
        let admin = seed_role(&pool, "admin").await;
        sqlx::query(
            "INSERT INTO oidc_role_mappings (provider_id, external_value, role_id, organization_id) \
             VALUES ($1, 'admins', $2, $3)",
        )
        .bind(provider)
        .bind(admin)
        .bind(org)
        .execute(&pool)
        .await
        .unwrap();

        let mut cfg = config(provider);
        cfg.default_organization_id = Some(org);
        cfg.default_role_id = Some(member);
        cfg.groups_claim = Some("groups".to_owned());

        // A user in the `admins` group: the default member role is assigned, then the
        // mapping upserts it to admin (last write wins on the (user, org) membership).
        let id = identity("sub-alice", Some("alice@example.test"), true, &["admins"]);
        let user_id = provision_with_config(&pool, &cfg, &id).await.unwrap();
        assert_eq!(
            membership_role(&pool, user_id.as_uuid()).await,
            Some(admin),
            "group→role mapping must override the default role"
        );

        // A user with no mapped group keeps the default member role.
        let id2 = identity("sub-carol", Some("carol@example.test"), true, &["staff"]);
        let carol = provision_with_config(&pool, &cfg, &id2).await.unwrap();
        assert_eq!(
            membership_role(&pool, carol.as_uuid()).await,
            Some(member),
            "an unmapped group keeps the default role"
        );
    }
}
