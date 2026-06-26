use anyhow::Context;
use app::{App, shell};
use axum::Router;
use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::routing::get;
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

    let conf = get_configuration(None).context("loading Leptos configuration")?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let state = AppState {
        leptos_options: leptos_options.clone(),
        db: pool,
    };

    let router = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(ready))
        .route(
            "/metrics",
            get(|| async { observability::render_metrics() }),
        )
        .leptos_routes(&state, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

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
