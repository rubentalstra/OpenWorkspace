//! The authentication boundary: the sign-in page and its server functions.
//!
//! Every operation here is a Leptos `#[server]` fn whose body runs on the server
//! and calls the `auth` facade — there is no parallel raw HTTP handler. Reads use
//! `GetUrl` (CSRF-exempt by method); mutations use [`CsrfClient`](crate::CsrfClient).
//! SSO itself is a browser redirect, so it stays the server's `/auth/{slug}/start`
//! + `/callback` routes; the SSO button just links there.

pub mod page;

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use serde::{Deserialize, Serialize};

use crate::CsrfClient;

pub use page::{LoginPage, SignupPage};

/// A sign-in-with provider button descriptor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OidcProviderDto {
    /// `/auth/{slug}/start` key.
    pub slug: String,
    /// Button label (provider override, else display name).
    pub label: String,
    /// Optional icon identifier (unused by the current buttons).
    pub icon: Option<String>,
}

/// The outcome of a password first factor.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LoginOutcome {
    /// `true` when the account owes a confirmed second factor; the client then
    /// completes MFA (TOTP/recovery/passkey) before the session is authenticated.
    pub mfa_required: bool,
}

/// The enabled SSO providers, for rendering sign-in buttons. A real `GET`, so it
/// is CSRF-exempt by method.
#[server(input = GetUrl)]
pub async fn list_oidc_providers() -> Result<Vec<OidcProviderDto>, ServerFnError> {
    backend::list_providers().await
}

/// Verify an email/password first factor. On success the session is authenticated
/// (unless a second factor is owed).
#[server(client = CsrfClient)]
pub async fn login(email: String, password: String) -> Result<LoginOutcome, ServerFnError> {
    backend::login(email, password).await
}

/// Complete a pending sign-in with an authenticator (TOTP) code.
#[server(client = CsrfClient)]
pub async fn verify_totp(code: String) -> Result<(), ServerFnError> {
    backend::verify_totp(code).await
}

/// Complete a pending sign-in with a one-time recovery code.
#[server(client = CsrfClient)]
pub async fn verify_recovery(code: String) -> Result<(), ServerFnError> {
    backend::verify_recovery(code).await
}

/// Clear the session; returns the IdP logout URL to navigate to, if the session
/// came from an SSO provider that offers RP-initiated logout.
#[server(client = CsrfClient)]
pub async fn logout() -> Result<Option<String>, ServerFnError> {
    backend::logout().await
}

/// Begin a passwordless passkey sign-in for `email`; returns the
/// `RequestChallengeResponse` JSON (the account's credentials) for the ceremony.
#[server(client = CsrfClient)]
pub async fn passkey_login_start(email: String) -> Result<String, ServerFnError> {
    backend::passkey_login_start(email).await
}

/// Finish a passwordless passkey sign-in from the ceremony's assertion JSON.
#[server(client = CsrfClient)]
pub async fn passkey_login_finish(credential_json: String) -> Result<(), ServerFnError> {
    backend::passkey_login_finish(credential_json).await
}

#[cfg(feature = "ssr")]
mod backend {
    use auth::{
        AuthSession, Credentials, MfaSession, OidcSession, PasskeyCandidate, PasswordOutcome,
        ProviderRegistry, PublicKeyCredential, StoredTotp, TotpService,
    };
    use leptos::prelude::*;
    use secrecy::SecretString;

    use super::{LoginOutcome, OidcProviderDto};

    /// A generic failure message: never reveal whether the email exists or which
    /// field was wrong, and never leak an internal error.
    const SIGN_IN_FAILED: &str = "sign-in failed";
    /// The one client-actionable first-factor outcome the page distinguishes.
    const BAD_CREDENTIALS: &str = "invalid email or password";
    /// Shown when a second-factor code does not verify.
    const BAD_CODE: &str = "invalid code";

    fn db() -> db::Db {
        expect_context::<db::Db>()
    }

    fn fail() -> ServerFnError {
        ServerFnError::new(SIGN_IN_FAILED)
    }

    pub(super) async fn list_providers() -> Result<Vec<OidcProviderDto>, ServerFnError> {
        let rows = db::load_enabled_provider_summaries(&db())
            .await
            .map_err(|_| fail())?;
        Ok(rows
            .into_iter()
            .map(|r| OidcProviderDto {
                slug: r.slug,
                label: r.button_label.unwrap_or(r.display_name),
                icon: r.icon,
            })
            .collect())
    }

    pub(super) async fn login(
        email: String,
        password: String,
    ) -> Result<LoginOutcome, ServerFnError> {
        let mut auth_session: AuthSession = leptos_axum::extract().await?;
        let mfa_session: MfaSession = leptos_axum::extract().await?;
        let creds = Credentials {
            email,
            password: SecretString::from(password),
        };
        match auth::password_first_factor(&mut auth_session, &mfa_session, &db(), creds).await {
            Ok(PasswordOutcome::Authenticated) => Ok(LoginOutcome {
                mfa_required: false,
            }),
            Ok(PasswordOutcome::MfaRequired) => Ok(LoginOutcome { mfa_required: true }),
            Ok(PasswordOutcome::InvalidCredentials) => Err(ServerFnError::new(BAD_CREDENTIALS)),
            Err(_) => Err(fail()),
        }
    }

    pub(super) async fn verify_totp(code: String) -> Result<(), ServerFnError> {
        let mut auth_session: AuthSession = leptos_axum::extract().await?;
        let mfa_session: MfaSession = leptos_axum::extract().await?;
        let pool = db();
        let totp = expect_context::<TotpService>();

        let pending = mfa_session
            .peek_pending_mfa()
            .await
            .map_err(|_| fail())?
            .ok_or_else(|| ServerFnError::new("no pending sign-in"))?;
        let row = db::load_confirmed_totp(&pool, pending.user_id)
            .await
            .map_err(|_| fail())?
            .ok_or_else(fail)?;
        let stored = StoredTotp {
            secret_encrypted: &row.secret_encrypted,
            digits: row.digits,
            period_seconds: row.period_seconds,
            algorithm: &row.algorithm,
        };
        if totp.verify(&stored, &code).map_err(|_| fail())? {
            auth::complete_second_factor(
                &mut auth_session,
                &mfa_session,
                domain::UserId::new(pending.user_id),
            )
            .await
            .map_err(|_| fail())?;
            Ok(())
        } else {
            Err(ServerFnError::new(BAD_CODE))
        }
    }

    pub(super) async fn verify_recovery(code: String) -> Result<(), ServerFnError> {
        let mut auth_session: AuthSession = leptos_axum::extract().await?;
        let mfa_session: MfaSession = leptos_axum::extract().await?;
        let pool = db();

        let pending = mfa_session
            .peek_pending_mfa()
            .await
            .map_err(|_| fail())?
            .ok_or_else(|| ServerFnError::new("no pending sign-in"))?;
        let hash = auth::hash_submitted_code(&code);
        if db::consume_recovery_code(&pool, pending.user_id, &hash)
            .await
            .map_err(|_| fail())?
        {
            auth::complete_second_factor(
                &mut auth_session,
                &mfa_session,
                domain::UserId::new(pending.user_id),
            )
            .await
            .map_err(|_| fail())?;
            Ok(())
        } else {
            Err(ServerFnError::new(BAD_CODE))
        }
    }

    pub(super) async fn logout() -> Result<Option<String>, ServerFnError> {
        let mut auth_session: AuthSession = leptos_axum::extract().await?;
        let oidc_session: OidcSession = leptos_axum::extract().await?;
        let hint = oidc_session.take_logout_hint().await.ok().flatten();

        auth::sign_out(&mut auth_session)
            .await
            .map_err(|_| ServerFnError::new("sign-out failed"))?;

        // For an SSO session, hand back the provider's RP-initiated logout URL so
        // the client can navigate to it and end the IdP session too.
        let Some(hint) = hint else {
            return Ok(None);
        };
        let registry = expect_context::<ProviderRegistry>();
        let Ok(provider) = registry.discovered(&hint.provider_slug).await else {
            return Ok(None);
        };
        let base = expect_context::<crate::PublicBaseUrl>().0;
        let post_logout = format!("{base}/login");
        let state = uuid::Uuid::new_v4().simple().to_string();
        Ok(auth::logout_url(
            &provider,
            &hint.id_token,
            &post_logout,
            &state,
        ))
    }

    async fn load_candidates(
        pool: &db::Db,
        user_id: uuid::Uuid,
    ) -> Result<Vec<PasskeyCandidate>, ServerFnError> {
        Ok(db::load_passkeys_for_user(pool, user_id)
            .await
            .map_err(|_| fail())?
            .into_iter()
            .map(|r| PasskeyCandidate {
                credential_id: r.credential_id,
                passkey: r.passkey,
                sign_count: r.sign_count,
            })
            .collect())
    }

    pub(super) async fn passkey_login_start(email: String) -> Result<String, ServerFnError> {
        let mfa: MfaSession = leptos_axum::extract().await?;
        let webauthn = expect_context::<auth::WebauthnService>();
        let pool = db();
        let no_passkey = || ServerFnError::new("no passkey is registered for that email");

        let user_id = db::load_user_id_by_email(&pool, &email)
            .await
            .map_err(|_| fail())?
            .ok_or_else(no_passkey)?;
        let candidates = load_candidates(&pool, user_id).await?;
        if candidates.is_empty() {
            return Err(no_passkey());
        }
        let (challenge, state) = webauthn
            .start_authentication(&candidates)
            .map_err(|_| fail())?;
        mfa.set_passkey_authentication(&state)
            .await
            .map_err(|_| fail())?;
        serde_json::to_string(&challenge).map_err(|_| fail())
    }

    pub(super) async fn passkey_login_finish(credential_json: String) -> Result<(), ServerFnError> {
        let mut auth_session: AuthSession = leptos_axum::extract().await?;
        let mfa: MfaSession = leptos_axum::extract().await?;
        let webauthn = expect_context::<auth::WebauthnService>();
        let pool = db();

        let credential: PublicKeyCredential = serde_json::from_str(&credential_json)
            .map_err(|_| ServerFnError::new("invalid credential"))?;
        let state = mfa
            .take_passkey_authentication()
            .await
            .map_err(|_| fail())?
            .ok_or_else(|| ServerFnError::new("no sign-in in progress"))?;
        // The credential id identifies the owner; verify against their candidates.
        let owner = db::load_passkey_by_credential_id(&pool, credential.get_credential_id())
            .await
            .map_err(|_| fail())?
            .ok_or_else(|| ServerFnError::new("unknown passkey"))?
            .user_id;
        let candidates = load_candidates(&pool, owner).await?;
        let outcome = webauthn
            .finish_authentication(&credential, &state, &candidates)
            .map_err(|_| fail())?;

        let passkey = db::load_passkey_by_credential_id(&pool, &outcome.credential_id)
            .await
            .map_err(|_| fail())?
            .ok_or_else(fail)?;
        db::update_passkey_after_auth(
            &pool,
            passkey.id,
            &outcome.updated_passkey,
            outcome.new_sign_count,
        )
        .await
        .map_err(|_| fail())?;
        auth::login_verified_user(&mut auth_session, domain::UserId::new(owner))
            .await
            .map_err(|_| fail())?;
        auth::rotate_csrf_token(&auth_session)
            .await
            .map_err(|_| fail())?;
        Ok(())
    }
}
