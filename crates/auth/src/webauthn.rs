//! WebAuthn passkey facade over `webauthn-rs`.
//!
//! The vendor `Webauthn` engine and result types stay inside this module. The
//! server exchanges only: the W3C wire DTOs ([`CreationChallengeResponse`],
//! [`RequestChallengeResponse`], [`RegisterPublicKeyCredential`],
//! [`PublicKeyCredential`]) re-exported as the ceremony API; the opaque ceremony
//! *state* types persisted server-side in the session; and the first-party
//! [`RegisteredPasskey`] / [`AuthOutcome`] / [`PasskeyCandidate`] structs below.
//!
//! Stored passkeys are carried as `serde_json::Value` (the `passkeys.passkey`
//! jsonb); this module owns the (de)serialization to the vendor `Passkey`.

use std::sync::Arc;

use uuid::Uuid;
use webauthn_rs::prelude::{CredentialID, DiscoverableKey, Passkey, Url, WebauthnError};
use webauthn_rs::{Webauthn, WebauthnBuilder};

// Re-exported ceremony surface (W3C wire DTOs + opaque server-side state).
pub use webauthn_rs::prelude::{
    CreationChallengeResponse, DiscoverableAuthentication, PasskeyAuthentication,
    PasskeyRegistration, PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse,
};

use crate::AuthError;

/// A registered passkey ready to persist (raw credential id + serialized blob).
#[derive(Clone, Debug)]
pub struct RegisteredPasskey {
    /// Globally-unique raw credential id.
    pub credential_id: Vec<u8>,
    /// Serialized webauthn-rs `Passkey` for the `passkeys.passkey` column.
    pub passkey_json: serde_json::Value,
}

/// A stored passkey offered as a candidate during authentication.
#[derive(Clone, Debug)]
pub struct PasskeyCandidate {
    /// Raw credential id.
    pub credential_id: Vec<u8>,
    /// Serialized webauthn-rs `Passkey`.
    pub passkey: serde_json::Value,
    /// Last persisted signature counter.
    pub sign_count: i64,
}

/// The result of a successful authentication: which credential was used, its
/// re-serialized state, the advanced counter, and whether the user was verified.
#[derive(Clone, Debug)]
pub struct AuthOutcome {
    /// Raw credential id that authenticated.
    pub credential_id: Vec<u8>,
    /// Re-serialized `Passkey` (counter/backup advanced) to persist.
    pub updated_passkey: serde_json::Value,
    /// New signature counter to persist.
    pub new_sign_count: i64,
    /// Whether the authenticator asserted user verification (PIN/biometric).
    pub user_verified: bool,
}

/// WebAuthn relying-party engine, built once from configuration.
#[derive(Clone)]
pub struct WebauthnService {
    webauthn: Arc<Webauthn>,
}

impl WebauthnService {
    /// Build the engine for the given relying-party id, origin and display name.
    ///
    /// # Errors
    ///
    /// [`AuthError::Config`] if the origin is not a valid URL or the RP id is not
    /// an effective domain of it.
    pub fn new(rp_id: &str, rp_origin: &str, rp_name: &str) -> Result<Self, AuthError> {
        let origin = Url::parse(rp_origin).map_err(|_| AuthError::Config)?;
        let webauthn = WebauthnBuilder::new(rp_id, &origin)
            .map_err(|_| AuthError::Config)?
            .rp_name(rp_name)
            .build()
            .map_err(|_| AuthError::Config)?;
        Ok(Self {
            webauthn: Arc::new(webauthn),
        })
    }

    /// Begin registering a new passkey for `user_handle` (the WebAuthn user id),
    /// excluding the user's existing credentials so a device cannot double-register.
    ///
    /// # Errors
    ///
    /// [`AuthError::Webauthn`] if the ceremony cannot be started.
    pub fn start_registration(
        &self,
        user_handle: Uuid,
        user_name: &str,
        display_name: &str,
        exclude: &[Vec<u8>],
    ) -> Result<(CreationChallengeResponse, PasskeyRegistration), AuthError> {
        let exclude_ids: Vec<CredentialID> =
            exclude.iter().cloned().map(CredentialID::from).collect();
        let exclude_opt = (!exclude_ids.is_empty()).then_some(exclude_ids);
        self.webauthn
            .start_passkey_registration(user_handle, user_name, display_name, exclude_opt)
            .map_err(|e| log_webauthn(&e))
    }

    /// Complete registration, yielding the credential to persist.
    ///
    /// # Errors
    ///
    /// [`AuthError::Webauthn`] if the attestation is invalid;
    /// [`AuthError::Serialization`] if the credential cannot be serialized.
    pub fn finish_registration(
        &self,
        credential: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> Result<RegisteredPasskey, AuthError> {
        let passkey = self
            .webauthn
            .finish_passkey_registration(credential, state)
            .map_err(|e| log_webauthn(&e))?;
        let credential_id = passkey.cred_id().as_ref().to_vec();
        let passkey_json = serde_json::to_value(&passkey).map_err(|_| AuthError::Serialization)?;
        Ok(RegisteredPasskey {
            credential_id,
            passkey_json,
        })
    }

    /// Begin username-first authentication against the user's known credentials.
    ///
    /// # Errors
    ///
    /// [`AuthError::Serialization`] if a stored credential is corrupt;
    /// [`AuthError::Webauthn`] if the ceremony cannot be started.
    pub fn start_authentication(
        &self,
        candidates: &[PasskeyCandidate],
    ) -> Result<(RequestChallengeResponse, PasskeyAuthentication), AuthError> {
        let passkeys = deserialize_passkeys(candidates)?;
        self.webauthn
            .start_passkey_authentication(&passkeys)
            .map_err(|e| log_webauthn(&e))
    }

    /// Complete username-first authentication, returning the credential to update.
    ///
    /// # Errors
    ///
    /// [`AuthError::Webauthn`] on assertion failure; [`AuthError::ClonedCredential`]
    /// on a counter regression; [`AuthError::Serialization`] on a corrupt credential.
    pub fn finish_authentication(
        &self,
        credential: &PublicKeyCredential,
        state: &PasskeyAuthentication,
        candidates: &[PasskeyCandidate],
    ) -> Result<AuthOutcome, AuthError> {
        let result = self
            .webauthn
            .finish_passkey_authentication(credential, state)
            .map_err(|e| log_webauthn(&e))?;
        apply_auth_result(&result, candidates)
    }

    /// Begin discoverable (usernameless) authentication.
    ///
    /// # Errors
    ///
    /// [`AuthError::Webauthn`] if the ceremony cannot be started.
    pub fn start_discoverable(
        &self,
    ) -> Result<(RequestChallengeResponse, DiscoverableAuthentication), AuthError> {
        self.webauthn
            .start_discoverable_authentication()
            .map_err(|e| log_webauthn(&e))
    }

    /// Extract the user handle a discoverable assertion claims, so the caller can
    /// load that user's credentials before finishing.
    ///
    /// # Errors
    ///
    /// [`AuthError::Webauthn`] if the response cannot be pre-processed.
    pub fn identify_discoverable(&self, credential: &PublicKeyCredential) -> Result<Uuid, AuthError> {
        let (user_handle, _cred_id) = self
            .webauthn
            .identify_discoverable_authentication(credential)
            .map_err(|e| log_webauthn(&e))?;
        Ok(user_handle)
    }

    /// Complete discoverable authentication against the identified user's credentials.
    ///
    /// # Errors
    ///
    /// [`AuthError::Webauthn`] on assertion failure; [`AuthError::ClonedCredential`]
    /// on a counter regression; [`AuthError::Serialization`] on a corrupt credential.
    pub fn finish_discoverable(
        &self,
        credential: &PublicKeyCredential,
        state: DiscoverableAuthentication,
        candidates: &[PasskeyCandidate],
    ) -> Result<AuthOutcome, AuthError> {
        let passkeys = deserialize_passkeys(candidates)?;
        let keys: Vec<DiscoverableKey> = passkeys.iter().map(DiscoverableKey::from).collect();
        let result = self
            .webauthn
            .finish_discoverable_authentication(credential, state, &keys)
            .map_err(|e| log_webauthn(&e))?;
        apply_auth_result(&result, candidates)
    }
}

fn deserialize_passkeys(candidates: &[PasskeyCandidate]) -> Result<Vec<Passkey>, AuthError> {
    candidates
        .iter()
        .map(|c| serde_json::from_value(c.passkey.clone()).map_err(|_| AuthError::Serialization))
        .collect()
}

fn apply_auth_result(
    result: &webauthn_rs::prelude::AuthenticationResult,
    candidates: &[PasskeyCandidate],
) -> Result<AuthOutcome, AuthError> {
    let used = result.cred_id().as_ref();
    let matched = candidates
        .iter()
        .find(|c| c.credential_id.as_slice() == used)
        .ok_or(AuthError::Webauthn)?;

    // Clone detection: a non-zero counter must strictly exceed the stored one.
    if result.counter() > 0 && i64::from(result.counter()) <= matched.sign_count {
        return Err(AuthError::ClonedCredential);
    }

    let mut passkey: Passkey =
        serde_json::from_value(matched.passkey.clone()).map_err(|_| AuthError::Serialization)?;
    passkey.update_credential(result);
    let updated_passkey = serde_json::to_value(&passkey).map_err(|_| AuthError::Serialization)?;

    Ok(AuthOutcome {
        credential_id: matched.credential_id.clone(),
        updated_passkey,
        new_sign_count: i64::from(result.counter()),
        user_verified: result.user_verified(),
    })
}

fn log_webauthn(err: &WebauthnError) -> AuthError {
    tracing::warn!(error = %err, "webauthn ceremony failed");
    AuthError::Webauthn
}

#[cfg(test)]
mod tests {
    use super::*;
    use webauthn_authenticator_rs::WebauthnAuthenticator;
    use webauthn_authenticator_rs::softpasskey::SoftPasskey;

    const RP_ID: &str = "localhost";
    const ORIGIN: &str = "http://localhost:3000";

    fn service() -> WebauthnService {
        WebauthnService::new(RP_ID, ORIGIN, "OpenWorkspace").unwrap()
    }

    fn origin() -> Url {
        Url::parse(ORIGIN).unwrap()
    }

    /// Drive a full registration with a software authenticator and return the
    /// credential as it would be stored.
    fn register(
        svc: &WebauthnService,
        wa: &mut WebauthnAuthenticator<SoftPasskey>,
        handle: Uuid,
    ) -> PasskeyCandidate {
        let (ccr, state) = svc
            .start_registration(handle, "claire@example.test", "Claire", &[])
            .unwrap();
        let credential = wa.do_registration(origin(), ccr).unwrap();
        let registered = svc.finish_registration(&credential, &state).unwrap();
        PasskeyCandidate {
            credential_id: registered.credential_id,
            passkey: registered.passkey_json,
            sign_count: 0,
        }
    }

    #[test]
    fn register_then_authenticate_round_trip() {
        let svc = service();
        let mut wa = WebauthnAuthenticator::new(SoftPasskey::new(true));
        let candidate = register(&svc, &mut wa, Uuid::new_v4());
        assert!(!candidate.credential_id.is_empty());

        let (rcr, state) = svc.start_authentication(std::slice::from_ref(&candidate)).unwrap();
        let assertion = wa.do_authentication(origin(), rcr).unwrap();
        let outcome = svc
            .finish_authentication(&assertion, &state, std::slice::from_ref(&candidate))
            .unwrap();

        assert_eq!(outcome.credential_id, candidate.credential_id);
        assert!(outcome.user_verified, "soft authenticator asserts user verification");
        // The signature counter advanced past the stored value and is returned to persist.
        assert!(outcome.new_sign_count > candidate.sign_count);
    }

    #[test]
    fn counter_regression_is_rejected_as_cloned() {
        let svc = service();
        let mut wa = WebauthnAuthenticator::new(SoftPasskey::new(true));
        let candidate = register(&svc, &mut wa, Uuid::new_v4());

        // Pretend we previously recorded a far higher counter than the
        // authenticator will now assert — the hallmark of a cloned credential.
        let stale = PasskeyCandidate {
            sign_count: i64::MAX,
            ..candidate.clone()
        };
        let (rcr, state) = svc.start_authentication(std::slice::from_ref(&candidate)).unwrap();
        let assertion = wa.do_authentication(origin(), rcr).unwrap();
        let err = svc
            .finish_authentication(&assertion, &state, std::slice::from_ref(&stale))
            .unwrap_err();
        assert!(matches!(err, AuthError::ClonedCredential));
    }

    #[test]
    fn unknown_credential_is_rejected() {
        // An assertion whose credential is not among the candidates is refused.
        let svc = service();
        let mut wa = WebauthnAuthenticator::new(SoftPasskey::new(true));
        let candidate = register(&svc, &mut wa, Uuid::new_v4());

        let (rcr, state) = svc.start_authentication(std::slice::from_ref(&candidate)).unwrap();
        let assertion = wa.do_authentication(origin(), rcr).unwrap();
        let err = svc.finish_authentication(&assertion, &state, &[]).unwrap_err();
        assert!(matches!(err, AuthError::Webauthn));
    }

    // The discoverable (usernameless) ceremony cannot be driven by `SoftPasskey`:
    // it is a U2F-style authenticator that requires a non-empty allow-list and
    // returns no user handle, so an empty-allow-list challenge fails inside the
    // authenticator. The server-side half is still covered — `start_discoverable`
    // below, and `finish_discoverable` shares `apply_auth_result` with the
    // username-first `finish_authentication` exercised above. End-to-end coverage
    // needs a CTAP2 resident-key authenticator (or a browser), out of scope here.
    #[test]
    fn discoverable_challenge_starts() {
        let svc = service();
        let (rcr, _state) = svc.start_discoverable().unwrap();
        // A discoverable request carries no allow-list (the client discovers it).
        let json = serde_json::to_value(&rcr).unwrap();
        let allow = &json["publicKey"]["allowCredentials"];
        assert!(allow.is_null() || allow.as_array().is_some_and(Vec::is_empty));
    }
}
