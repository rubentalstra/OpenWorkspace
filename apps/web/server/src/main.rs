use std::sync::Arc;

use anyhow::Context;
use app::{App, shell};
use auth::{AuthzBackend, TotpService, WebauthnService};
use axum::Router;
use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::routing::get;
use db::Db;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::trace::TraceLayer;

mod context;
mod oidc;
mod upload;

/// Shared web state. `FromRef` lets Leptos pull `LeptosOptions` and the handlers
/// pull the `Db`, the WebAuthn engine and the TOTP service from the one state.
#[derive(Clone)]
struct AppState {
    leptos_options: LeptosOptions,
    db: Db,
    authz: AuthzBackend,
    webauthn: WebauthnService,
    totp: TotpService,
    oidc: oidc::OidcServices,
    storage: storage::Storage,
    upload_limits: storage::ImageLimits,
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

impl FromRef<AppState> for AuthzBackend {
    fn from_ref(state: &AppState) -> Self {
        state.authz.clone()
    }
}

impl FromRef<AppState> for storage::Storage {
    fn from_ref(state: &AppState) -> Self {
        state.storage.clone()
    }
}

impl FromRef<AppState> for storage::ImageLimits {
    fn from_ref(state: &AppState) -> Self {
        state.upload_limits
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

    // Privileged setup runs once under the owner/migrator role (DDL + RLS-bypassing
    // reads/writes): migrations, the system-role seed, the bootstrap admin, and the
    // field-encryption keyring. The pool is then dropped — the process serves under
    // the least-privilege runtime role only.
    let owner_pool = db::connect(&cfg.database.migrator_url, cfg.database.max_connections).await?;
    db::run_migrations(&owner_pool).await?;
    db::seed_system_roles(&owner_pool)
        .await
        .context("seeding system roles")?;
    auth::bootstrap_admin(&owner_pool, &cfg.auth)
        .await
        .context("bootstrapping instance admin")?;
    let keyring = auth::FieldKeyring::load(&owner_pool, &cfg.auth.field_encryption_key)
        .await
        .context("loading the field-encryption keyring")?;
    // Dev-only: seed the local Keycloak SSO provider (sealing its secret with the
    // keyring) so `/login` offers a working SSO button. No-op in production.
    auth::seed_dev_oidc_provider(&owner_pool, &keyring, &cfg.auth)
        .await
        .context("seeding the dev OIDC provider")?;
    owner_pool.close().await;

    // The runtime pool: every request-time query runs as `openworkspace_app`.
    let pool = db::connect(&cfg.database.url, cfg.database.max_connections).await?;

    // Reap expired session rows so the table cannot grow unbounded. Expired rows
    // never authenticate (the store filters them on load); this is housekeeping.
    // Held for the process lifetime.
    let _reaper = auth::spawn_session_reaper(pool.clone(), std::time::Duration::from_hours(6));

    let auth_layer = auth::build_auth_layer(
        pool.clone(),
        cfg.auth.argon2_pepper.clone(),
        cfg.auth.session_idle_timeout,
    );

    // The TOTP service that seals secrets with the keyring's data key, and the
    // WebAuthn engine.
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

    // Object storage for binary assets (presigned URLs via the S3 backend).
    let object_storage =
        storage::Storage::from_config(&cfg.storage).context("building object storage")?;
    let upload_limits = storage::ImageLimits {
        max_bytes: cfg.storage.max_upload_bytes,
        thumbnail_max_px: cfg.storage.thumbnail_max_px,
    };

    let conf = get_configuration(None).context("loading Leptos configuration")?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let state = AppState {
        leptos_options: leptos_options.clone(),
        authz: AuthzBackend::new(pool.clone()),
        db: pool,
        webauthn,
        totp,
        oidc,
        storage: object_storage,
        upload_limits,
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

    // Authenticated surface. The web layer is Leptos: every UI-facing operation
    // (login, logout, change-password, MFA, …) is a `#[server]` fn in the `app`
    // crate, reached under `/api/*`. Raw Axum routes survive only for the protocol
    // needs that a server fn cannot express: the OIDC redirect flow and asset
    // upload/serve. See `docs/web-architecture.md`.
    //
    // FAIL-CLOSED CSRF: `auth::csrf_layer` rejects every unsafe method lacking a
    // valid token, INCLUDING the Leptos server-fn endpoints under `/api/*`. A
    // mutating `#[server]` fn must attach the token via the first-party
    // `app::CsrfClient` (`#[server(client = CsrfClient)]`), which reads
    // `<meta name="csrf-token">` and sets `X-CSRF-Token`; read-only queries use a
    // real GET (`#[server(input = GetUrl)]`) and are CSRF-exempt by method. Do NOT
    // exempt `/api/*` from CSRF to work around a 403 — see csrf.rs.
    let app = Router::new()
        .merge(oidc::routes())
        .merge(upload::routes(cfg.storage.max_upload_bytes))
        .leptos_routes_with_context(&state, routes, context::provider(&state), {
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

/// Readiness probe: 200 when the database is reachable, 503 otherwise.
async fn ready(State(pool): State<Db>) -> StatusCode {
    match db::ping(&pool).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
