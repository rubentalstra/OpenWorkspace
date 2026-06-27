//! The single, reused outbound HTTP client for OIDC discovery, JWKS, token
//! exchange and userinfo. It owns the `reqwest::Client` so no vendor HTTP type
//! escapes the facade, and it installs the aws-lc-rs rustls provider once.

use std::sync::Once;
use std::time::Duration;

use crate::oidc::error::OidcError;

/// Installs aws-lc-rs as the process-wide rustls `CryptoProvider` exactly once.
///
/// The `reqwest` `*-no-provider` rustls feature ships rustls without a default
/// provider, so one must be installed before the first TLS handshake. This is
/// idempotent: if another component (e.g. the sqlx pool) installed a provider
/// first, `install_default` returns `Err` and we simply reuse the existing one.
fn ensure_crypto_provider() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        if rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .is_err()
        {
            tracing::debug!("a rustls CryptoProvider was already installed; reusing it");
        }
    });
}

/// The first-party outbound HTTP client for OIDC. Clone-cheap (reqwest is `Arc`
/// inside); built once at startup and shared into the [`super::provider::ProviderRegistry`].
#[derive(Clone, Debug)]
pub struct OidcHttpClient {
    client: reqwest::Client,
}

impl OidcHttpClient {
    /// Build the client with redirects disabled (an SSRF guard the OIDC crates
    /// require) and a per-request `timeout`, installing the crypto provider first.
    ///
    /// # Errors
    ///
    /// [`OidcError::Http`] if the underlying client cannot be constructed.
    pub fn new(timeout: Duration) -> Result<Self, OidcError> {
        ensure_crypto_provider();
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(timeout)
            .build()
            .map_err(|_| OidcError::Http)?;
        Ok(Self { client })
    }

    /// The underlying client, passed by reference into the openidconnect
    /// `*_async` calls. `pub(crate)` so it never leaves the `auth` crate.
    pub(crate) fn inner(&self) -> &reqwest::Client {
        &self.client
    }
}
