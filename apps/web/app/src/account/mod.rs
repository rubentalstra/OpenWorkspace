//! The signed-in user's account & security surface: change password, TOTP
//! enrolment, passkeys, and recovery codes.
//!
//! Every operation is a Leptos `#[server]` fn whose body runs on the server and
//! calls the `auth` facade / `db` for the **current** user — there is no parallel
//! raw HTTP handler. Mutations use [`CsrfClient`](crate::CsrfClient); the passkey
//! ceremony exchanges its WebAuthn JSON as opaque strings so the server-fn
//! boundary stays wasm-safe (the browser ceremony lives in [`crate::webauthn`]).

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use serde::{Deserialize, Serialize};

use crate::CsrfClient;

/// A fresh TOTP enrolment to display: the QR PNG (base64), the `otpauth://` URL,
/// and the secret in base32 for manual entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TotpEnrollDto {
    pub qr_png_base64: String,
    pub otpauth_url: String,
    pub secret_base32: String,
}

/// One-time recovery codes, shown to the user exactly once.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoveryCodesDto {
    pub codes: Vec<String>,
}

/// A registered passkey, for the management list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PasskeyDto {
    pub id: String,
    pub label: Option<String>,
    pub backup_state: bool,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

/// Change the signed-in user's password (re-binds this session, invalidates others).
#[server(client = CsrfClient)]
pub async fn change_password(
    current_password: String,
    new_password: String,
) -> Result<(), ServerFnError> {
    backend::change_password(current_password, new_password).await
}

/// Begin TOTP enrolment: store a pending secret and return the QR / URL / base32.
#[server(client = CsrfClient)]
pub async fn totp_enroll_start() -> Result<TotpEnrollDto, ServerFnError> {
    backend::totp_enroll_start().await
}

/// Confirm TOTP enrolment with a code from the authenticator; returns the issued
/// recovery codes (shown once).
#[server(client = CsrfClient)]
pub async fn totp_enroll_confirm(code: String) -> Result<RecoveryCodesDto, ServerFnError> {
    backend::totp_enroll_confirm(code).await
}

/// Disable TOTP for the signed-in user.
#[server(client = CsrfClient)]
pub async fn totp_disable() -> Result<(), ServerFnError> {
    backend::totp_disable().await
}

/// Replace the signed-in user's recovery codes, returning the new set (shown once).
#[server(client = CsrfClient)]
pub async fn regenerate_recovery_codes() -> Result<RecoveryCodesDto, ServerFnError> {
    backend::regenerate_recovery().await
}

/// The signed-in user's registered passkeys.
#[server(input = GetUrl)]
pub async fn list_passkeys() -> Result<Vec<PasskeyDto>, ServerFnError> {
    backend::list_passkeys().await
}

/// Begin passkey registration; returns the `CreationChallengeResponse` JSON for the
/// browser ceremony.
#[server(client = CsrfClient)]
pub async fn passkey_register_start() -> Result<String, ServerFnError> {
    backend::passkey_register_start().await
}

/// Finish passkey registration from the ceremony's credential JSON.
#[server(client = CsrfClient)]
pub async fn passkey_register_finish(
    credential_json: String,
    label: Option<String>,
) -> Result<(), ServerFnError> {
    backend::passkey_register_finish(credential_json, label).await
}

/// Delete one of the signed-in user's passkeys.
#[server(client = CsrfClient)]
pub async fn delete_passkey(id: String) -> Result<(), ServerFnError> {
    backend::delete_passkey(id).await
}

#[cfg(feature = "ssr")]
mod backend {
    use auth::{
        AuthSession, MfaSession, RegisterPublicKeyCredential, StoredTotp, TotpService, User,
        WebauthnService,
    };
    use leptos::prelude::*;
    use secrecy::SecretString;

    use super::{PasskeyDto, RecoveryCodesDto, TotpEnrollDto};

    fn db() -> db::Db {
        expect_context::<db::Db>()
    }

    fn fail() -> ServerFnError {
        ServerFnError::new("request failed")
    }

    /// Extract the current session and require an authenticated user.
    async fn require_user() -> Result<(AuthSession, User), ServerFnError> {
        let auth_session: AuthSession = leptos_axum::extract().await?;
        let user = auth_session
            .user
            .clone()
            .ok_or_else(|| ServerFnError::new("not authenticated"))?;
        Ok((auth_session, user))
    }

    /// Issue a fresh set of recovery codes, persisting their hashes.
    async fn issue_recovery(
        pool: &db::Db,
        user_id: uuid::Uuid,
    ) -> Result<Vec<String>, ServerFnError> {
        let codes = auth::generate_recovery_codes().map_err(|_| fail())?;
        db::replace_recovery_codes(pool, user_id, &codes.hashes)
            .await
            .map_err(|_| fail())?;
        Ok(codes.plaintext)
    }

    pub(super) async fn change_password(current: String, new: String) -> Result<(), ServerFnError> {
        let (mut auth_session, user) = require_user().await?;
        let new = SecretString::from(new);
        if auth::validate_password(&new).is_err() {
            return Err(ServerFnError::new("new password does not meet the policy"));
        }
        match auth_session
            .backend
            .change_password(user.id.as_uuid(), &SecretString::from(current), &new)
            .await
        {
            Ok(()) => {}
            Err(auth::AuthError::WrongPassword) => {
                return Err(ServerFnError::new("current password is incorrect"));
            }
            Err(_) => return Err(fail()),
        }
        auth::rebind_after_password_change(&mut auth_session, user.id)
            .await
            .map_err(|_| fail())?;
        auth::rotate_csrf_token(&auth_session)
            .await
            .map_err(|_| fail())?;
        Ok(())
    }

    pub(super) async fn totp_enroll_start() -> Result<TotpEnrollDto, ServerFnError> {
        let (_session, user) = require_user().await?;
        let totp = expect_context::<TotpService>();
        let enrollment = totp.start_enrollment(&user.email).map_err(|_| fail())?;
        db::upsert_totp_pending(
            &db(),
            user.id.as_uuid(),
            &enrollment.secret_encrypted,
            enrollment.digits,
            enrollment.period_seconds,
            &enrollment.algorithm,
        )
        .await
        .map_err(|_| fail())?;
        Ok(TotpEnrollDto {
            qr_png_base64: enrollment.qr_png_base64,
            otpauth_url: enrollment.otpauth_url,
            secret_base32: enrollment.secret_base32,
        })
    }

    pub(super) async fn totp_enroll_confirm(
        code: String,
    ) -> Result<RecoveryCodesDto, ServerFnError> {
        let (_session, user) = require_user().await?;
        let pool = db();
        let totp = expect_context::<TotpService>();
        let row = db::load_totp(&pool, user.id.as_uuid())
            .await
            .map_err(|_| fail())?
            .ok_or_else(|| ServerFnError::new("no pending enrolment"))?;
        let stored = StoredTotp {
            secret_encrypted: &row.secret_encrypted,
            digits: row.digits,
            period_seconds: row.period_seconds,
            algorithm: &row.algorithm,
        };
        if !totp.verify(&stored, &code).map_err(|_| fail())? {
            return Err(ServerFnError::new("invalid code"));
        }
        if !db::confirm_totp(&pool, user.id.as_uuid())
            .await
            .map_err(|_| fail())?
        {
            return Err(ServerFnError::new("no pending enrolment"));
        }
        Ok(RecoveryCodesDto {
            codes: issue_recovery(&pool, user.id.as_uuid()).await?,
        })
    }

    pub(super) async fn totp_disable() -> Result<(), ServerFnError> {
        let (_session, user) = require_user().await?;
        db::delete_totp(&db(), user.id.as_uuid())
            .await
            .map_err(|_| fail())
    }

    pub(super) async fn regenerate_recovery() -> Result<RecoveryCodesDto, ServerFnError> {
        let (_session, user) = require_user().await?;
        Ok(RecoveryCodesDto {
            codes: issue_recovery(&db(), user.id.as_uuid()).await?,
        })
    }

    pub(super) async fn list_passkeys() -> Result<Vec<PasskeyDto>, ServerFnError> {
        let (_session, user) = require_user().await?;
        let rows = db::list_passkeys(&db(), user.id.as_uuid())
            .await
            .map_err(|_| fail())?;
        Ok(rows
            .into_iter()
            .map(|p| PasskeyDto {
                id: p.id.to_string(),
                label: p.label,
                backup_state: p.backup_state,
                created_at: p.created_at.to_rfc3339(),
                last_used_at: p.last_used_at.map(|t| t.to_rfc3339()),
            })
            .collect())
    }

    pub(super) async fn passkey_register_start() -> Result<String, ServerFnError> {
        let (_session, user) = require_user().await?;
        let mfa: MfaSession = leptos_axum::extract().await?;
        let webauthn = expect_context::<WebauthnService>();
        let pool = db();
        let identity = db::load_webauthn_identity(&pool, user.id.as_uuid())
            .await
            .map_err(|_| fail())?
            .ok_or_else(fail)?;
        let handle = uuid::Uuid::from_slice(&identity.webauthn_user_handle).map_err(|_| fail())?;
        let exclude: Vec<Vec<u8>> = db::load_passkeys_for_user(&pool, user.id.as_uuid())
            .await
            .map_err(|_| fail())?
            .into_iter()
            .map(|p| p.credential_id)
            .collect();
        let (challenge, state) = webauthn
            .start_registration(handle, &identity.email, &identity.display_name, &exclude)
            .map_err(|_| fail())?;
        mfa.set_passkey_registration(&state)
            .await
            .map_err(|_| fail())?;
        serde_json::to_string(&challenge).map_err(|_| fail())
    }

    pub(super) async fn passkey_register_finish(
        credential_json: String,
        label: Option<String>,
    ) -> Result<(), ServerFnError> {
        let (_session, user) = require_user().await?;
        let mfa: MfaSession = leptos_axum::extract().await?;
        let webauthn = expect_context::<WebauthnService>();
        let credential: RegisterPublicKeyCredential = serde_json::from_str(&credential_json)
            .map_err(|_| ServerFnError::new("invalid credential"))?;
        let state = mfa
            .take_passkey_registration()
            .await
            .map_err(|_| fail())?
            .ok_or_else(|| ServerFnError::new("no registration in progress"))?;
        let registered = webauthn
            .finish_registration(&credential, &state)
            .map_err(|_| fail())?;
        let new = db::NewPasskey {
            user_id: user.id.as_uuid(),
            credential_id: registered.credential_id,
            passkey: registered.passkey_json,
            sign_count: 0,
            aaguid: None,
            transports: Vec::new(),
            backup_eligible: false,
            backup_state: false,
            label,
        };
        match db::insert_passkey(&db(), &new).await {
            Ok(_) => Ok(()),
            Err(db::DbError::Conflict) => Err(ServerFnError::new("passkey already registered")),
            Err(_) => Err(fail()),
        }
    }

    pub(super) async fn delete_passkey(id: String) -> Result<(), ServerFnError> {
        let (_session, user) = require_user().await?;
        let passkey_id = uuid::Uuid::parse_str(&id).map_err(|_| fail())?;
        match db::delete_passkey(&db(), passkey_id, user.id.as_uuid()).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(ServerFnError::new("passkey not found")),
            Err(_) => Err(fail()),
        }
    }
}
