//! Provider configuration, discovery, and a TTL-cached registry.
//!
//! `openidconnect` types stay inside this crate. The registry loads an
//! `oidc_providers` row, decrypts the client secret through the `crypto` facade,
//! runs OIDC discovery (which fetches the JWKS), and caches the result per
//! provider for `metadata_cache_seconds`. Re-discovery on TTL expiry picks up the
//! IdP's rotated signing keys (providers publish old and new keys during overlap).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use db::Db;
use openidconnect::core::{
    CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClientAuthMethod, CoreGrantType,
    CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm,
    CoreJwsSigningAlgorithm, CoreResponseMode, CoreResponseType, CoreSubjectIdentifierType,
};
use openidconnect::{AdditionalClaims, AdditionalProviderMetadata, IssuerUrl, ProviderMetadata};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::FieldKeyring;
use crate::oidc::error::OidcError;

/// Associated data binding the OIDC-client-secret ciphertext to its purpose.
const OIDC_SECRET_AAD: &[u8] = b"oidc_client_secret";

/// Discovery metadata extended with the RP-initiated-logout endpoint, which the
/// OpenID Core metadata struct omits.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EndSessionMetadata {
    /// The provider's `end_session_endpoint`, when advertised.
    pub(crate) end_session_endpoint: Option<String>,
}
impl AdditionalProviderMetadata for EndSessionMetadata {}

/// Provider metadata carrying [`EndSessionMetadata`]; otherwise the Core shapes.
pub(crate) type OwkProviderMetadata = ProviderMetadata<
    EndSessionMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

/// Captures every non-standard claim as JSON, so a configurable `groups_claim`
/// (or any custom claim) can be read by name from the UserInfo response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ExtraClaims {
    /// All non-standard claims, keyed by name.
    #[serde(flatten)]
    pub(crate) extra: serde_json::Map<String, serde_json::Value>,
}
impl AdditionalClaims for ExtraClaims {}

/// Token-endpoint client-authentication method (the JWT-assertion methods are
/// accepted by the schema but not yet implemented; see [`OidcError::UnsupportedClientAuthMethod`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClientAuthMethod {
    /// `client_secret_basic` (HTTP Basic) — the default.
    Basic,
    /// `client_secret_post` (credentials in the request body).
    Post,
    /// `none` — a public client with no secret.
    None,
}

impl ClientAuthMethod {
    fn parse(raw: &str) -> Result<Self, OidcError> {
        match raw {
            "client_secret_basic" => Ok(Self::Basic),
            "client_secret_post" => Ok(Self::Post),
            "none" => Ok(Self::None),
            "client_secret_jwt" | "private_key_jwt" => Err(OidcError::UnsupportedClientAuthMethod),
            _ => Err(OidcError::Config),
        }
    }
}

/// The account auto-linking policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AccountLinking {
    /// Never auto-link to an existing local account.
    Disabled,
    /// Auto-link only when the IdP asserts a verified, allowed-domain email.
    VerifiedEmail,
}

impl AccountLinking {
    fn parse(raw: &str) -> Result<Self, OidcError> {
        match raw {
            "disabled" => Ok(Self::Disabled),
            "verified_email" => Ok(Self::VerifiedEmail),
            _ => Err(OidcError::Config),
        }
    }
}

/// A validated, first-party provider configuration derived from an `oidc_providers`
/// row, with the client secret decrypted. No vendor types appear in its public shape.
#[derive(Clone, Debug)]
pub(crate) struct ProviderConfig {
    pub(crate) provider_id: Uuid,
    pub(crate) issuer_url: String,
    pub(crate) use_discovery: bool,
    pub(crate) client_id: String,
    pub(crate) client_secret: Option<SecretString>,
    pub(crate) auth_method: ClientAuthMethod,
    pub(crate) scopes: Vec<String>,
    pub(crate) acr_values: Option<Vec<String>>,
    pub(crate) id_token_algs: Vec<CoreJwsSigningAlgorithm>,
    pub(crate) groups_claim: Option<String>,
    pub(crate) jit_provisioning: bool,
    pub(crate) default_role_id: Option<Uuid>,
    pub(crate) default_organization_id: Option<Uuid>,
    pub(crate) allowed_email_domains: Vec<String>,
    pub(crate) account_linking: AccountLinking,
    pub(crate) update_profile_on_login: bool,
    pub(crate) sync_roles_on_login: bool,
    pub(crate) rp_initiated_logout: bool,
    pub(crate) metadata_cache: Duration,
}

impl ProviderConfig {
    /// Build a validated config from a row, decrypting the client secret through
    /// the `crypto` facade. Rejects unsupported auth methods and malformed config.
    fn from_row(row: db::OidcProviderRow, keyring: &FieldKeyring) -> Result<Self, OidcError> {
        let auth_method = ClientAuthMethod::parse(&row.client_auth_method)?;
        let account_linking = AccountLinking::parse(&row.account_linking)?;
        let id_token_algs = parse_signing_algs(&row.id_token_signed_response_alg)?;

        let client_secret = match (auth_method, row.client_secret_encrypted.as_deref()) {
            (ClientAuthMethod::None, _) => None,
            (_, Some(envelope)) => {
                let bytes = crypto::decrypt_field(keyring.oidc_dek(), envelope, OIDC_SECRET_AAD)?;
                let text = String::from_utf8(bytes).map_err(|_| OidcError::Config)?;
                Some(SecretString::from(text))
            }
            (_, None) => return Err(OidcError::Config),
        };

        let metadata_cache =
            Duration::from_secs(u64::try_from(row.metadata_cache_seconds.max(0)).unwrap_or(3600));

        Ok(Self {
            provider_id: row.id,
            issuer_url: row.issuer_url,
            use_discovery: row.use_discovery,
            client_id: row.client_id,
            client_secret,
            auth_method,
            scopes: row.scopes,
            acr_values: row.acr_values,
            id_token_algs,
            groups_claim: row.groups_claim,
            jit_provisioning: row.jit_provisioning,
            default_role_id: row.default_role_id,
            default_organization_id: row.default_organization_id,
            allowed_email_domains: row.allowed_email_domains,
            account_linking,
            update_profile_on_login: row.update_profile_on_login,
            sync_roles_on_login: row.sync_roles_on_login,
            rp_initiated_logout: row.rp_initiated_logout,
            metadata_cache,
        })
    }
}

/// A discovered provider: its OIDC metadata (with JWKS) plus the validated config.
/// Clone-cheap relative to a network round-trip; shared from the registry cache.
#[derive(Clone, Debug)]
pub struct DiscoveredProvider {
    pub(crate) metadata: OwkProviderMetadata,
    pub(crate) config: ProviderConfig,
}

impl DiscoveredProvider {
    /// The RP-initiated-logout endpoint advertised by the provider, if any.
    pub(crate) fn end_session_endpoint(&self) -> Option<&str> {
        self.metadata
            .additional_metadata()
            .end_session_endpoint
            .as_deref()
    }
}

/// One cached discovery result with its expiry.
#[derive(Clone)]
struct Cached {
    provider: DiscoveredProvider,
    fetched_at: Instant,
    ttl: Duration,
}

/// Loads, discovers and caches OIDC providers. Held in `AppState`; cheap to clone
/// (an `Arc` around the shared cache).
#[derive(Clone)]
pub struct ProviderRegistry {
    db: Db,
    http: crate::OidcHttpClient,
    keyring: FieldKeyring,
    cache: Arc<Mutex<HashMap<String, Cached>>>,
}

impl ProviderRegistry {
    /// Construct a registry over the shared pool, HTTP client and field keyring.
    #[must_use]
    pub fn new(db: Db, http: crate::OidcHttpClient, keyring: FieldKeyring) -> Self {
        Self {
            db,
            http,
            keyring,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Resolve an enabled provider by `slug`, discovering (and caching) its metadata
    /// and JWKS. Re-discovers when the cached entry is older than its TTL.
    ///
    /// # Errors
    ///
    /// [`OidcError::Config`] when the slug is unknown/disabled or its config is
    /// invalid; [`OidcError::Discovery`] when the discovery request fails;
    /// [`OidcError::UnsupportedClientAuthMethod`] for a JWT client-auth method.
    pub async fn discovered(&self, slug: &str) -> Result<DiscoveredProvider, OidcError> {
        if let Some(cached) = self.cache.lock().await.get(slug)
            && cached.fetched_at.elapsed() < cached.ttl
        {
            return Ok(cached.provider.clone());
        }

        let row = db::load_enabled_provider_by_slug(&self.db, slug)
            .await?
            .ok_or(OidcError::Config)?;
        let config = ProviderConfig::from_row(row, &self.keyring)?;
        if !config.use_discovery {
            // Manual-endpoint providers are deferred; all named IdPs (Entra, Okta,
            // Google) and the Keycloak test realm support discovery.
            return Err(OidcError::Config);
        }

        let issuer = IssuerUrl::new(config.issuer_url.clone()).map_err(|_| OidcError::Config)?;
        // discover_async fetches the metadata + JWKS and verifies the document's
        // `issuer` equals the requested issuer (mismatch surfaces as an error).
        let metadata = OwkProviderMetadata::discover_async(issuer, self.http.inner())
            .await
            .map_err(|_| OidcError::Discovery)?;

        let provider = DiscoveredProvider { metadata, config };
        let ttl = provider.config.metadata_cache;
        self.cache.lock().await.insert(
            slug.to_owned(),
            Cached {
                provider: provider.clone(),
                fetched_at: Instant::now(),
                ttl,
            },
        );
        Ok(provider)
    }

    /// List enabled providers for rendering "Sign in with …" buttons.
    ///
    /// # Errors
    ///
    /// [`OidcError::Db`] on a database error.
    pub async fn button_list(&self) -> Result<Vec<ProviderButton>, OidcError> {
        let rows = db::load_enabled_provider_summaries(&self.db).await?;
        Ok(rows
            .into_iter()
            .map(|r| ProviderButton {
                slug: r.slug,
                label: r.button_label.unwrap_or(r.display_name),
                icon: r.icon,
            })
            .collect())
    }
}

/// A login-button descriptor for the sign-in page.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderButton {
    /// `/auth/{slug}/start` key.
    pub slug: String,
    /// Display label (button override, else provider name).
    pub label: String,
    /// Optional icon identifier.
    pub icon: Option<String>,
}

/// Parse JOSE algorithm names (e.g. `RS256`) into the typed allow-list.
fn parse_signing_algs(names: &[String]) -> Result<Vec<CoreJwsSigningAlgorithm>, OidcError> {
    names
        .iter()
        .map(|name| {
            serde_json::from_value::<CoreJwsSigningAlgorithm>(serde_json::Value::String(
                name.clone(),
            ))
            .map_err(|_| OidcError::Config)
        })
        .collect()
}

/// Encrypt a plaintext OIDC client secret for storage in
/// `oidc_providers.client_secret_encrypted` (used by seeds and, later, the admin UI).
///
/// # Errors
///
/// [`OidcError::Crypto`] if sealing fails.
pub fn seal_client_secret(
    keyring: &FieldKeyring,
    plaintext: &SecretString,
) -> Result<Vec<u8>, OidcError> {
    Ok(crypto::encrypt_field(
        keyring.oidc_dek(),
        plaintext.expose_secret().as_bytes(),
        OIDC_SECRET_AAD,
    )?)
}
