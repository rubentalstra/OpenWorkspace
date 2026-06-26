//! OpenWorkspace background worker (apalis). Boots the platform services; job
//! definitions arrive in later phases.

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

    tracing::info!("openworkspace-worker started (no jobs registered yet)");
    Ok(())
}
