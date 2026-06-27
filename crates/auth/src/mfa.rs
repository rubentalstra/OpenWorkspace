//! MFA session state and the post-password step decision.
//!
//! In-progress WebAuthn ceremony state and the "password verified, second factor
//! pending" marker live server-side in the `tower-sessions` session — the only
//! place that is safe from replay. [`MfaSession`] is a typed view over that
//! session so the vendor session type stays inside this facade; the server uses
//! it as an extractor and shuttles opaque ceremony state between start and finish.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use db::Db;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use uuid::Uuid;

use crate::AuthError;
use crate::webauthn::{DiscoverableAuthentication, PasskeyAuthentication, PasskeyRegistration};

const KEY_PENDING_MFA: &str = "mfa.pending";
const KEY_PASSKEY_REG: &str = "mfa.passkey_reg";
const KEY_PASSKEY_AUTH: &str = "mfa.passkey_auth";
const KEY_DISCOVERABLE: &str = "mfa.discoverable";

/// State held between a verified password and a cleared second factor. While this
/// is present the user is *not* logged in; only a passing second factor calls
/// `login`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingMfa {
    /// The user who cleared the first factor.
    pub user_id: Uuid,
    /// Whether a confirmed TOTP enrolment is available to challenge.
    pub totp: bool,
}

/// A typed view over the session for in-progress MFA state. Construct it as an
/// axum extractor; the methods (de)serialize state under fixed keys.
pub struct MfaSession(Session);

impl<S> FromRequestParts<S> for MfaSession
where
    S: Send + Sync,
{
    type Rejection = <Session as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(Session::from_request_parts(parts, state).await?))
    }
}

impl MfaSession {
    /// Record that a user has cleared the first factor and now owes a second.
    ///
    /// # Errors
    ///
    /// [`AuthError::CeremonyState`] if the session cannot be written.
    pub async fn set_pending_mfa(&self, pending: &PendingMfa) -> Result<(), AuthError> {
        self.insert(KEY_PENDING_MFA, pending).await
    }

    /// Read the pending-MFA marker without consuming it.
    ///
    /// # Errors
    ///
    /// [`AuthError::CeremonyState`] if the session cannot be read.
    pub async fn peek_pending_mfa(&self) -> Result<Option<PendingMfa>, AuthError> {
        self.get(KEY_PENDING_MFA).await
    }

    /// Consume the pending-MFA marker (on successful second-factor completion).
    ///
    /// # Errors
    ///
    /// [`AuthError::CeremonyState`] if the session cannot be written.
    pub async fn take_pending_mfa(&self) -> Result<Option<PendingMfa>, AuthError> {
        self.remove(KEY_PENDING_MFA).await
    }

    /// Stash passkey-registration ceremony state.
    ///
    /// # Errors
    ///
    /// [`AuthError::CeremonyState`] if the session cannot be written.
    pub async fn set_passkey_registration(
        &self,
        state: &PasskeyRegistration,
    ) -> Result<(), AuthError> {
        self.insert(KEY_PASSKEY_REG, state).await
    }

    /// Consume passkey-registration ceremony state.
    ///
    /// # Errors
    ///
    /// [`AuthError::CeremonyState`] if the session cannot be written.
    pub async fn take_passkey_registration(
        &self,
    ) -> Result<Option<PasskeyRegistration>, AuthError> {
        self.remove(KEY_PASSKEY_REG).await
    }

    /// Stash passkey-authentication ceremony state.
    ///
    /// # Errors
    ///
    /// [`AuthError::CeremonyState`] if the session cannot be written.
    pub async fn set_passkey_authentication(
        &self,
        state: &PasskeyAuthentication,
    ) -> Result<(), AuthError> {
        self.insert(KEY_PASSKEY_AUTH, state).await
    }

    /// Consume passkey-authentication ceremony state.
    ///
    /// # Errors
    ///
    /// [`AuthError::CeremonyState`] if the session cannot be written.
    pub async fn take_passkey_authentication(
        &self,
    ) -> Result<Option<PasskeyAuthentication>, AuthError> {
        self.remove(KEY_PASSKEY_AUTH).await
    }

    /// Stash discoverable-authentication ceremony state.
    ///
    /// # Errors
    ///
    /// [`AuthError::CeremonyState`] if the session cannot be written.
    pub async fn set_discoverable(
        &self,
        state: &DiscoverableAuthentication,
    ) -> Result<(), AuthError> {
        self.insert(KEY_DISCOVERABLE, state).await
    }

    /// Consume discoverable-authentication ceremony state.
    ///
    /// # Errors
    ///
    /// [`AuthError::CeremonyState`] if the session cannot be written.
    pub async fn take_discoverable(
        &self,
    ) -> Result<Option<DiscoverableAuthentication>, AuthError> {
        self.remove(KEY_DISCOVERABLE).await
    }

    async fn insert<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), AuthError> {
        self.0
            .insert(key, value)
            .await
            .map_err(|_| AuthError::CeremonyState)
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, AuthError> {
        self.0.get(key).await.map_err(|_| AuthError::CeremonyState)
    }

    async fn remove<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, AuthError> {
        self.0
            .remove(key)
            .await
            .map_err(|_| AuthError::CeremonyState)
    }
}

/// Whether `user_id` must clear a second factor after a correct password.
///
/// True when the user has a confirmed TOTP enrolment (the post-password
/// challenge is TOTP or a recovery code). Instance-wide `require_mfa` *enrolment*
/// enforcement — forcing a user without any factor to set one up — is a UX
/// concern deferred with the security settings UI.
///
/// # Errors
///
/// [`AuthError::Db`] on any database error.
pub async fn second_factor_required(db: &Db, user_id: Uuid) -> Result<bool, AuthError> {
    Ok(db::totp_is_confirmed(db, user_id).await?)
}
