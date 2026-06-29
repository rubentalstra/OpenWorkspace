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

    // Migrations and the system-role seed run under the owner/migrator role
    // (advisory-locked, idempotent — safe to run from web and worker both). Job
    // backends will connect under the least-privilege runtime role in P16.
    let owner_pool = db::connect(&cfg.database.migrator_url, cfg.database.max_connections).await?;
    db::run_migrations(&owner_pool).await?;
    db::seed_system_roles(&owner_pool).await?;
    owner_pool.close().await;

    tracing::info!("openworkspace-worker started (no jobs registered yet)");
    Ok(())
}
