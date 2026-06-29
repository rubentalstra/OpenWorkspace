//! HTTP endpoints for OIDC SSO: start the Authorization Code + PKCE flow, handle
//! the callback (validate, provision, sign in), list providers, and drive
//! RP-initiated logout.
//!
//! Thin glue over the `auth` facade — all protocol, validation, provisioning and
//! linking logic (and its tests) live in `crates/auth`. The MFA decision for P7 is
//! "trust the IdP": a validated OIDC sign-in completes the session directly via
//! `login_verified_user`, with no local second-factor gate.

use std::sync::Arc;

use auth::{AuthSession, LogoutHint, OidcCallback, OidcHttpClient, OidcSession, ProviderRegistry};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use db::Db;
use serde::Deserialize;

/// OIDC services held in `AppState`: the provider registry (discovery + cache),
/// the shared outbound HTTP client, and the app's public base URL.
#[derive(Clone)]
pub(crate) struct OidcServices {
    pub(crate) registry: ProviderRegistry,
    pub(crate) http: OidcHttpClient,
    pub(crate) base_url: Arc<str>,
}

/// Optional post-login redirect target, validated to a local path.
#[derive(Deserialize)]
pub(crate) struct StartParams {
    return_to: Option<String>,
}

/// Map a facade error to a client status, keeping the body generic. Auth-style
/// failures are 401, policy refusals 403, everything else 500.
fn oidc_status(err: &auth::OidcError) -> StatusCode {
    use auth::OidcError as E;
    match err {
        E::StateMismatch
        | E::ResponseIssuerMismatch
        | E::IdToken
        | E::AccessTokenHash
        | E::TokenExchange => StatusCode::UNAUTHORIZED,
        E::EmailUnverified | E::DomainNotAllowed | E::ProvisioningDisabled | E::Provisioning => {
            StatusCode::FORBIDDEN
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Whether a post-login redirect target is a safe local path (no open redirect).
fn is_safe_return_to(path: &str) -> bool {
    path.starts_with('/') && !path.starts_with("//")
}

/// The exact redirect URI registered with the provider for this slug.
fn redirect_uri(base_url: &str, slug: &str) -> String {
    format!("{base_url}/auth/{slug}/callback")
}

/// `GET /auth/{slug}/start` — begin the flow and 302 to the IdP.
pub(crate) async fn start(
    Path(slug): Path<String>,
    oidc_session: OidcSession,
    State(svc): State<OidcServices>,
    Query(params): Query<StartParams>,
) -> Response {
    let Ok(provider) = svc.registry.discovered(&slug).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let return_to = params
        .return_to
        .filter(|r| is_safe_return_to(r))
        .unwrap_or_else(|| "/".to_owned());
    let Ok(auth_request) =
        auth::begin_login(&provider, &redirect_uri(&svc.base_url, &slug), return_to)
    else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if oidc_session
        .set_transaction(&auth_request.transaction)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Redirect::to(&auth_request.authorize_url).into_response()
}

/// `GET /auth/{slug}/callback` — validate the response and ID token, provision or
/// link the user, then sign them in and redirect to the stored `return_to`.
pub(crate) async fn callback(
    Path(slug): Path<String>,
    mut auth_session: AuthSession,
    oidc_session: OidcSession,
    State(svc): State<OidcServices>,
    State(db): State<Db>,
    Query(params): Query<OidcCallback>,
) -> Response {
    let Ok(Some(transaction)) = oidc_session.take_transaction().await else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Ok(provider) = svc.registry.discovered(&slug).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let return_to = transaction.return_to.clone();

    let identity = match auth::complete_login(
        &provider,
        &svc.http,
        &redirect_uri(&svc.base_url, &slug),
        params,
        transaction,
    )
    .await
    {
        Ok(identity) => identity,
        Err(err) => return oidc_status(&err).into_response(),
    };

    let id_token = identity.id_token_compact.clone();
    let user_id = match auth::provision_user(&db, &provider, &identity).await {
        Ok(user_id) => user_id,
        Err(err) => return oidc_status(&err).into_response(),
    };

    // Re-auth over a live session: cycle the id first so it always rotates.
    if auth_session.user.is_some() && auth::cycle_session_id(&auth_session).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    // Trust the IdP: complete the session directly (no local second factor).
    if auth::login_verified_user(&mut auth_session, user_id)
        .await
        .is_err()
        || auth::rotate_csrf_token(&auth_session).await.is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    // Retain the compact ID token (only) for RP-initiated logout's id_token_hint.
    let hint = LogoutHint {
        provider_slug: slug,
        id_token,
    };
    if oidc_session.set_logout_hint(&hint).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to(&return_to).into_response()
}

/// Build the OIDC route subtree, wired under the auth + CSRF layers. Only the
/// browser-redirect endpoints live here; the provider list and logout are
/// `#[server]` functions in `app::auth` (logout returns the IdP logout URL it
/// builds from this same `base_url`).
pub(crate) fn routes() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .route("/auth/{slug}/start", get(start))
        .route("/auth/{slug}/callback", get(callback))
}
