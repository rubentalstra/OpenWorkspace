use std::sync::Arc;

use anyhow::Context;
use app::{App, CsrfToken, shell};
use auth::{AuthSession, Credentials, MfaSession, TotpService, WebauthnService};
use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use db::Db;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use secrecy::SecretString;
use tower_http::trace::TraceLayer;

mod mfa;
mod oidc;

/// Shared web state. `FromRef` lets Leptos pull `LeptosOptions` and the handlers
/// pull the `Db`, the WebAuthn engine and the TOTP service from the one state.
#[derive(Clone)]
struct AppState {
    leptos_options: LeptosOptions,
    db: Db,
    webauthn: WebauthnService,
    totp: TotpService,
    oidc: oidc::OidcServices,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

impl FromRef<AppState> for oidc::OidcServices {
    fn from_ref(state: &AppState) -> Self {
        state.oidc.clone()
    }
}

impl FromRef<AppState> for Db {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for WebauthnService {
    fn from_ref(state: &AppState) -> Self {
        state.webauthn.clone()
    }
}

impl FromRef<AppState> for TotpService {
    fn from_ref(state: &AppState) -> Self {
        state.totp.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::load()?;
    let _telemetry = observability::init(&observability::Options {
        service_name: cfg.observability.service_name.clone(),
        log_filter: cfg.observability.log_filter.clone(),
        otlp_endpoint: cfg.observability.otlp_endpoint.clone(),
        metrics_enabled: cfg.observability.metrics_enabled,
    })?;

    let pool = db::connect(&cfg.database.url, cfg.database.max_connections).await?;
    db::run_migrations(&pool).await?;

    auth::bootstrap_admin(&pool, &cfg.auth)
        .await
        .context("bootstrapping instance admin")?;

    // Reap expired session rows so the table cannot grow unbounded. Expired rows
    // never authenticate (the store filters them on load); this is housekeeping.
    // Held for the process lifetime.
    let _reaper = auth::spawn_session_reaper(pool.clone(), std::time::Duration::from_hours(6));

    let auth_layer = auth::build_auth_layer(
        pool.clone(),
        cfg.auth.argon2_pepper.clone(),
        cfg.auth.session_idle_timeout,
    );

    // Field-encryption keyring (unwraps the TOTP-secret data key under the root
    // KEK), the TOTP service that seals secrets with it, and the WebAuthn engine.
    let keyring = auth::FieldKeyring::load(&pool, &cfg.auth.field_encryption_key)
        .await
        .context("loading the field-encryption keyring")?;
    let totp = TotpService::new(
        cfg.auth.webauthn_rp_name.clone(),
        keyring.totp_dek().clone(),
    );
    let webauthn = WebauthnService::new(
        &cfg.auth.webauthn_rp_id,
        &cfg.auth.webauthn_rp_origin,
        &cfg.auth.webauthn_rp_name,
    )
    .context("building the webauthn relying party")?;

    // OIDC SSO services: one outbound HTTP client (aws-lc-rs rustls, redirects off)
    // and the provider registry that discovers + caches each IdP. The keyring is
    // moved in last; it also seals/opens OIDC client secrets.
    let oidc_http = auth::OidcHttpClient::new(cfg.auth.oidc_http_timeout)
        .context("building the OIDC HTTP client")?;
    let oidc = oidc::OidcServices {
        registry: auth::ProviderRegistry::new(pool.clone(), oidc_http.clone(), keyring),
        http: oidc_http,
        base_url: Arc::from(cfg.auth.public_base_url.trim_end_matches('/')),
    };

    let conf = get_configuration(None).context("loading Leptos configuration")?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let state = AppState {
        leptos_options: leptos_options.clone(),
        db: pool,
        webauthn,
        totp,
        oidc,
    };

    // Operational probes live outside the auth/session/CSRF stack.
    let ops = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(ready))
        .route(
            "/metrics",
            get(|| async { observability::render_metrics() }),
        )
        .with_state(state.clone());

    // Authenticated surface: Leptos routes plus the login/logout/change-password
    // endpoints. The CSRF middleware runs inside the session/auth layer (it needs
    // the session) and is outermost over the handlers; the auth layer is outermost
    // overall.
    //
    // FAIL-CLOSED CSRF: `auth::csrf_layer` rejects every unsafe method lacking a
    // valid token, INCLUDING the Leptos server-fn endpoints under `/api/*`. A
    // mutating `#[server]` fn must attach the token via the first-party
    // `app::CsrfClient` (`#[server(client = CsrfClient)]`), which reads
    // `<meta name="csrf-token">` and sets `X-CSRF-Token`; read-only queries use a
    // real GET (`#[server(input = GetUrl)]`) and are CSRF-exempt by method. P5
    // ships no server fns yet, so there is nothing to annotate. Do NOT exempt
    // `/api/*` from CSRF to work around a 403 — see csrf.rs.
    let app = Router::new()
        .route("/api/login", post(login))
        .route("/api/logout", post(oidc::logout))
        .route("/api/change-password", post(change_password))
        .merge(mfa::routes())
        .merge(oidc::routes())
        .leptos_routes_with_context(&state, routes, provide_csrf_context, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(axum::middleware::from_fn(auth::csrf_layer))
        .layer(auth_layer)
        .with_state(state);

    let router = ops.merge(app).layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    log!("listening on http://{addr}");
    axum::serve(listener, router.into_make_service())
        .await
        .context("serving HTTP")?;
    Ok(())
}

/// Per-request Leptos context: surface the CSRF token (set in request extensions
/// by the CSRF middleware) to the `App` so it can render the `<meta>` tag.
fn provide_csrf_context() {
    if let Some(parts) = use_context::<axum::http::request::Parts>()
        && let Some(token) = parts.extensions.get::<auth::CsrfToken>()
    {
        provide_context(CsrfToken(token.as_str().to_owned()));
    }
}

/// Login endpoint (first factor). Verifies the password; if the account has a
/// confirmed second factor it stops short of signing in, records a pending-MFA
/// marker in the session, and answers `{ "mfa_required": true }` so the client
/// continues to `/api/mfa/totp/verify` or `/api/mfa/recovery/verify`. Otherwise
/// it signs the user in (cycling the session id, defeating fixation) and rotates
/// the CSRF token across the auth boundary.
///
/// If the session is already authenticated (re-auth/step-up), the existing
/// session is flushed first so the id always rotates afresh.
async fn login(
    mut auth_session: AuthSession,
    mfa_session: MfaSession,
    State(db): State<Db>,
    Form(creds): Form<Credentials>,
) -> Response {
    if auth_session.user.is_some() && auth::cycle_session_id(&auth_session).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match auth::second_factor_required(&db, user.id.as_uuid()).await {
        Ok(true) => {
            if mfa_session
                .set_pending_mfa(&mfa::pending_for(user.id.as_uuid()))
                .await
                .is_err()
            {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            Json(mfa::LoginResponse { mfa_required: true }).into_response()
        }
        Ok(false) => match auth_session.login(&user).await {
            Ok(()) => match auth::rotate_csrf_token(&auth_session).await {
                Ok(()) => Json(mfa::LoginResponse {
                    mfa_required: false,
                })
                .into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            },
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Change-password form. `Debug` is derived only via the redacting `SecretString`,
/// so neither password reaches logs.
#[derive(serde::Deserialize)]
struct ChangePassword {
    current_password: SecretString,
    new_password: SecretString,
}

/// Change-password endpoint.
///
/// Requires an authenticated session (else `401`). Validates the new password
/// against the policy (length over composition; see [`auth::validate_password`]),
/// then verifies the current password and persists the new hash. On success it
/// re-binds **this** session via [`auth::rebind_after_password_change`] (reload +
/// cycle id + re-login, re-stamping the stored auth hash) and rotates the CSRF
/// token, so the active session stays valid under the new hash while
/// `session_auth_hash` invalidates every *other* live session on its next
/// request.
///
/// `must_change` is surfaced (cleared by the change) but **not** enforced with a
/// blanket forced-redirect here: route-gating that forces a flagged user to this
/// endpoint before anything else is P8's job. Mirrors the deferral note in
/// `bootstrap.rs` / `auth::User`.
///
/// Status mapping: wrong current password → `403` (never revealing which field);
/// policy failure → `400`; missing auth → `401`; infrastructure error → `500`.
async fn change_password(
    mut auth_session: AuthSession,
    Form(form): Form<ChangePassword>,
) -> StatusCode {
    let Some(user) = auth_session.user.clone() else {
        return StatusCode::UNAUTHORIZED;
    };
    if auth::validate_password(&form.new_password).is_err() {
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
        Err(auth::AuthError::WrongPassword) => return StatusCode::FORBIDDEN,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    }
    // Re-bind THIS session to the new hash (cycle id + re-login) so it stays valid
    // while session_auth_hash invalidates every OTHER session; then rotate the
    // per-session CSRF token across the change.
    if auth::rebind_after_password_change(&mut auth_session, user.id)
        .await
        .is_err()
        || auth::rotate_csrf_token(&auth_session).await.is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

/// Readiness probe: 200 when the database is reachable, 503 otherwise.
async fn ready(State(pool): State<Db>) -> StatusCode {
    match db::ping(&pool).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
