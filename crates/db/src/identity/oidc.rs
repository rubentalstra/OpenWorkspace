//! OIDC provider configuration and federated-identity queries — the storage the
//! `auth` crate's OIDC facade builds on.
//!
//! These return first-party row structs; no `openidconnect`/`oauth2` type ever
//! reaches this crate. Identity is keyed on `(provider_id, subject)`, which is the
//! immutable `(issuer, sub)` pair, never the email.

use uuid::Uuid;

use crate::{Db, DbError, classify};

/// A configured OIDC provider, with the fields the relying-party flow needs. The
/// `auth` facade maps this to its own validated config; the raw row never leaks.
#[derive(Clone, Debug)]
pub struct OidcProviderRow {
    /// `oidc_providers.id`.
    pub id: Uuid,
    /// URL-stable key used in `/auth/{slug}/callback`.
    pub slug: String,
    /// Human-readable provider name.
    pub display_name: String,
    /// The IdP issuer identifier; the discovery base and the value the `iss` claim
    /// must match exactly.
    pub issuer_url: String,
    /// Whether to read `.well-known/openid-configuration`.
    pub use_discovery: bool,
    /// Manual endpoint overrides, used when discovery is off.
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    /// RP-initiated-logout endpoint (also discoverable).
    pub end_session_endpoint: Option<String>,
    /// Discovery/JWKS cache lifetime override, in seconds.
    pub metadata_cache_seconds: i32,
    /// OAuth client id.
    pub client_id: String,
    /// Token-endpoint authentication method (`client_secret_basic`/`_post`/`none`;
    /// the two JWT methods are accepted by the schema but not yet implemented).
    pub client_auth_method: String,
    /// AEAD-sealed client secret (envelope from the `crypto` facade).
    pub client_secret_encrypted: Option<Vec<u8>>,
    /// `crypto_keys` id of the signing key for `private_key_jwt` (deferred).
    pub client_assertion_key_id: Option<Uuid>,
    /// Requested scopes (always includes `openid`).
    pub scopes: Vec<String>,
    /// `query` or `form_post`.
    pub response_mode: String,
    /// Optional `prompt` value.
    pub prompt: Option<String>,
    /// Optional requested authentication-context class references (e.g. MFA at the IdP).
    pub acr_values: Option<Vec<String>>,
    /// Optional `max_age` in seconds.
    pub max_age_seconds: Option<i32>,
    /// Allow-list of acceptable ID-token signature algorithms (e.g. `RS256`).
    pub id_token_signed_response_alg: Vec<String>,
    /// Leeway on `exp`/`iat`/`nbf`, in seconds.
    pub clock_skew_seconds: i32,
    /// Claim names to read identity attributes from.
    pub email_claim: String,
    pub email_verified_claim: String,
    pub name_claim: String,
    pub username_claim: String,
    /// Claim carrying group/role values for `oidc_role_mappings`, if any.
    pub groups_claim: Option<String>,
    /// Whether the provider is offered at all.
    pub enabled: bool,
    /// Whether a first sign-in provisions a new local user.
    pub jit_provisioning: bool,
    /// Role/org assigned to a just-provisioned user, when both are set.
    pub default_role_id: Option<Uuid>,
    pub default_organization_id: Option<Uuid>,
    /// Email domains permitted to provision/link (empty = no restriction).
    pub allowed_email_domains: Vec<String>,
    /// `disabled` or `verified_email` — the auto-link policy.
    pub account_linking: String,
    /// Whether to refresh the local display name from the IdP on each login.
    pub update_profile_on_login: bool,
    /// Whether to re-apply `oidc_role_mappings` on each login.
    pub sync_roles_on_login: bool,
    /// Whether RP-initiated logout is offered.
    pub rp_initiated_logout: bool,
}

/// Lightweight provider listing for rendering "Sign in with …" buttons.
#[derive(Clone, Debug)]
pub struct OidcProviderSummary {
    /// `/auth/{slug}/start` key.
    pub slug: String,
    /// Provider display name (fallback when `button_label` is unset).
    pub display_name: String,
    /// Optional button label override.
    pub button_label: Option<String>,
    /// Optional icon identifier.
    pub icon: Option<String>,
    /// Presentation order.
    pub sort_order: i32,
}

/// A federated-identity link row.
#[derive(Clone, Debug)]
pub struct OidcIdentityRow {
    /// `oidc_identities.id`.
    pub id: Uuid,
    /// The linked local user.
    pub user_id: Uuid,
    /// Owning provider.
    pub provider_id: Uuid,
    /// The IdP `sub` claim.
    pub subject: String,
}

/// A group/role mapping: an IdP-asserted value mapped to an internal role, scoped
/// optionally to an organization and team.
#[derive(Clone, Debug)]
pub struct OidcRoleMappingRow {
    /// The group/role value asserted by the IdP (matched against the `groups` claim).
    pub external_value: String,
    /// The internal role to assign.
    pub role_id: Uuid,
    /// Optional organization scope; falls back to the provider default.
    pub organization_id: Option<Uuid>,
    /// Optional team scope (must belong to `organization_id`).
    pub team_id: Option<Uuid>,
}

/// Load an enabled provider by its `slug`. Returns `None` when absent or disabled.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_enabled_provider_by_slug(
    pool: &Db,
    slug: &str,
) -> Result<Option<OidcProviderRow>, DbError> {
    let row = sqlx::query_as!(
        OidcProviderRow,
        r#"
        SELECT
            id,
            slug::text AS "slug!",
            display_name,
            issuer_url,
            use_discovery,
            authorization_endpoint,
            token_endpoint,
            userinfo_endpoint,
            jwks_uri,
            end_session_endpoint,
            metadata_cache_seconds,
            client_id,
            client_auth_method,
            client_secret_encrypted,
            client_assertion_key_id,
            scopes,
            response_mode,
            prompt,
            acr_values,
            max_age_seconds,
            id_token_signed_response_alg,
            clock_skew_seconds,
            email_claim,
            email_verified_claim,
            name_claim,
            username_claim,
            groups_claim,
            enabled,
            jit_provisioning,
            default_role_id,
            default_organization_id,
            allowed_email_domains,
            account_linking,
            update_profile_on_login,
            sync_roles_on_login,
            rp_initiated_logout
        FROM oidc_providers
        WHERE slug = $1::citext AND enabled
        "#,
        slug,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// List enabled providers (button presentation order) for the login page.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_enabled_provider_summaries(
    pool: &Db,
) -> Result<Vec<OidcProviderSummary>, DbError> {
    let rows = sqlx::query_as!(
        OidcProviderSummary,
        r#"
        SELECT slug::text AS "slug!", display_name, button_label, icon, sort_order
        FROM oidc_providers
        WHERE enabled
        ORDER BY sort_order, display_name
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Find the federated identity for `(provider_id, subject)` — the immutable
/// `(issuer, sub)` key. Returns `None` on first sign-in.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn find_oidc_identity(
    pool: &Db,
    provider_id: Uuid,
    subject: &str,
) -> Result<Option<OidcIdentityRow>, DbError> {
    let row = sqlx::query_as!(
        OidcIdentityRow,
        r#"
        SELECT id, user_id, provider_id, subject
        FROM oidc_identities
        WHERE provider_id = $1 AND subject = $2
        "#,
        provider_id,
        subject,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Link a local user to a provider subject, recording the email at link time.
/// A duplicate `(provider_id, subject)` surfaces as [`DbError::Conflict`].
///
/// # Errors
///
/// [`DbError::Conflict`] on a duplicate link; [`DbError::Sqlx`] otherwise.
pub async fn link_oidc_identity(
    pool: &Db,
    user_id: Uuid,
    provider_id: Uuid,
    subject: &str,
    email_at_link: Option<&str>,
) -> Result<Uuid, DbError> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO oidc_identities (user_id, provider_id, subject, email_at_link, last_login_at)
        VALUES ($1, $2, $3, $4::citext, now())
        RETURNING id
        "#,
        user_id,
        provider_id,
        subject,
        email_at_link,
    )
    .fetch_one(pool)
    .await
    .map_err(classify)?;
    Ok(id)
}

/// Stamp `last_login_at = now()` on an existing identity link.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn touch_oidc_identity(pool: &Db, identity_id: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE oidc_identities SET last_login_at = now() WHERE id = $1"#,
        identity_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Just-in-time provision a password-less user (no `password_credentials` row) with
/// a random opaque `webauthn_user_handle`, returning the new id. A unique-email
/// clash surfaces as [`DbError::Conflict`].
///
/// # Errors
///
/// [`DbError::Conflict`] when the email already exists; [`DbError::Sqlx`] otherwise.
pub async fn jit_create_user(pool: &Db, email: &str, display_name: &str) -> Result<Uuid, DbError> {
    let handle = Uuid::new_v4().into_bytes().to_vec();
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO users (email, display_name, webauthn_user_handle, email_verified_at, last_login_at)
        VALUES ($1::citext, $2, $3, now(), now())
        RETURNING id
        "#,
        email,
        display_name,
        handle,
    )
    .fetch_one(pool)
    .await
    .map_err(classify)?;
    Ok(id)
}

/// Refresh a user's display name from the IdP (when `update_profile_on_login`).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn update_user_display_name(
    pool: &Db,
    user_id: Uuid,
    display_name: &str,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE users SET display_name = $2 WHERE id = $1"#,
        user_id,
        display_name,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Load the group→role mappings for a provider.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_role_mappings(
    pool: &Db,
    provider_id: Uuid,
) -> Result<Vec<OidcRoleMappingRow>, DbError> {
    let rows = sqlx::query_as!(
        OidcRoleMappingRow,
        r#"
        SELECT external_value, role_id, organization_id, team_id
        FROM oidc_role_mappings
        WHERE provider_id = $1
        "#,
        provider_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Assign (or update) a user's role within an organization, optionally scoped to a
/// team. Idempotent: a repeated sign-in re-applies the role via the partial-unique
/// membership index (last write wins), so `sync_roles_on_login` is safe to run on
/// every login.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn assign_membership(
    pool: &Db,
    user_id: Uuid,
    organization_id: Uuid,
    role_id: Uuid,
    team_id: Option<Uuid>,
) -> Result<(), DbError> {
    match team_id {
        None => {
            sqlx::query!(
                r#"
                INSERT INTO memberships (user_id, organization_id, role_id)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_id, organization_id) WHERE team_id IS NULL
                DO UPDATE SET role_id = EXCLUDED.role_id
                "#,
                user_id,
                organization_id,
                role_id,
            )
            .execute(pool)
            .await
            .map_err(classify)?;
        }
        Some(team_id) => {
            sqlx::query!(
                r#"
                INSERT INTO memberships (user_id, organization_id, team_id, role_id)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (user_id, team_id) WHERE team_id IS NOT NULL
                DO UPDATE SET role_id = EXCLUDED.role_id
                "#,
                user_id,
                organization_id,
                team_id,
                role_id,
            )
            .execute(pool)
            .await
            .map_err(classify)?;
        }
    }
    Ok(())
}
