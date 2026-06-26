//! Telemetry facade: structured tracing, optional OTLP trace export and a
//! Prometheus metrics recorder — all behind intent-named API. No otel/metrics
//! types appear in the public surface except the opaque [`TelemetryGuard`].

use std::sync::OnceLock;
use std::time::Duration;

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, fmt};

static METRICS: OnceLock<PrometheusHandle> = OnceLock::new();

/// Telemetry options, mapped from configuration by the app.
#[derive(Debug, Clone)]
pub struct Options {
    pub service_name: String,
    pub log_filter: String,
    /// When `Some`, spans export to this OTLP/gRPC endpoint; otherwise OTLP is off.
    pub otlp_endpoint: Option<String>,
    pub metrics_enabled: bool,
}

/// Owns the trace pipeline; flushes batched spans on drop. Hold for the process lifetime.
pub struct TelemetryGuard {
    provider: Option<SdkTracerProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.provider.take() {
            // Flush batched spans before exit; ignore shutdown errors.
            drop(provider.shutdown());
        }
    }
}

/// Current metrics in Prometheus text exposition format (empty when metrics are
/// off or telemetry has not been initialised).
#[must_use]
pub fn render_metrics() -> String {
    METRICS
        .get()
        .map(PrometheusHandle::render)
        .unwrap_or_default()
}

/// Initialise the global subscriber (`EnvFilter` + fmt), the W3C trace-context
/// propagator, optional OTLP export, and the Prometheus recorder. Call once at
/// startup from within the Tokio runtime.
pub fn init(opts: &Options) -> anyhow::Result<TelemetryGuard> {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let filter = EnvFilter::try_new(&opts.log_filter).unwrap_or_else(|_| EnvFilter::new("info"));
    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer());

    let provider = if let Some(endpoint) = &opts.otlp_endpoint {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.clone())
            .build()?;
        let provider = SdkTracerProvider::builder()
            .with_resource(
                Resource::builder()
                    .with_service_name(opts.service_name.clone())
                    .build(),
            )
            .with_batch_exporter(exporter)
            .build();
        let tracer = provider.tracer(opts.service_name.clone());
        registry
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();
        Some(provider)
    } else {
        registry.init();
        None
    };

    if opts.metrics_enabled {
        let handle = PrometheusBuilder::new().install_recorder()?;
        // First init wins; store the handle so render_metrics() can reach it.
        METRICS.get_or_init(|| handle.clone());
        // We expose /metrics ourselves, so maintain histograms/idle series on a timer.
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(5));
            loop {
                ticker.tick().await;
                handle.run_upkeep();
            }
        });
    }

    Ok(TelemetryGuard { provider })
}
