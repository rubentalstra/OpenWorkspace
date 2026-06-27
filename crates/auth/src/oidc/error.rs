//! Typed OIDC errors. No `openidconnect`/`oauth2`/`reqwest` type appears here, so
//! the facade boundary holds; `Display` is lowercase with no trailing period and
//! the client-facing HTTP mapping (in the server layer) stays generic.

/// Errors raised by the OIDC relying-party facade.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OidcError {
    /// Provider discovery (`.well-known/openid-configuration` or JWKS) failed.
    #[error("oidc provider discovery failed")]
    Discovery,
    /// The discovered `issuer` did not match the configured issuer.
    #[error("oidc issuer does not match the configured value")]
    IssuerMismatch,
    /// The authorization-response `iss` (RFC 9207) did not match the provider.
    #[error("oidc authorization-response issuer mismatch")]
    ResponseIssuerMismatch,
    /// The callback `state` did not match the stored value (CSRF / mix-up).
    #[error("oidc state mismatch")]
    StateMismatch,
    /// Exchanging the authorization code for tokens failed.
    #[error("oidc token exchange failed")]
    TokenExchange,
    /// ID-token validation failed (signature, issuer, audience, expiry or nonce).
    #[error("oidc id-token validation failed")]
    IdToken,
    /// The provider asserted an access-token hash that did not match.
    #[error("oidc access-token hash mismatch")]
    AccessTokenHash,
    /// A configured client-authentication method is not implemented (the
    /// `client_secret_jwt` / `private_key_jwt` JWT-assertion methods are deferred).
    #[error("oidc client authentication method is not supported")]
    UnsupportedClientAuthMethod,
    /// A required endpoint was absent from discovery and not configured manually.
    #[error("oidc provider is missing a required endpoint")]
    MissingEndpoint,
    /// The provider configuration is structurally invalid (e.g. a malformed URL or
    /// an unrecognised signature algorithm).
    #[error("oidc provider configuration is invalid")]
    Config,
    /// The IdP did not assert a verified email, so the account cannot be linked.
    #[error("oidc identity has no verified email")]
    EmailUnverified,
    /// The email's domain is not in the provider's allow-list.
    #[error("oidc email domain is not allowed")]
    DomainNotAllowed,
    /// Just-in-time provisioning is disabled and no local account matched.
    #[error("oidc just-in-time provisioning is disabled")]
    ProvisioningDisabled,
    /// A local account with this email already exists but cannot be linked under
    /// the configured policy (e.g. linking disabled, or the email is unverified).
    #[error("oidc cannot provision: a conflicting local account exists")]
    Provisioning,
    /// An outbound OIDC HTTP request could not be built or sent.
    #[error("oidc http transport failed")]
    Http,
    /// Reading or writing the in-flight transaction in the session failed.
    #[error("oidc session state access failed")]
    Session,
    /// A persistence error during provisioning, linking or role mapping.
    #[error(transparent)]
    Db(#[from] db::DbError),
    /// A cryptographic error decrypting the client secret.
    #[error(transparent)]
    Crypto(#[from] crypto::CryptoError),
}
