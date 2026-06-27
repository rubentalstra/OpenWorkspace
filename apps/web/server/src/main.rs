use anyhow::Context;
use app::{App, CsrfToken, shell};
use auth::{AuthSession, Credentials};
use axum::Form;
use axum::Router;
use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use db::Db;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::trace::TraceLayer;

/// Shared web state. `FromRef` lets Leptos pull `LeptosOptions` and handlers pull
/// the `Db` from the one combined state.
#[derive(Clone)]
struct AppState {
    leptos_options: LeptosOptions,
    db: Db,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

impl FromRef<AppState> for Db {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
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

    let auth_layer = auth::build_auth_layer(
        pool.clone(),
        cfg.auth.argon2_pepper.clone(),
        cfg.auth.session_idle_timeout,
    );

    let conf = get_configuration(None).context("loading Leptos configuration")?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let state = AppState {
        leptos_options: leptos_options.clone(),
        db: pool,
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

    // Authenticated surface: Leptos routes plus the login/logout endpoints. The
    // CSRF middleware runs inside the session/auth layer (it needs the session)
    // and is outermost over the handlers; the auth layer is outermost overall.
    //
    // FAIL-CLOSED CSRF: `auth::csrf_layer` rejects every unsafe method lacking a
    // valid token, INCLUDING the Leptos server-fn endpoints under `/api/*`. The
    // Leptos client attaches no `X-CSRF-Token` yet (no cleanly-documented hook in
    // 0.8.20 to inject a header from the `<meta name="csrf-token">`), so the first
    // mutating `#[server]` fn MUST add the token before it can succeed. P5 ships
    // none. Do NOT exempt `/api/*` from CSRF to work around this — see csrf.rs.
    let app = Router::new()
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
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

/// Login endpoint: authenticates the submitted credentials and, on success,
/// logs the user in (which cycles the session id, defeating fixation) and rotates
/// the CSRF token across the auth boundary so no pre-auth token survives.
///
/// If the session is already authenticated (re-auth/step-up), the existing
/// session is flushed first so the id always rotates afresh.
async fn login(mut auth_session: AuthSession, Form(creds): Form<Credentials>) -> StatusCode {
    if auth_session.user.is_some() && auth::cycle_session_id(&auth_session).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    match auth_session.authenticate(creds).await {
        Ok(Some(user)) => match auth_session.login(&user).await {
            Ok(()) => match auth::rotate_csrf_token(&auth_session).await {
                Ok(()) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        },
        Ok(None) => StatusCode::UNAUTHORIZED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Logout endpoint: rotates the CSRF token then flushes the current session.
async fn logout(mut auth_session: AuthSession) -> StatusCode {
    if auth::rotate_csrf_token(&auth_session).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    match auth_session.logout().await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Readiness probe: 200 when the database is reachable, 503 otherwise.
async fn ready(State(pool): State<Db>) -> StatusCode {
    match db::ping(&pool).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
