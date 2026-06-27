//! HTTP endpoints for the MFA flows: passkey registration and authentication
//! (username-first and discoverable), TOTP enrolment/verification, and recovery
//! codes.
//!
//! These are thin glue over the `auth` facade — all ceremony, crypto and
//! counter logic (and its tests) live in `crates/auth`. Every POST sits behind
//! the fail-closed CSRF layer, so a browser client must attach `X-CSRF-Token`.

use auth::{
    AuthOutcome, AuthSession, CreationChallengeResponse, MfaSession, PasskeyCandidate, PendingMfa,
    PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse, StoredTotp,
    TotpService, WebauthnService,
};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use db::Db;
use domain::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Map a facade error to a client status. Ceremony-state problems are the
/// client's (stale/missing challenge → 400); a failed assertion, bad code or
/// cloned credential is an auth failure (401); everything else is a 500.
fn auth_status(err: &auth::AuthError) -> StatusCode {
    match err {
        auth::AuthError::CeremonyState => StatusCode::BAD_REQUEST,
        auth::AuthError::Webauthn | auth::AuthError::ClonedCredential | auth::AuthError::Totp => {
            StatusCode::UNAUTHORIZED
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn load_candidates(db: &Db, user_id: Uuid) -> Result<Vec<PasskeyCandidate>, StatusCode> {
    let rows = db::load_passkeys_for_user(db, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(rows
        .into_iter()
        .map(|r| PasskeyCandidate {
            credential_id: r.credential_id,
            passkey: r.passkey,
            sign_count: r.sign_count,
        })
        .collect())
}

/// Persist a successful assertion's advanced counter, then complete the login.
/// A passkey assertion is both factors, so any pending second-factor marker is
/// cleared and the user is signed in.
async fn persist_and_login(
    auth_session: &mut AuthSession,
    mfa: &MfaSession,
    db: &Db,
    user_id: Uuid,
    outcome: &AuthOutcome,
) -> Result<(), ()> {
    let pk = db::load_passkey_by_credential_id(db, &outcome.credential_id)
        .await
        .map_err(|_| ())?
        .ok_or(())?;
    db::update_passkey_after_auth(db, pk.id, &outcome.updated_passkey, outcome.new_sign_count)
        .await
        .map_err(|_| ())?;
    mfa.take_pending_mfa().await.map_err(|_| ())?;
    auth::login_verified_user(auth_session, UserId::new(user_id))
        .await
        .map_err(|_| ())?;
    auth::rotate_csrf_token(auth_session).await.map_err(|_| ())
}

/// Finish the second-factor step for a code-based factor: consume the pending
/// marker, sign the user in, and rotate the CSRF token.
async fn complete_code_login(
    auth_session: &mut AuthSession,
    mfa: &MfaSession,
    user_id: Uuid,
) -> StatusCode {
    if mfa.take_pending_mfa().await.is_err()
        || auth::login_verified_user(auth_session, UserId::new(user_id))
            .await
            .is_err()
        || auth::rotate_csrf_token(auth_session).await.is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

// ---------------------------------------------------------------------------
// Passkey registration (authenticated: add a passkey to the current account).
// ---------------------------------------------------------------------------

pub(crate) async fn register_start(
    auth_session: AuthSession,
    mfa: MfaSession,
    State(webauthn): State<WebauthnService>,
    State(db): State<Db>,
) -> Result<Json<CreationChallengeResponse>, StatusCode> {
    let user = auth_session.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let user_id = user.id.as_uuid();
    let identity = db::load_webauthn_identity(&db, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let handle = Uuid::from_slice(&identity.webauthn_user_handle)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let exclude: Vec<Vec<u8>> = db::load_passkeys_for_user(&db, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|p| p.credential_id)
        .collect();

    let (challenge, state) = webauthn
        .start_registration(handle, &identity.email, &identity.display_name, &exclude)
        .map_err(|e| auth_status(&e))?;
    mfa.set_passkey_registration(&state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(challenge))
}

#[derive(Deserialize)]
pub(crate) struct RegisterFinish {
    credential: RegisterPublicKeyCredential,
    label: Option<String>,
}

pub(crate) async fn register_finish(
    auth_session: AuthSession,
    mfa: MfaSession,
    State(webauthn): State<WebauthnService>,
    State(db): State<Db>,
    Json(body): Json<RegisterFinish>,
) -> StatusCode {
    let Some(user) = auth_session.user else {
        return StatusCode::UNAUTHORIZED;
    };
    let Ok(Some(state)) = mfa.take_passkey_registration().await else {
        return StatusCode::BAD_REQUEST;
    };
    let registered = match webauthn.finish_registration(&body.credential, &state) {
        Ok(r) => r,
        Err(e) => return auth_status(&e),
    };
    let new = db::NewPasskey {
        user_id: user.id.as_uuid(),
        credential_id: registered.credential_id,
        passkey: registered.passkey_json,
        sign_count: 0,
        aaguid: None,
        transports: Vec::new(),
        backup_eligible: false,
        backup_state: false,
        label: body.label,
    };
    match db::insert_passkey(&db, &new).await {
        Ok(_) => StatusCode::OK,
        Err(db::DbError::Conflict) => StatusCode::CONFLICT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Serialize)]
pub(crate) struct PasskeyView {
    id: Uuid,
    label: Option<String>,
    backup_state: bool,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
}

pub(crate) async fn list_passkeys(
    auth_session: AuthSession,
    State(db): State<Db>,
) -> Result<Json<Vec<PasskeyView>>, StatusCode> {
    let user = auth_session.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let views = db::list_passkeys(&db, user.id.as_uuid())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|p| PasskeyView {
            id: p.id,
            label: p.label,
            backup_state: p.backup_state,
            created_at: p.created_at,
            last_used_at: p.last_used_at,
        })
        .collect();
    Ok(Json(views))
}

pub(crate) async fn delete_passkey(
    auth_session: AuthSession,
    State(db): State<Db>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    let Some(user) = auth_session.user else {
        return StatusCode::UNAUTHORIZED;
    };
    match db::delete_passkey(&db, id, user.id.as_uuid()).await {
        Ok(true) => StatusCode::OK,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// ---------------------------------------------------------------------------
// Passkey authentication (passwordless sign-in).
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct AuthenticateStart {
    email: String,
}

pub(crate) async fn authenticate_start(
    mfa: MfaSession,
    State(webauthn): State<WebauthnService>,
    State(db): State<Db>,
    Json(body): Json<AuthenticateStart>,
) -> Result<Json<RequestChallengeResponse>, StatusCode> {
    let user_id = db::load_user_id_by_email(&db, &body.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let candidates = load_candidates(&db, user_id).await?;
    if candidates.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let (challenge, state) = webauthn
        .start_authentication(&candidates)
        .map_err(|e| auth_status(&e))?;
    mfa.set_passkey_authentication(&state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(challenge))
}

pub(crate) async fn authenticate_finish(
    mut auth_session: AuthSession,
    mfa: MfaSession,
    State(webauthn): State<WebauthnService>,
    State(db): State<Db>,
    Json(credential): Json<PublicKeyCredential>,
) -> StatusCode {
    let Ok(Some(state)) = mfa.take_passkey_authentication().await else {
        return StatusCode::BAD_REQUEST;
    };
    let cred_id = credential.get_credential_id().to_vec();
    let owner = match db::load_passkey_by_credential_id(&db, &cred_id).await {
        Ok(Some(p)) => p.user_id,
        Ok(None) => return StatusCode::UNAUTHORIZED,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let candidates = match load_candidates(&db, owner).await {
        Ok(c) => c,
        Err(status) => return status,
    };
    let outcome = match webauthn.finish_authentication(&credential, &state, &candidates) {
        Ok(o) => o,
        Err(e) => return auth_status(&e),
    };
    match persist_and_login(&mut auth_session, &mfa, &db, owner, &outcome).await {
        Ok(()) => StatusCode::OK,
        Err(()) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn discoverable_start(
    mfa: MfaSession,
    State(webauthn): State<WebauthnService>,
) -> Result<Json<RequestChallengeResponse>, StatusCode> {
    let (challenge, state) = webauthn.start_discoverable().map_err(|e| auth_status(&e))?;
    mfa.set_discoverable(&state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(challenge))
}

pub(crate) async fn discoverable_finish(
    mut auth_session: AuthSession,
    mfa: MfaSession,
    State(webauthn): State<WebauthnService>,
    State(db): State<Db>,
    Json(credential): Json<PublicKeyCredential>,
) -> StatusCode {
    let Ok(Some(state)) = mfa.take_discoverable().await else {
        return StatusCode::BAD_REQUEST;
    };
    let handle = match webauthn.identify_discoverable(&credential) {
        Ok(h) => h,
        Err(e) => return auth_status(&e),
    };
    let user_id = match db::load_user_id_by_webauthn_handle(&db, handle.as_bytes()).await {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::UNAUTHORIZED,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let candidates = match load_candidates(&db, user_id).await {
        Ok(c) => c,
        Err(status) => return status,
    };
    let outcome = match webauthn.finish_discoverable(&credential, state, &candidates) {
        Ok(o) => o,
        Err(e) => return auth_status(&e),
    };
    match persist_and_login(&mut auth_session, &mfa, &db, user_id, &outcome).await {
        Ok(()) => StatusCode::OK,
        Err(()) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// ---------------------------------------------------------------------------
// TOTP enrolment and the second-factor challenge.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub(crate) struct TotpEnrollView {
    qr_png_base64: String,
    otpauth_url: String,
    secret_base32: String,
}

pub(crate) async fn totp_enroll_start(
    auth_session: AuthSession,
    State(totp): State<TotpService>,
    State(db): State<Db>,
) -> Result<Json<TotpEnrollView>, StatusCode> {
    let user = auth_session.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let enrollment = totp
        .start_enrollment(&user.email)
        .map_err(|e| auth_status(&e))?;
    db::upsert_totp_pending(
        &db,
        user.id.as_uuid(),
        &enrollment.secret_encrypted,
        enrollment.digits,
        enrollment.period_seconds,
        &enrollment.algorithm,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(TotpEnrollView {
        qr_png_base64: enrollment.qr_png_base64,
        otpauth_url: enrollment.otpauth_url,
        secret_base32: enrollment.secret_base32,
    }))
}

#[derive(Deserialize)]
pub(crate) struct CodeForm {
    code: String,
}

#[derive(Serialize)]
pub(crate) struct RecoveryCodesView {
    recovery_codes: Vec<String>,
}

pub(crate) async fn totp_enroll_confirm(
    auth_session: AuthSession,
    State(totp): State<TotpService>,
    State(db): State<Db>,
    Json(body): Json<CodeForm>,
) -> Result<Json<RecoveryCodesView>, StatusCode> {
    let user = auth_session.user.ok_or(StatusCode::UNAUTHORIZED)?;
    let user_id = user.id.as_uuid();
    let row = db::load_totp(&db, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;
    let stored = StoredTotp {
        secret_encrypted: &row.secret_encrypted,
        digits: row.digits,
        period_seconds: row.period_seconds,
        algorithm: &row.algorithm,
    };
    if !totp
        .verify(&stored, &body.code)
        .map_err(|e| auth_status(&e))?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if !db::confirm_totp(&db, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(Json(RecoveryCodesView {
        recovery_codes: issue_recovery_codes(&db, user_id).await?,
    }))
}

pub(crate) async fn totp_disable(auth_session: AuthSession, State(db): State<Db>) -> StatusCode {
    let Some(user) = auth_session.user else {
        return StatusCode::UNAUTHORIZED;
    };
    match db::delete_totp(&db, user.id.as_uuid()).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn recovery_regenerate(
    auth_session: AuthSession,
    State(db): State<Db>,
) -> Result<Json<RecoveryCodesView>, StatusCode> {
    let user = auth_session.user.ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(RecoveryCodesView {
        recovery_codes: issue_recovery_codes(&db, user.id.as_uuid()).await?,
    }))
}

async fn issue_recovery_codes(db: &Db, user_id: Uuid) -> Result<Vec<String>, StatusCode> {
    let codes = auth::generate_recovery_codes().map_err(|e| auth_status(&e))?;
    db::replace_recovery_codes(db, user_id, &codes.hashes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(codes.plaintext)
}

pub(crate) async fn totp_verify(
    mut auth_session: AuthSession,
    mfa: MfaSession,
    State(totp): State<TotpService>,
    State(db): State<Db>,
    Json(body): Json<CodeForm>,
) -> StatusCode {
    let Ok(Some(pending)) = mfa.peek_pending_mfa().await else {
        return StatusCode::BAD_REQUEST;
    };
    let row = match db::load_confirmed_totp(&db, pending.user_id).await {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::BAD_REQUEST,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let stored = StoredTotp {
        secret_encrypted: &row.secret_encrypted,
        digits: row.digits,
        period_seconds: row.period_seconds,
        algorithm: &row.algorithm,
    };
    match totp.verify(&stored, &body.code) {
        Ok(true) => complete_code_login(&mut auth_session, &mfa, pending.user_id).await,
        Ok(false) => StatusCode::UNAUTHORIZED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn recovery_verify(
    mut auth_session: AuthSession,
    mfa: MfaSession,
    State(db): State<Db>,
    Json(body): Json<CodeForm>,
) -> StatusCode {
    let Ok(Some(pending)) = mfa.peek_pending_mfa().await else {
        return StatusCode::BAD_REQUEST;
    };
    let hash = auth::hash_submitted_code(&body.code);
    match db::consume_recovery_code(&db, pending.user_id, &hash).await {
        Ok(true) => complete_code_login(&mut auth_session, &mfa, pending.user_id).await,
        Ok(false) => StatusCode::UNAUTHORIZED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Build the MFA route subtree. Wired by the server under the auth + CSRF layers.
pub(crate) fn routes() -> axum::Router<crate::AppState> {
    use axum::routing::{delete, get, post};

    axum::Router::new()
        .route("/api/webauthn/register/start", post(register_start))
        .route("/api/webauthn/register/finish", post(register_finish))
        .route("/api/webauthn/passkeys", get(list_passkeys))
        .route("/api/webauthn/passkeys/{id}", delete(delete_passkey))
        .route("/api/webauthn/authenticate/start", post(authenticate_start))
        .route(
            "/api/webauthn/authenticate/finish",
            post(authenticate_finish),
        )
        .route("/api/webauthn/discoverable/start", post(discoverable_start))
        .route(
            "/api/webauthn/discoverable/finish",
            post(discoverable_finish),
        )
        .route("/api/totp/enroll/start", post(totp_enroll_start))
        .route("/api/totp/enroll/confirm", post(totp_enroll_confirm))
        .route("/api/totp/disable", post(totp_disable))
        .route("/api/mfa/totp/verify", post(totp_verify))
        .route("/api/mfa/recovery/verify", post(recovery_verify))
        .route("/api/mfa/recovery/regenerate", post(recovery_regenerate))
}

/// Response body for `/api/login`: signals whether a second factor is owed.
#[derive(Serialize)]
pub(crate) struct LoginResponse {
    pub(crate) mfa_required: bool,
}

/// Build the `PendingMfa` marker for a user who has cleared the password step.
pub(crate) fn pending_for(user_id: Uuid) -> PendingMfa {
    PendingMfa {
        user_id,
        totp: true,
    }
}
