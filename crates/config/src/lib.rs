//! Typed application configuration, loaded once at startup (figment + secrecy).
//!
//! Sources, lowest to highest precedence: serde defaults, `config/app.toml`
//! (organised as `[default]`/`[dev]`/`[prod]`/`[global]` profile tables), then
//! `APP_`-prefixed environment variables (`APP_DATABASE__URL`, `__` = nesting).

use figment::Figment;
use figment::providers::{Env, Format, Toml};
use secrecy::SecretString;
use serde::Deserialize;

/// Top-level configuration. Construct with [`load`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub addr: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:3000".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// PostgreSQL connection string. A secret: redacted in `Debug`/logs.
    pub url: SecretString,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: SecretString::from(String::new()),
            max_connections: 5,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ObservabilityConfig {
    pub service_name: String,
    pub log_filter: String,
    /// When set, traces are exported to this OTLP/gRPC endpoint. Off when `None`.
    pub otlp_endpoint: Option<String>,
    pub metrics_enabled: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            service_name: "openworkspace".to_owned(),
            log_filter: "info".to_owned(),
            otlp_endpoint: None,
            metrics_enabled: true,
        }
    }
}

/// Errors raised while loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to load configuration")]
    Figment(#[from] Box<figment::Error>),
}

/// Load and validate the configuration for the active profile (`APP_PROFILE`,
/// default `dev`). The app holds the result and passes slices to the facades.
pub fn load() -> Result<AppConfig, ConfigError> {
    let profile = std::env::var("APP_PROFILE").unwrap_or_else(|_| "dev".to_owned());
    let config = Figment::new()
        .merge(Toml::file("config/app.toml").nested())
        .merge(Env::prefixed("APP_").split("__").global())
        .select(profile)
        .extract()
        .map_err(Box::new)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn defaults_extract_cleanly() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.server.addr, "127.0.0.1:3000");
        assert_eq!(cfg.database.max_connections, 5);
        assert!(cfg.observability.metrics_enabled);
        assert!(cfg.database.url.expose_secret().is_empty());
    }
}
