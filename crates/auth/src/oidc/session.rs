//! Typed view over the session for the in-flight OIDC transaction and the logout
//! hint, mirroring [`crate::MfaSession`] so the `tower_sessions` type stays inside
//! the facade. The transaction (state/nonce/PKCE verifier) and the retained
//! compact ID token live server-side — the only replay-safe location.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::oidc::error::OidcError;
use crate::oidc::flow::OidcTransaction;

const KEY_TRANSACTION: &str = "oidc.txn";
const KEY_LOGOUT: &str = "oidc.logout";

/// What the logout path needs to drive RP-initiated logout: which provider, and
/// the validated compact ID token to send as `id_token_hint`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogoutHint {
    /// The provider slug, to rediscover the `end_session_endpoint`.
    pub provider_slug: String,
    /// The compact ID token retained as `id_token_hint`.
    pub id_token: String,
}

/// A typed view over the session for OIDC state. Construct it as an axum extractor.
pub struct OidcSession(Session);

impl<S> FromRequestParts<S> for OidcSession
where
    S: Send + Sync,
{
    type Rejection = <Session as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(Session::from_request_parts(parts, state).await?))
    }
}

impl OidcSession {
    /// Persist the in-flight transaction before redirecting to the IdP.
    ///
    /// # Errors
    ///
    /// [`OidcError::Session`] if the session cannot be written.
    pub async fn set_transaction(&self, transaction: &OidcTransaction) -> Result<(), OidcError> {
        self.0
            .insert(KEY_TRANSACTION, transaction)
            .await
            .map_err(|_| OidcError::Session)
    }

    /// Consume the in-flight transaction at the callback (single-use).
    ///
    /// # Errors
    ///
    /// [`OidcError::Session`] if the session cannot be written.
    pub async fn take_transaction(&self) -> Result<Option<OidcTransaction>, OidcError> {
        self.0
            .remove(KEY_TRANSACTION)
            .await
            .map_err(|_| OidcError::Session)
    }

    /// Store the logout hint after a successful sign-in.
    ///
    /// # Errors
    ///
    /// [`OidcError::Session`] if the session cannot be written.
    pub async fn set_logout_hint(&self, hint: &LogoutHint) -> Result<(), OidcError> {
        self.0
            .insert(KEY_LOGOUT, hint)
            .await
            .map_err(|_| OidcError::Session)
    }

    /// Consume the logout hint (called before clearing the session on logout).
    ///
    /// # Errors
    ///
    /// [`OidcError::Session`] if the session cannot be written.
    pub async fn take_logout_hint(&self) -> Result<Option<LogoutHint>, OidcError> {
        self.0
            .remove(KEY_LOGOUT)
            .await
            .map_err(|_| OidcError::Session)
    }
}
