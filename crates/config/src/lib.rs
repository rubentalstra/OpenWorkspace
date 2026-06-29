//! Typed application configuration, loaded once at startup (figment + secrecy).
//!
//! Sources, lowest to highest precedence: serde defaults, `config/app.toml`
//! (organised as `[default]`/`[dev]`/`[prod]`/`[global]` profile tables), then
//! `APP_`-prefixed environment variables (`APP_DATABASE__URL`, `__` = nesting).

use std::time::Duration;

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
    pub auth: AuthConfig,
    pub storage: StorageConfig,
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
    /// Runtime connection string for the least-privilege `openworkspace_app` role
    /// the app and worker serve under (DML only, governed by RLS, no DDL, no
    /// audit-log mutation). A secret: redacted in `Debug`/logs.
    pub url: SecretString,
    /// Owner/migrator connection string used once at startup to run migrations and
    /// privileged setup (role seed, bootstrap admin, keyring). Superuser/owner, so
    /// it bypasses RLS. A secret: redacted. Env: `APP_DATABASE__MIGRATOR_URL`.
    pub migrator_url: SecretString,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: SecretString::from(String::new()),
            migrator_url: SecretString::from(String::new()),
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

/// Authentication and session configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    /// Cookie-signing key. Reserved for P7+ (signed/encrypted cookies); unused in
    /// P5, where session data lives server-side in Postgres. A secret: redacted.
    pub session_key: SecretString,
    /// Optional Argon2 pepper: a server-held secret keyed into every password
    /// hash so a stolen database alone cannot be brute-forced. `None` disables it.
    pub argon2_pepper: Option<SecretString>,
    /// Sliding idle timeout: a session expires this long after the last request.
    /// Parsed from a humantime string (e.g. `8h`, `30m`). Defaults to 8 hours.
    #[serde(with = "humantime_serde")]
    pub session_idle_timeout: Duration,
    /// Email of the first-boot instance admin. Bootstrap runs only when this AND
    /// [`AuthConfig::bootstrap_admin_password`] are set and no instance admin yet
    /// exists. `None` skips it. Env: `APP_AUTH__BOOTSTRAP_ADMIN_EMAIL`.
    pub bootstrap_admin_email: Option<String>,
    /// Operator-supplied password for the first-boot instance admin. A secret:
    /// redacted in `Debug`/logs and never logged in plaintext. Bootstrap is a
    /// no-op (with a warning) when the email is set but this is not. `None` skips
    /// it. Env: `APP_AUTH__BOOTSTRAP_ADMIN_PASSWORD`.
    pub bootstrap_admin_password: Option<SecretString>,
    /// Root key-encryption key for field encryption (TOTP secrets today), as
    /// base64 of exactly 32 bytes. Wraps the per-purpose data keys in
    /// `crypto_keys`. A secret: redacted. Empty disables field encryption and
    /// makes TOTP enrolment fail with a clear error. Env:
    /// `APP_AUTH__FIELD_ENCRYPTION_KEY`.
    pub field_encryption_key: SecretString,
    /// WebAuthn relying-party id: the registrable domain passkeys are bound to
    /// (e.g. `localhost`, `openworkspace.example`). Must be an effective domain
    /// of [`AuthConfig::webauthn_rp_origin`]. Env: `APP_AUTH__WEBAUTHN_RP_ID`.
    pub webauthn_rp_id: String,
    /// WebAuthn relying-party origin: the exact scheme/host/port the app is
    /// served from (e.g. `http://localhost:3000`). Env:
    /// `APP_AUTH__WEBAUTHN_RP_ORIGIN`.
    pub webauthn_rp_origin: String,
    /// Human-readable relying-party name shown by authenticators during a
    /// ceremony. Env: `APP_AUTH__WEBAUTHN_RP_NAME`.
    pub webauthn_rp_name: String,
    /// Public origin the app is reached at (scheme/host[/port], no trailing slash),
    /// e.g. `https://workspace.example`. OIDC redirect and post-logout URIs are
    /// derived from it as `{base}/auth/{slug}/callback` and `{base}/auth/{slug}/logged-out`,
    /// and must exactly match what each provider has registered. Env:
    /// `APP_AUTH__PUBLIC_BASE_URL`.
    pub public_base_url: String,
    /// How long discovery metadata and JWKS are cached before a refresh. Parsed
    /// from a humantime string (e.g. `1h`). Env: `APP_AUTH__OIDC_DISCOVERY_CACHE`.
    #[serde(with = "humantime_serde")]
    pub oidc_discovery_cache: Duration,
    /// Per-request timeout for outbound OIDC HTTP calls (discovery, JWKS, token,
    /// userinfo). Parsed from a humantime string (e.g. `10s`). Env:
    /// `APP_AUTH__OIDC_HTTP_TIMEOUT`.
    #[serde(with = "humantime_serde")]
    pub oidc_http_timeout: Duration,
    /// Default leeway allowed on ID-token `exp`/`iat`/`nbf` to tolerate clock drift,
    /// when a provider row does not set its own `clock_skew_seconds`. Parsed from a
    /// humantime string (e.g. `60s`). Env: `APP_AUTH__OIDC_DEFAULT_CLOCK_SKEW`.
    #[serde(with = "humantime_serde")]
    pub oidc_default_clock_skew: Duration,
    /// Dev-only: seed the local Keycloak OIDC provider row (slug `keycloak`,
    /// pointing at the compose realm) at startup, idempotently. **Never enable in
    /// production** — real providers are configured per deployment. Defaults to
    /// `false`; `[dev.auth]` turns it on. Env: `APP_AUTH__DEV_SEED_KEYCLOAK`.
    pub dev_seed_keycloak: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_key: SecretString::from(String::new()),
            argon2_pepper: None,
            session_idle_timeout: Duration::from_hours(8),
            bootstrap_admin_email: None,
            bootstrap_admin_password: None,
            field_encryption_key: SecretString::from(String::new()),
            webauthn_rp_id: "localhost".to_owned(),
            webauthn_rp_origin: "http://localhost:3000".to_owned(),
            webauthn_rp_name: "OpenWorkspace".to_owned(),
            public_base_url: "http://localhost:3000".to_owned(),
            oidc_discovery_cache: Duration::from_hours(1),
            oidc_http_timeout: Duration::from_secs(10),
            oidc_default_clock_skew: Duration::from_mins(1),
            dev_seed_keycloak: false,
        }
    }
}

/// Object-storage (binary assets) configuration. `object_store` over an
/// S3-compatible backend (SeaweedFS in dev/on-prem) or local disk.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// Backend selector: `"s3"` (S3-compatible, incl. SeaweedFS) or `"local"`
    /// (filesystem under [`StorageConfig::local_dir`], no presigned URLs).
    pub kind: String,
    /// S3 endpoint URL (e.g. `http://localhost:8333` for dev SeaweedFS). Empty
    /// uses the AWS default endpoint for the region.
    pub s3_endpoint: String,
    /// S3 region. SeaweedFS ignores it but sigv4 still requires a value.
    pub s3_region: String,
    /// Bucket holding all assets. Provisioned out-of-band (dev compose / deploy).
    pub s3_bucket: String,
    /// S3 access key id. A secret: redacted. Env: `APP_STORAGE__S3_ACCESS_KEY`.
    pub s3_access_key: SecretString,
    /// S3 secret access key. A secret: redacted. Env: `APP_STORAGE__S3_SECRET_KEY`.
    pub s3_secret_key: SecretString,
    /// Allow plain HTTP to the endpoint (dev SeaweedFS). Keep `false` in prod.
    pub s3_allow_http: bool,
    /// Use path-style addressing (`endpoint/bucket/key`). Required by SeaweedFS.
    pub s3_force_path_style: bool,
    /// How long a generated presigned URL stays valid. Humantime (e.g. `15m`).
    #[serde(with = "humantime_serde")]
    pub presign_ttl: Duration,
    /// Hard cap on an accepted upload's byte size (pre-decode rejection).
    pub max_upload_bytes: u64,
    /// Longest edge (px) of the generated thumbnail variant.
    pub thumbnail_max_px: u32,
    /// Root directory for the `local` backend. `None` unless `kind = "local"`.
    pub local_dir: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            kind: "s3".to_owned(),
            s3_endpoint: String::new(),
            s3_region: "us-east-1".to_owned(),
            s3_bucket: String::new(),
            s3_access_key: SecretString::from(String::new()),
            s3_secret_key: SecretString::from(String::new()),
            s3_allow_http: false,
            s3_force_path_style: true,
            presign_ttl: Duration::from_mins(15),
            max_upload_bytes: 10 * 1024 * 1024,
            thumbnail_max_px: 512,
            local_dir: None,
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

    #[test]
    fn auth_defaults_are_safe() {
        let cfg = AppConfig::default();
        // An absent Option<SecretString> defaults to None (pepper disabled).
        assert!(cfg.auth.argon2_pepper.is_none());
        assert!(cfg.auth.bootstrap_admin_email.is_none());
        assert!(cfg.auth.bootstrap_admin_password.is_none());
        assert_eq!(cfg.auth.session_idle_timeout, Duration::from_hours(8));
        assert!(cfg.auth.session_key.expose_secret().is_empty());
        // Field encryption is off by default (empty key); WebAuthn RP defaults to localhost.
        assert!(cfg.auth.field_encryption_key.expose_secret().is_empty());
        assert_eq!(cfg.auth.webauthn_rp_id, "localhost");
        assert_eq!(cfg.auth.webauthn_rp_origin, "http://localhost:3000");
        assert_eq!(cfg.auth.webauthn_rp_name, "OpenWorkspace");
        // Provider seeding is a dev-only convenience, off unless a profile opts in.
        assert!(!cfg.auth.dev_seed_keycloak);
    }
}
