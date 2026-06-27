//! The Authorization Code + PKCE flow: build the authorization URL, then verify
//! the callback and ID token. The `openidconnect` `CoreClient` is built inline in
//! each function (its endpoint typestate is inferred, never named) so no vendor
//! type leaks into a stored field or a public signature.

use std::fmt;

use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreGenderClaim, CoreIdTokenClaims};
use openidconnect::{
    AccessTokenHash, AuthType, AuthenticationContextClass, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, TokenResponse, UserInfoClaims,
};
use secrecy::ExposeSecret as _;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq as _;
use uuid::Uuid;

use crate::OidcHttpClient;
use crate::oidc::error::OidcError;
use crate::oidc::provider::{ClientAuthMethod, DiscoveredProvider, ExtraClaims};

/// The per-request transaction stored server-side (in the session) between the
/// authorization redirect and the callback. Single-use and time-boxed by the
/// session. `Debug` redacts the secret-bearing fields so they never reach logs.
#[derive(Clone, Serialize, Deserialize)]
pub struct OidcTransaction {
    /// The provider this transaction belongs to.
    pub provider_id: Uuid,
    /// The CSRF `state` to match against the callback.
    pub state: String,
    /// The `nonce` bound into the ID token.
    pub nonce: String,
    /// The PKCE code verifier sent at code exchange.
    pub pkce_verifier: String,
    /// Validated post-login redirect target (already on the local allow-list).
    pub return_to: String,
}

impl fmt::Debug for OidcTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OidcTransaction")
            .field("provider_id", &self.provider_id)
            .field("state", &"<redacted>")
            .field("nonce", &"<redacted>")
            .field("pkce_verifier", &"<redacted>")
            .field("return_to", &self.return_to)
            .finish()
    }
}

/// The authorization URL plus the transaction to persist before redirecting.
#[derive(Clone, Debug)]
pub struct AuthRequest {
    /// Where to send the user agent.
    pub authorize_url: String,
    /// State to store in the session, keyed by `state`.
    pub transaction: OidcTransaction,
}

/// The query parameters returned to the callback endpoint.
#[derive(Clone, Debug, Deserialize)]
pub struct OidcCallback {
    /// The authorization code.
    pub code: String,
    /// The `state` to match against the stored transaction.
    pub state: String,
    /// The RFC 9207 issuer identifier, when the provider sends it.
    pub iss: Option<String>,
}

/// The verified, first-party identity resolved from a successful login. Carries no
/// vendor type; `Debug` redacts the compact ID token retained for logout.
#[derive(Clone)]
pub struct VerifiedIdentity {
    /// The IdP issuer (the immutable half of the identity key).
    pub issuer: String,
    /// The IdP subject (`sub`) — the identity key, never the email.
    pub subject: String,
    /// The asserted email, if any.
    pub email: Option<String>,
    /// Whether the IdP asserted the email as verified.
    pub email_verified: bool,
    /// A display name (from `name`, else `preferred_username`, else the email).
    pub display_name: String,
    /// Group/role values from the configured `groups_claim` (empty if none).
    pub groups: Vec<String>,
    /// The compact ID token, retained only as the `id_token_hint` for RP-initiated
    /// logout — never as the session token, never sent to first-party APIs.
    pub id_token_compact: String,
}

impl fmt::Debug for VerifiedIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerifiedIdentity")
            .field("issuer", &self.issuer)
            .field("subject", &self.subject)
            .field("email", &self.email)
            .field("email_verified", &self.email_verified)
            .field("display_name", &self.display_name)
            .field("groups", &self.groups)
            .field("id_token_compact", &"<redacted>")
            .finish()
    }
}

/// Begin a login: build the authorization URL (Authorization Code + PKCE S256,
/// random `state` + `nonce`) and the transaction to persist before redirecting.
///
/// # Errors
///
/// [`OidcError::Config`] if the redirect URI is malformed.
pub fn begin_login(
    provider: &DiscoveredProvider,
    redirect_uri: &str,
    return_to: String,
) -> Result<AuthRequest, OidcError> {
    let redirect = RedirectUrl::new(redirect_uri.to_owned()).map_err(|_| OidcError::Config)?;
    let client = CoreClient::from_provider_metadata(
        provider.metadata.clone(),
        ClientId::new(provider.config.client_id.clone()),
        provider
            .config
            .client_secret
            .as_ref()
            .map(|s| ClientSecret::new(s.expose_secret().to_owned())),
    )
    .set_redirect_uri(redirect);
    let client = match provider.config.auth_method {
        ClientAuthMethod::Post => client.set_auth_type(AuthType::RequestBody),
        ClientAuthMethod::Basic | ClientAuthMethod::None => client,
    };

    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let mut request = client.authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    );
    for scope in &provider.config.scopes {
        request = request.add_scope(Scope::new(scope.clone()));
    }
    request = request.set_pkce_challenge(challenge);
    if let Some(acr_values) = &provider.config.acr_values {
        for acr in acr_values {
            request = request.add_auth_context_value(AuthenticationContextClass::new(acr.clone()));
        }
    }
    let (url, csrf_token, nonce) = request.url();

    Ok(AuthRequest {
        authorize_url: url.to_string(),
        transaction: OidcTransaction {
            provider_id: provider.config.provider_id,
            state: csrf_token.secret().clone(),
            nonce: nonce.secret().clone(),
            pkce_verifier: verifier.secret().clone(),
            return_to,
        },
    })
}

/// Complete a login: validate the RFC 9207 `iss` and the `state`, exchange the
/// code (sending the PKCE verifier), then verify the ID token (signature against
/// the JWKS with the per-provider algorithm allow-list, plus issuer, audience,
/// expiry, nonce and `at_hash`). Returns the verified identity.
///
/// # Errors
///
/// A specific [`OidcError`] for each failed check (`ResponseIssuerMismatch`,
/// `StateMismatch`, `TokenExchange`, `IdToken`, `AccessTokenHash`, …).
pub async fn complete_login(
    provider: &DiscoveredProvider,
    http: &OidcHttpClient,
    redirect_uri: &str,
    callback: OidcCallback,
    transaction: OidcTransaction,
) -> Result<VerifiedIdentity, OidcError> {
    // RFC 9207 mix-up defence: a returned `iss` must match this provider's issuer.
    let issuer = provider.metadata.issuer().as_str().to_owned();
    if let Some(iss) = &callback.iss
        && iss != &issuer
    {
        return Err(OidcError::ResponseIssuerMismatch);
    }
    // Exact, constant-time `state` match.
    if !bool::from(
        callback
            .state
            .as_bytes()
            .ct_eq(transaction.state.as_bytes()),
    ) {
        return Err(OidcError::StateMismatch);
    }

    let redirect = RedirectUrl::new(redirect_uri.to_owned()).map_err(|_| OidcError::Config)?;
    let client = CoreClient::from_provider_metadata(
        provider.metadata.clone(),
        ClientId::new(provider.config.client_id.clone()),
        provider
            .config
            .client_secret
            .as_ref()
            .map(|s| ClientSecret::new(s.expose_secret().to_owned())),
    )
    .set_redirect_uri(redirect);
    let client = match provider.config.auth_method {
        ClientAuthMethod::Post => client.set_auth_type(AuthType::RequestBody),
        ClientAuthMethod::Basic | ClientAuthMethod::None => client,
    };

    let token_response = client
        .exchange_code(AuthorizationCode::new(callback.code))
        .map_err(|_| OidcError::MissingEndpoint)?
        .set_pkce_verifier(PkceCodeVerifier::new(transaction.pkce_verifier))
        .request_async(http.inner())
        .await
        .map_err(|_| OidcError::TokenExchange)?;

    let id_token = token_response.id_token().ok_or(OidcError::IdToken)?;
    let nonce = Nonce::new(transaction.nonce);
    let verifier = client
        .id_token_verifier()
        .set_allowed_algs(provider.config.id_token_algs.clone());
    let claims = id_token
        .claims(&verifier, &nonce)
        .map_err(|_| OidcError::IdToken)?;

    // Bind the access token to the ID token when the provider asserts `at_hash`.
    if let Some(expected) = claims.access_token_hash() {
        let alg = id_token.signing_alg().map_err(|_| OidcError::IdToken)?;
        let key = id_token
            .signing_key(&verifier)
            .map_err(|_| OidcError::IdToken)?;
        let actual = AccessTokenHash::from_token(token_response.access_token(), alg, key)
            .map_err(|_| OidcError::IdToken)?;
        if actual != *expected {
            return Err(OidcError::AccessTokenHash);
        }
    }

    let subject = claims.subject().clone();
    let email = claims.email().map(|e| e.as_str().to_owned());
    let email_verified = claims.email_verified().unwrap_or(false);
    let display_name = display_name_from(claims, email.as_deref());

    // Groups come from UserInfo (the proven openidconnect pattern for non-standard
    // claims); only fetched when a provider actually maps groups to roles. The
    // expected `subject` is passed so the crate verifies the UserInfo `sub` matches.
    let groups = match &provider.config.groups_claim {
        Some(groups_claim) => {
            let userinfo: UserInfoClaims<ExtraClaims, CoreGenderClaim> = client
                .user_info(
                    token_response.access_token().to_owned(),
                    Some(subject.clone()),
                )
                .map_err(|_| OidcError::MissingEndpoint)?
                .request_async(http.inner())
                .await
                .map_err(|_| OidcError::Http)?;
            groups_from(userinfo.additional_claims(), groups_claim)
        }
        None => Vec::new(),
    };

    Ok(VerifiedIdentity {
        issuer,
        subject: subject.as_str().to_owned(),
        email,
        email_verified,
        display_name,
        groups,
        id_token_compact: id_token.to_string(),
    })
}

/// Choose a display name: `name`, else `preferred_username`, else the email local
/// part, else a generic fallback.
fn display_name_from(claims: &CoreIdTokenClaims, email: Option<&str>) -> String {
    if let Some(name) = claims.name().and_then(|localized| localized.get(None)) {
        return name.as_str().to_owned();
    }
    if let Some(username) = claims.preferred_username() {
        return username.as_str().to_owned();
    }
    if let Some(local) = email.and_then(|e| e.split('@').next())
        && !local.is_empty()
    {
        return local.to_owned();
    }
    "User".to_owned()
}

/// Extract group values from the configured claim, accepting a JSON array of
/// strings or a single string.
fn groups_from(extra: &ExtraClaims, claim: &str) -> Vec<String> {
    match extra.extra.get(claim) {
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        Some(serde_json::Value::String(single)) => vec![single.clone()],
        _ => Vec::new(),
    }
}
