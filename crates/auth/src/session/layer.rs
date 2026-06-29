//! Session-store and auth-manager layer assembly.

use std::time::Duration;

use axum_login::{AuthManagerLayer, AuthManagerLayerBuilder, AuthnBackend as _};
use db::Db;
use secrecy::SecretString;
use tower_sessions::cookie::SameSite;
use tower_sessions::{Expiry, SessionManagerLayer};

use crate::{Backend, PgSessionStore};

/// The session cookie name. `owk` = OpenWorkspace.
const SESSION_COOKIE_NAME: &str = "owk.sid";

/// The fully-typed `axum-login` session layer over the Postgres session store.
pub type AuthSession = axum_login::AuthSession<Backend>;

/// Error rotating the session id during a re-authentication reset.
#[derive(Debug, thiserror::Error)]
#[error("failed to cycle the session id")]
pub struct ReauthError;

/// Forces a fresh session id before a re-login, defeating fixation on the
/// re-auth/step-up path.
///
/// `axum-login`'s `login` only cycles the session id when the session is not
/// already authenticated, so re-logging-in over a live authenticated cookie
/// would otherwise keep the old id. Call this before `authenticate`/`login` when
/// the request is already authenticated: it cycles the id so the old id no longer
/// loads. Takes the first-party [`AuthSession`] so no `tower_sessions` type leaks.
///
/// # Errors
///
/// Returns [`ReauthError`] if cycling the underlying session id fails.
pub async fn cycle_session_id(auth_session: &AuthSession) -> Result<(), ReauthError> {
    auth_session
        .session
        .cycle_id()
        .await
        .map_err(|_| ReauthError)
}

/// Re-binds the **current** session after the signed-in user's password changed.
///
/// A password change rotates `session_auth_hash`. The current session's *stored*
/// auth hash is still the pre-change value, so without this it would be treated
/// as stale and flushed on its very next request — logging out the user who just
/// changed their own password. This reloads the user (now carrying the new hash)
/// from the backend, cycles the session id (fixation defence), and re-logs them
/// in, which re-stamps the stored auth hash to the new value. Every *other*
/// session keeps the old stored hash and is correctly invalidated on its next
/// request.
///
/// Call after [`Backend::change_password`] succeeds. The `login` call lives here,
/// inside the facade, so no `axum-login` method is invoked from app code beyond
/// the existing login/logout handlers.
///
/// # Errors
///
/// Returns [`ReauthError`] if reloading the user, cycling the id, or re-logging in
/// fails (including the user vanishing between the change and the reload).
pub async fn rebind_after_password_change(
    auth_session: &mut AuthSession,
    user_id: domain::UserId,
) -> Result<(), ReauthError> {
    let user = auth_session
        .backend
        .get_user(&user_id.as_uuid())
        .await
        .map_err(|_| ReauthError)?
        .ok_or(ReauthError)?;
    auth_session
        .session
        .cycle_id()
        .await
        .map_err(|_| ReauthError)?;
    auth_session.login(&user).await.map_err(|_| ReauthError)?;
    Ok(())
}

/// Completes a login for a user already proven by a non-password factor (a TOTP
/// code, a recovery code, or a passkey assertion) — the second step of the MFA
/// flow, and the only sign-in path for passwordless passkeys.
///
/// The caller verifies the factor, then calls this with the user's id. It reloads
/// the user from the backend and logs them in; because the pending session is not
/// yet authenticated, `axum-login`'s `login` cycles the session id itself,
/// retiring the pre-auth (MFA-pending) id. The caller should rotate the CSRF
/// token afterwards, as on the password path. The `login` call stays inside the
/// facade so no `axum-login` method is invoked from app code.
///
/// # Errors
///
/// Returns [`ReauthError`] if the user cannot be reloaded (including a vanished
/// user) or the login fails.
pub async fn login_verified_user(
    auth_session: &mut AuthSession,
    user_id: domain::UserId,
) -> Result<(), ReauthError> {
    let user = auth_session
        .backend
        .get_user(&user_id.as_uuid())
        .await
        .map_err(|_| ReauthError)?
        .ok_or(ReauthError)?;
    auth_session.login(&user).await.map_err(|_| ReauthError)?;
    Ok(())
}

/// The result of verifying a password first factor — the single decision every
/// password sign-in entry point reports to its caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordOutcome {
    /// Password verified and the session is now authenticated.
    Authenticated,
    /// Password verified, but the account owes a confirmed second factor; a
    /// pending-MFA marker has been recorded and the session is **not** yet
    /// authenticated. The caller drives the second-factor step.
    MfaRequired,
    /// The email/password did not match (uniform failure — no enumeration).
    InvalidCredentials,
}

/// Verify a password first factor and advance the session accordingly.
///
/// This is the **one** implementation of the password sign-in sequence: re-auth
/// over a live session cycles the id first (fixation defence); the password is
/// verified; an account with a confirmed second factor stops at
/// [`PasswordOutcome::MfaRequired`] after recording the pending marker; otherwise
/// the user is signed in (which cycles the id) and the CSRF token is rotated.
/// Every entry point (the `login` server function, and the integration-test
/// handler) calls this rather than re-assembling the steps.
///
/// # Errors
///
/// [`AuthError::Session`] if any session-store step fails;
/// [`AuthError::Db`]/[`AuthError::Crypto`]/[`AuthError::Task`] from credential
/// lookup and verification.
pub async fn password_first_factor(
    auth_session: &mut AuthSession,
    mfa: &crate::MfaSession,
    db: &Db,
    creds: crate::Credentials,
) -> Result<PasswordOutcome, crate::AuthError> {
    if auth_session.user.is_some() {
        cycle_session_id(auth_session)
            .await
            .map_err(|_| crate::AuthError::Session)?;
    }
    let Some(user) = auth_session
        .authenticate(creds)
        .await
        .map_err(|_| crate::AuthError::Session)?
    else {
        return Ok(PasswordOutcome::InvalidCredentials);
    };

    if crate::second_factor_required(db, user.id.as_uuid()).await? {
        let pending = crate::PendingMfa {
            user_id: user.id.as_uuid(),
            totp: true,
        };
        mfa.set_pending_mfa(&pending)
            .await
            .map_err(|_| crate::AuthError::Session)?;
        return Ok(PasswordOutcome::MfaRequired);
    }

    auth_session
        .login(&user)
        .await
        .map_err(|_| crate::AuthError::Session)?;
    crate::rotate_csrf_token(auth_session)
        .await
        .map_err(|_| crate::AuthError::Session)?;
    Ok(PasswordOutcome::Authenticated)
}

/// Complete a sign-in from the pending-MFA marker after a second factor (a TOTP
/// code, a recovery code, or a passkey assertion) has been verified: consume the
/// marker, sign the user in (cycling the pending id), and rotate the CSRF token.
/// The single implementation of the MFA-completion sequence.
///
/// # Errors
///
/// Returns [`ReauthError`] if the marker cannot be cleared or the sign-in/rotation
/// fails.
pub async fn complete_second_factor(
    auth_session: &mut AuthSession,
    mfa: &crate::MfaSession,
    user_id: domain::UserId,
) -> Result<(), ReauthError> {
    mfa.take_pending_mfa().await.map_err(|_| ReauthError)?;
    login_verified_user(auth_session, user_id).await?;
    crate::rotate_csrf_token(auth_session)
        .await
        .map_err(|_| ReauthError)?;
    Ok(())
}

/// Sign the current session out: rotate the CSRF token across the boundary, then
/// clear the session. Keeps the only `axum-login` `logout` call inside the facade
/// (as with [`login_verified_user`]/[`rebind_after_password_change`]).
///
/// # Errors
///
/// Returns [`ReauthError`] if rotating the token or clearing the session fails.
pub async fn sign_out(auth_session: &mut AuthSession) -> Result<(), ReauthError> {
    crate::rotate_csrf_token(auth_session)
        .await
        .map_err(|_| ReauthError)?;
    auth_session.logout().await.map_err(|_| ReauthError)?;
    Ok(())
}

/// Builds the auth-manager layer: an [`AuthManagerLayer`] wrapping the
/// [`Backend`] and the first-party [`PgSessionStore`].
///
/// The session cookie is `Secure`, `HttpOnly`, `SameSite=Lax` (Strict would
/// break the P7 OIDC redirect-back; CSRF covers the gap), named
/// [`SESSION_COOKIE_NAME`], and expires on `idle_timeout` of inactivity (sliding).
///
/// The store runs session-row CRUD over the same `db` pool as the rest of the
/// application; its schema is owned by the reversible `tower_sessions_session`
/// migration, so there is nothing to migrate here.
#[must_use]
pub fn build_auth_layer(
    db: Db,
    pepper: Option<SecretString>,
    idle_timeout: Duration,
) -> AuthManagerLayer<Backend, PgSessionStore> {
    let session_store = PgSessionStore::new(db.clone());

    // time::Duration cannot represent every std::time::Duration; clamp lossily to
    // whole seconds (sub-second idle precision is irrelevant for an 8h timeout).
    let expiry_secs = i64::try_from(idle_timeout.as_secs()).unwrap_or(i64::MAX);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_name(SESSION_COOKIE_NAME)
        .with_expiry(Expiry::OnInactivity(time::Duration::seconds(expiry_secs)));

    let backend = Backend::new(db, pepper);
    AuthManagerLayerBuilder::new(backend, session_layer).build()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::{HeaderMap, StatusCode, header};
    use axum::response::IntoResponse;
    use axum::routing::{get, post};
    use axum::{Form, Router};
    use http_body_util::BodyExt as _;
    use secrecy::SecretString;
    use sqlx::PgPool;
    use tower::ServiceExt as _;
    use uuid::Uuid;

    use super::{AuthSession, build_auth_layer};
    use crate::{Credentials, CsrfToken, csrf_layer};

    async fn seed_user(pool: &PgPool, password: &str) -> String {
        seed_user_with(pool, password, false).await
    }

    /// A deliberately weak Argon2id hash (params below the active suite), to
    /// exercise rehash-on-login.
    fn weak_hash(password: &str) -> String {
        use argon2::password_hash::rand_core::OsRng;
        use argon2::password_hash::{PasswordHasher as _, SaltString};
        use argon2::{Algorithm, Argon2, Params, Version};

        let weak = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(8 * 1024, 1, 1, None).unwrap(),
        );
        let salt = SaltString::generate(&mut OsRng);
        weak.hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    async fn seed_user_with(pool: &PgPool, password: &str, weak: bool) -> String {
        let tag = Uuid::new_v4().simple().to_string();
        let email = format!("{tag}@example.test");
        let hash = if weak {
            weak_hash(password)
        } else {
            crypto::hash_password(&SecretString::from(password.to_owned()), None)
                .unwrap()
                .into_string()
        };
        let user_id: Uuid = sqlx::query_scalar(
            "INSERT INTO users (email, display_name, webauthn_user_handle) \
             VALUES ($1::citext, 'Test', $2) RETURNING id",
        )
        .bind(&email)
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .fetch_one(pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO password_credentials (user_id, password_hash) VALUES ($1, $2)")
            .bind(user_id)
            .bind(&hash)
            .execute(pool)
            .await
            .unwrap();
        email
    }

    async fn login_handler(
        mut auth_session: AuthSession,
        Form(creds): Form<Credentials>,
    ) -> StatusCode {
        if auth_session.user.is_some() && crate::cycle_session_id(&auth_session).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        match auth_session.authenticate(creds).await {
            Ok(Some(user)) => match auth_session.login(&user).await {
                Ok(()) => match crate::rotate_csrf_token(&auth_session).await {
                    Ok(()) => StatusCode::OK,
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
                },
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Ok(None) => StatusCode::UNAUTHORIZED,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    async fn logout_handler(mut auth_session: AuthSession) -> StatusCode {
        if crate::rotate_csrf_token(&auth_session).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        match auth_session.logout().await {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[derive(serde::Deserialize)]
    struct ChangePasswordForm {
        current_password: SecretString,
        new_password: SecretString,
    }

    /// Mirror of the server binary's `/api/change-password` handler, so the
    /// session-invalidation and policy behaviour is covered by the auth crate's
    /// integration tests (the binary itself is not unit-testable).
    async fn change_password_handler(
        mut auth_session: AuthSession,
        Form(form): Form<ChangePasswordForm>,
    ) -> StatusCode {
        let Some(user) = auth_session.user.clone() else {
            return StatusCode::UNAUTHORIZED;
        };
        if crate::validate_password(&form.new_password).is_err() {
            return StatusCode::BAD_REQUEST;
        }
        match auth_session
            .backend
            .change_password(
                user.id.as_uuid(),
                &form.current_password,
                &form.new_password,
            )
            .await
        {
            Ok(()) => {}
            Err(crate::AuthError::WrongPassword) => return StatusCode::FORBIDDEN,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        }
        if crate::rebind_after_password_change(&mut auth_session, user.id)
            .await
            .is_err()
            || crate::rotate_csrf_token(&auth_session).await.is_err()
        {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        StatusCode::OK
    }

    /// Reports whether the request's session is still authenticated. Used to
    /// confirm a session survives a second request after rehash-on-login.
    async fn whoami(auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => (StatusCode::OK, user.id.as_uuid().to_string()),
            None => (StatusCode::UNAUTHORIZED, String::new()),
        }
    }

    /// Probe handler echoing the current session id and CSRF token, so tests can
    /// observe id-cycling and obtain a valid token.
    async fn probe(session: tower_sessions::Session, req: Request) -> impl IntoResponse {
        let token = req
            .extensions()
            .get::<CsrfToken>()
            .map(|t| t.as_str().to_owned())
            .unwrap_or_default();
        session.insert("probe", true).await.unwrap();
        let id = session.id().map(|i| i.to_string()).unwrap_or_default();
        format!("{id}|{token}")
    }

    /// A read-only probe that returns the CSRF token from extensions but never
    /// writes the session, so a test can observe whether the CSRF *layer* (not a
    /// handler) created a session row.
    async fn readonly_probe(req: Request) -> impl IntoResponse {
        let token = req
            .extensions()
            .get::<CsrfToken>()
            .map(|t| t.as_str().to_owned())
            .unwrap_or_default();
        format!("|{token}")
    }

    fn build_router(pool: PgPool) -> Router {
        let auth_layer = build_auth_layer(pool, None, Duration::from_hours(8));
        Router::new()
            .route("/probe", get(probe))
            .route("/readonly", get(readonly_probe))
            .route("/whoami", get(whoami))
            .route("/api/login", post(login_handler))
            .route("/api/logout", post(logout_handler))
            .route("/api/change-password", post(change_password_handler))
            .layer(axum::middleware::from_fn(csrf_layer))
            .layer(auth_layer)
    }

    /// Extract the `owk.sid` Set-Cookie value (name=value, no attributes).
    fn session_cookie(headers: &HeaderMap) -> Option<String> {
        headers
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|c| c.starts_with("owk.sid="))
            .and_then(|c| c.split(';').next())
            .map(str::to_owned)
    }

    /// GET /probe as an HTML navigation; returns (cookie, session id, CSRF token).
    /// The `Accept: text/html` header is what makes the lazy-minting layer mint a
    /// token and set the session cookie (asset/XHR GETs do not).
    async fn prime(router: &Router) -> (String, String, String) {
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let cookie = session_cookie(resp.headers()).expect("session cookie set");
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let (id, token) = text.split_once('|').unwrap();
        (cookie, id.to_owned(), token.to_owned())
    }

    fn login_body(email: &str, token: &str) -> String {
        login_body_pw(email, "correct horse", token)
    }

    fn change_pw_body(current: &str, new: &str, token: &str) -> String {
        form_urlencoded::Serializer::new(String::new())
            .append_pair("current_password", current)
            .append_pair("new_password", new)
            .append_pair("csrf_token", token)
            .finish()
    }

    /// Logs in on a fresh anonymous session and returns the post-login cookie.
    async fn login(router: &Router, email: &str) -> String {
        let (cookie, _id, token) = prime(router).await;
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(login_body(email, &token)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "login should succeed");
        session_cookie(resp.headers()).expect("login re-issues owk.sid")
    }

    /// Posts a form to a path with the given cookie and CSRF header token.
    async fn post_form(router: &Router, path: &str, cookie: &str, body: String) -> StatusCode {
        router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header(header::COOKIE, cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    /// GET /whoami with a cookie; returns the status.
    async fn whoami_status(router: &Router, cookie: &str) -> StatusCode {
        router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/whoami")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    fn login_body_pw(email: &str, password: &str, token: &str) -> String {
        form_urlencoded::Serializer::new(String::new())
            .append_pair("email", email)
            .append_pair("password", password)
            .append_pair("csrf_token", token)
            .finish()
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn login_roundtrip_sets_cookie_and_hydrates(pool: PgPool) {
        let email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool);

        let (cookie, _id, token) = prime(&router).await;
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(login_body(&email, &token)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "login should succeed");
        assert!(
            session_cookie(resp.headers()).is_some(),
            "login should set owk.sid"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn login_cycles_session_id(pool: PgPool) {
        let email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool);

        let (cookie_before, _id, token) = prime(&router).await;
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::COOKIE, &cookie_before)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(login_body(&email, &token)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // login() calls cycle_id(), so owk.sid is re-issued with a fresh id.
        let cookie_after = session_cookie(resp.headers()).expect("login re-issues owk.sid");
        assert_ne!(
            cookie_before, cookie_after,
            "login must cycle the session id (cookie value changes)"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn logout_removes_session_record(pool: PgPool) {
        let email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool.clone());

        let (cookie, _id, token) = prime(&router).await;
        let login_resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(login_body(&email, &token)))
                    .unwrap(),
            )
            .await
            .unwrap();
        let cookie = session_cookie(login_resp.headers()).expect("login re-issues owk.sid");

        let count_before: i64 = sqlx::query_scalar("SELECT count(*) FROM tower_sessions.session")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(count_before >= 1, "a session row must exist after login");

        // A CSRF token valid for the post-login session. Sent as an HTML
        // navigation so the lazy-minting layer mints a fresh post-login token.
        let token = {
            let resp = router
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/probe")
                        .header(header::ACCEPT, "text/html")
                        .header(header::COOKIE, &cookie)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            String::from_utf8(body.to_vec())
                .unwrap()
                .split_once('|')
                .unwrap()
                .1
                .to_owned()
        };

        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/logout")
                    .header(header::COOKIE, &cookie)
                    .header("x-csrf-token", &token)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM tower_sessions.session")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(remaining, 0, "logout must remove the session record");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn csrf_rejects_unsafe_post_without_token(pool: PgPool) {
        let router = build_router(pool);

        let (cookie, _id, _token) = prime(&router).await;
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/logout")
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "unsafe POST without a CSRF token must be 403"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn csrf_accepts_header_token(pool: PgPool) {
        let _email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool);

        let (cookie, _id, token) = prime(&router).await;
        // Header token, empty body — passes CSRF and reaches the logout handler.
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/logout")
                    .header(header::COOKIE, &cookie)
                    .header("x-csrf-token", &token)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "valid header token must pass CSRF"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn csrf_accepts_form_field_token(pool: PgPool) {
        let email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool);

        let (cookie, _id, token) = prime(&router).await;
        // No header; token only in the form field (no-JS). Must pass CSRF and the
        // body must survive re-injection so login parses email/password → 200.
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(login_body(&email, &token)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "form-field token must pass CSRF and the body must survive re-injection"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn html_get_mints_token_and_sets_cookie(pool: PgPool) {
        let router = build_router(pool);
        // prime() sends Accept: text/html → an HTML navigation, which mints.
        let (cookie, _id, token) = prime(&router).await;
        assert!(cookie.starts_with("owk.sid="), "HTML GET must set owk.sid");
        assert!(!token.is_empty(), "HTML GET must mint a CSRF token");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn non_html_get_does_not_write_session(pool: PgPool) {
        let router = build_router(pool.clone());
        // A GET WITHOUT Accept: text/html (asset/XHR) must do a read-only lookup:
        // no Set-Cookie, no token in the body, and no session row persisted.
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/readonly")
                    .header(header::ACCEPT, "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(
            session_cookie(resp.headers()).is_none(),
            "a non-HTML GET must not emit a Set-Cookie"
        );
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let token = text.split_once('|').unwrap().1;
        assert!(token.is_empty(), "a non-HTML GET must not mint a token");

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM tower_sessions.session")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "a non-HTML GET must not persist a session row");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn unsafe_post_without_stored_token_is_403_and_writes_no_session(pool: PgPool) {
        let router = build_router(pool.clone());
        // Straight POST, no prior priming → no stored token. Must be 403, and the
        // unsafe path must never mint, so no session row is written.
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/logout")
                    .header("x-csrf-token", "anything")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "an unsafe POST with no stored token must be 403"
        );
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM tower_sessions.session")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(
            count, 0,
            "the unsafe path must never mint a token (no session row written)"
        );
    }

    /// Fetch the (cookie, csrf token) pair from /probe for a given cookie. Sent as
    /// an HTML navigation so the lazy-minting layer mints a token if absent.
    async fn probe_with(router: &Router, cookie: &str) -> (String, String) {
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .header(header::ACCEPT, "text/html")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let set = session_cookie(resp.headers()).unwrap_or_else(|| cookie.to_owned());
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(body.to_vec()).unwrap();
        let token = text.split_once('|').unwrap().1.to_owned();
        (set, token)
    }

    /// M4: the CSRF token must rotate across the login boundary, and the pre-auth
    /// token must no longer be accepted on a later unsafe POST.
    #[sqlx::test(migrations = "../db/migrations")]
    async fn csrf_token_rotates_across_login(pool: PgPool) {
        let email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool);

        // Anonymous token A.
        let (cookie, _id, token_a) = prime(&router).await;

        // Log in with A; login must rotate the token.
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(login_body(&email, &token_a)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let cookie = session_cookie(resp.headers()).expect("login re-issues owk.sid");

        // Probe post-login token B; it must differ from A.
        let (cookie, token_b) = probe_with(&router, &cookie).await;
        assert_ne!(
            token_a, token_b,
            "CSRF token must be rotated across the login boundary"
        );

        // An unsafe POST presenting the OLD token A must be rejected.
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/logout")
                    .header(header::COOKIE, &cookie)
                    .header("x-csrf-token", &token_a)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "the pre-auth CSRF token must not survive into the authenticated session"
        );
    }

    /// S1: re-login over a live authenticated cookie must yield a new owk.sid and
    /// the old session id must no longer load.
    #[sqlx::test(migrations = "../db/migrations")]
    async fn relogin_rotates_session_id(pool: PgPool) {
        let email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool);

        let (cookie, _id, token) = prime(&router).await;
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(login_body(&email, &token)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let cookie_first = session_cookie(resp.headers()).expect("login re-issues owk.sid");

        // Re-login over the authenticated cookie.
        let (cookie_first, token2) = probe_with(&router, &cookie_first).await;
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::COOKIE, &cookie_first)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(login_body(&email, &token2)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let cookie_second = session_cookie(resp.headers()).expect("re-login re-issues owk.sid");
        assert_ne!(
            cookie_first, cookie_second,
            "re-login must rotate the session id"
        );

        // The first authenticated session id must no longer load.
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/whoami")
                    .header(header::COOKIE, &cookie_first)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "the flushed pre-relogin session id must no longer authenticate"
        );
    }

    /// M1: a weak-param-hash user logs in (triggering rehash-on-login); a SECOND
    /// authenticated request on the same cookie must still load the session — the
    /// session must not be invalidated by the rehash.
    #[sqlx::test(migrations = "../db/migrations")]
    async fn rehash_on_login_keeps_session_alive(pool: PgPool) {
        let email = seed_user_with(&pool, "weak pw", true).await;
        let router = build_router(pool);

        let (cookie, _id, token) = prime(&router).await;
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(login_body_pw(&email, "weak pw", &token)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "weak-hash user logs in");
        let cookie = session_cookie(resp.headers()).expect("login re-issues owk.sid");

        // Second request on the same cookie: the session must still authenticate.
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/whoami")
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "rehash-on-login must not log the user out on the next request"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn change_password_succeeds_and_invalidates_other_sessions(pool: PgPool) {
        let email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool.clone());

        // Two independent authenticated sessions for the same user.
        let cookie_a = login(&router, &email).await;
        let cookie_b = login(&router, &email).await;

        // Both authenticate before the change.
        assert_eq!(whoami_status(&router, &cookie_a).await, StatusCode::OK);
        assert_eq!(whoami_status(&router, &cookie_b).await, StatusCode::OK);

        // Change the password on session A (CSRF token bound to A).
        let (cookie_a, token_a) = probe_with(&router, &cookie_a).await;
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/change-password")
                    .header(header::COOKIE, &cookie_a)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(change_pw_body(
                        "correct horse",
                        "a brand new long password",
                        &token_a,
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "correct current password → 200"
        );

        // The handler cycles A's id, so the response re-issues owk.sid with a fresh
        // value. A keeps working under the new cookie because its session was
        // re-bound; the old cookie value no longer loads.
        let cookie_a_after = session_cookie(resp.headers())
            .expect("change-password must re-issue owk.sid for the active session");
        assert_ne!(
            cookie_a, cookie_a_after,
            "the active session id must be cycled across the password change"
        );
        assert_eq!(
            whoami_status(&router, &cookie_a_after).await,
            StatusCode::OK,
            "session A must still authenticate under its cycled cookie"
        );

        // B's session_auth_hash no longer matches the new stored hash → its next
        // request must fail to authenticate.
        assert_eq!(
            whoami_status(&router, &cookie_b).await,
            StatusCode::UNAUTHORIZED,
            "other sessions must be invalidated by the password change"
        );

        // The stored hash actually changed and must_change is cleared.
        let (hash, must_change): (String, bool) = sqlx::query_as(
            "SELECT pc.password_hash, pc.must_change FROM password_credentials pc \
             JOIN users u ON u.id = pc.user_id WHERE u.email = $1::citext",
        )
        .bind(&email)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!must_change, "must_change must be cleared");
        assert!(!hash.is_empty());
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn change_password_wrong_current_is_403(pool: PgPool) {
        let email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool);

        let cookie = login(&router, &email).await;
        let (cookie, token) = probe_with(&router, &cookie).await;
        let status = post_form(
            &router,
            "/api/change-password",
            &cookie,
            change_pw_body("wrong current pw", "a brand new long password", &token),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "wrong current password → 403"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn change_password_short_new_is_400(pool: PgPool) {
        let email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool);

        let cookie = login(&router, &email).await;
        let (cookie, token) = probe_with(&router, &cookie).await;
        let status = post_form(
            &router,
            "/api/change-password",
            &cookie,
            change_pw_body("correct horse", "tooshort", &token),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "a new password failing the length policy → 400"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn change_password_without_csrf_is_403(pool: PgPool) {
        let email = seed_user(&pool, "correct horse").await;
        let router = build_router(pool);

        let cookie = login(&router, &email).await;
        // No csrf_token field and no header → CSRF layer rejects before the handler.
        let body = form_urlencoded::Serializer::new(String::new())
            .append_pair("current_password", "correct horse")
            .append_pair("new_password", "a brand new long password")
            .finish();
        let status = post_form(&router, "/api/change-password", &cookie, body).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "change-password without a CSRF token must be 403"
        );
    }

    // --- P6: the two-step MFA login -----------------------------------------
    //
    // Mirrors the server binary's `/api/login` + `/api/mfa/*` handlers so the
    // session state machine — password verified → pending second factor (not yet
    // signed in) → factor cleared → signed in — is covered as an integration test
    // (the binary is not unit-testable). The factor itself is stubbed; its own
    // verification lives in the `totp`/`recovery` unit tests.

    use axum::extract::State;

    use crate::{MfaSession, login_verified_user};

    /// Seed a user that already has a confirmed TOTP enrolment, so the login flow
    /// demands a second factor.
    async fn seed_user_with_totp(pool: &PgPool, password: &str) -> String {
        let email = seed_user(pool, password).await;
        let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1::citext")
            .bind(&email)
            .fetch_one(pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO totp_credentials (user_id, secret_encrypted, confirmed_at) \
             VALUES ($1, $2, now())",
        )
        .bind(user_id)
        .bind(vec![0u8; 16])
        .execute(pool)
        .await
        .unwrap();
        email
    }

    async fn mfa_login_handler(
        mut auth_session: AuthSession,
        mfa: MfaSession,
        State(pool): State<PgPool>,
        Form(creds): Form<Credentials>,
    ) -> StatusCode {
        // Exercises the production sequence directly via the facade.
        match crate::password_first_factor(&mut auth_session, &mfa, &pool, creds).await {
            Ok(crate::PasswordOutcome::MfaRequired) => StatusCode::ACCEPTED, // 202
            Ok(crate::PasswordOutcome::Authenticated) => StatusCode::OK,
            Ok(crate::PasswordOutcome::InvalidCredentials) => StatusCode::UNAUTHORIZED,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Completes the login from the pending marker (the factor is taken as passed).
    async fn mfa_complete_handler(mut auth_session: AuthSession, mfa: MfaSession) -> StatusCode {
        let Ok(Some(pending)) = mfa.peek_pending_mfa().await else {
            return StatusCode::BAD_REQUEST;
        };
        if mfa.take_pending_mfa().await.is_err()
            || login_verified_user(&mut auth_session, domain::UserId::new(pending.user_id))
                .await
                .is_err()
            || crate::rotate_csrf_token(&auth_session).await.is_err()
        {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        StatusCode::OK
    }

    fn build_mfa_router(pool: PgPool) -> Router {
        let auth_layer = build_auth_layer(pool.clone(), None, Duration::from_hours(8));
        Router::new()
            .route("/probe", get(probe))
            .route("/whoami", get(whoami))
            .route("/api/login", post(mfa_login_handler))
            .route("/api/mfa/complete", post(mfa_complete_handler))
            .layer(axum::middleware::from_fn(csrf_layer))
            .layer(auth_layer)
            .with_state(pool)
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn password_login_requires_second_factor_then_completes(pool: PgPool) {
        let email = seed_user_with_totp(&pool, "correct horse").await;
        let router = build_mfa_router(pool);

        // First factor: correct password, but a confirmed TOTP exists → 202, and
        // the session must NOT be authenticated yet.
        let (cookie, _id, token) = prime(&router).await;
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(login_body(&email, &token)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::ACCEPTED,
            "a password alone must not sign in an MFA-enrolled user"
        );
        let cookie = session_cookie(resp.headers()).unwrap_or(cookie);
        assert_eq!(
            whoami_status(&router, &cookie).await,
            StatusCode::UNAUTHORIZED,
            "the pending-MFA session must not authenticate"
        );

        // Second factor clears → the login completes and the session authenticates.
        let (cookie, token) = probe_with(&router, &cookie).await;
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/mfa/complete")
                    .header(header::COOKIE, &cookie)
                    .header("x-csrf-token", &token)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "second factor completes login"
        );
        let cookie = session_cookie(resp.headers()).unwrap_or(cookie);
        assert_eq!(
            whoami_status(&router, &cookie).await,
            StatusCode::OK,
            "the session must authenticate after the second factor"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn mfa_complete_without_pending_is_rejected(pool: PgPool) {
        let router = build_mfa_router(pool);
        let (cookie, _id, token) = prime(&router).await;
        // No prior password step → no pending marker → 400.
        let status = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/mfa/complete")
                    .header(header::COOKIE, &cookie)
                    .header("x-csrf-token", &token)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .status();
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "completing MFA with no pending marker must be rejected"
        );
    }
}
