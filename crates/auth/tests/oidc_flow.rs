//! Real-Keycloak integration tests for the OIDC relying-party flow (P7 "Done":
//! a full Authorization Code + PKCE flow against a real Keycloak, plus JIT).
//!
//! Gated behind the `oidc-it` feature and a container runtime (podman/docker via
//! `DOCKER_HOST`), so the default `cargo nextest run --workspace` neither compiles
//! nor runs them. Run with:
//!
//! ```text
//! DOCKER_HOST=unix://$(podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}') \
//!   TESTCONTAINERS_RYUK_DISABLED=true \
//!   cargo nextest run -p auth --features oidc-it
//! ```
//!
//! One Keycloak container is shared across the binary; each test gets its own
//! auto-migrated Postgres via `#[sqlx::test]`. The test acts as the browser: it
//! GETs the authorize URL, submits the Keycloak login form, and intercepts the
//! redirect to the (never-served) callback to recover `code`/`state`/`iss`.
#![cfg(feature = "oidc-it")]
#![expect(
    clippy::expect_used,
    reason = "integration-test harness: a failed setup expectation should abort the test"
)]

use std::time::Duration;

use auth::{FieldKeyring, OidcCallback, OidcHttpClient, ProviderRegistry};
use base64::Engine as _;
use secrecy::SecretString;
use sqlx::PgPool;
use testcontainers::core::wait::HttpWaitStrategy;
use testcontainers::core::{ContainerPort, IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::sync::OnceCell;
use uuid::Uuid;

/// The RP callback the realm registers. No server listens here — the headless
/// browser intercepts the redirect to it.
const REDIRECT_URI: &str = "http://localhost:3000/auth/keycloak/callback";
const CLIENT_SECRET: &str = "test-client-secret-change-me";

/// A 32-byte all-zero root KEK (base64) — deterministic, test-only.
fn dev_kek() -> SecretString {
    SecretString::from(base64::engine::general_purpose::STANDARD.encode([0u8; 32]))
}

struct Keycloak {
    _container: ContainerAsync<GenericImage>,
    issuer: String,
}

static KEYCLOAK: OnceCell<Keycloak> = OnceCell::const_new();

/// The shared Keycloak (cold-started once per test binary).
async fn keycloak() -> &'static Keycloak {
    KEYCLOAK.get_or_init(start_keycloak).await
}

async fn start_keycloak() -> Keycloak {
    let realm = include_str!("../../../deploy/dev/keycloak/openworkspace-realm.json");
    let container = GenericImage::new("quay.io/keycloak/keycloak", "26.6.3")
        .with_exposed_port(8080.tcp())
        .with_wait_for(WaitFor::http(
            HttpWaitStrategy::new("/realms/openworkspace/.well-known/openid-configuration")
                .with_port(ContainerPort::Tcp(8080))
                .with_expected_status_code(200u16),
        ))
        .with_startup_timeout(Duration::from_mins(3))
        .with_env_var("KC_BOOTSTRAP_ADMIN_USERNAME", "admin")
        .with_env_var("KC_BOOTSTRAP_ADMIN_PASSWORD", "admin")
        .with_copy_to(
            "/opt/keycloak/data/import/openworkspace-realm.json",
            realm.as_bytes().to_vec(),
        )
        .with_cmd(["start-dev", "--import-realm"])
        .start()
        .await
        .expect("keycloak should start");
    let host = container.get_host().await.expect("container host");
    let port = container
        .get_host_port_ipv4(8080.tcp())
        .await
        .expect("mapped 8080");
    Keycloak {
        _container: container,
        issuer: format!("http://{host}:{port}/realms/openworkspace"),
    }
}

struct Services {
    registry: ProviderRegistry,
    http: OidcHttpClient,
}

fn services(pool: &PgPool, keyring: FieldKeyring) -> Services {
    let http = OidcHttpClient::new(Duration::from_secs(10)).expect("http client");
    let registry = ProviderRegistry::new(pool.clone(), http.clone(), keyring);
    Services { registry, http }
}

/// Insert a `keycloak`-slug provider row pointing at the live issuer, with the
/// client secret sealed through the crypto facade.
async fn seed_provider(pool: &PgPool, keyring: &FieldKeyring, issuer: &str) -> Uuid {
    let secret = auth::seal_client_secret(keyring, &SecretString::from(CLIENT_SECRET.to_owned()))
        .expect("seal secret");
    sqlx::query_scalar(
        "INSERT INTO oidc_providers \
         (display_name, slug, issuer_url, client_id, client_auth_method, client_secret_encrypted, groups_claim) \
         VALUES ('Keycloak', 'keycloak'::citext, $1, 'openworkspace', 'client_secret_basic', $2, 'groups') \
         RETURNING id",
    )
    .bind(issuer)
    .bind(&secret)
    .fetch_one(pool)
    .await
    .expect("seed provider")
}

/// Drive the Keycloak login form headlessly and recover the callback params.
async fn headless_login(authorize_url: &str, username: &str, password: &str) -> OidcCallback {
    let browser = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.url().path().contains("/auth/keycloak/callback") {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .expect("browser client");

    let login_html = browser
        .get(authorize_url)
        .send()
        .await
        .expect("GET authorize")
        .text()
        .await
        .expect("login page body");
    let action = extract_form_action(&login_html);

    let response = browser
        .post(&action)
        .form(&[
            ("username", username),
            ("password", password),
            ("credentialId", ""),
        ])
        .send()
        .await
        .expect("POST credentials");

    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
        .expect("redirect to the callback");
    parse_callback(&location)
}

/// Extract the `kc-form-login` action URL from the Keycloak login page.
fn extract_form_action(html: &str) -> String {
    let anchor = html
        .find("kc-form-login")
        .expect("login form present in the page");
    let rest = &html[anchor..];
    let start = rest.find("action=\"").expect("form action") + "action=\"".len();
    let end = rest[start..].find('"').expect("action close quote");
    rest[start..start + end].replace("&amp;", "&")
}

fn parse_callback(location: &str) -> OidcCallback {
    let url = reqwest::Url::parse(location).expect("callback url");
    let mut code = None;
    let mut state = None;
    let mut iss = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "iss" => iss = Some(value.into_owned()),
            _ => {}
        }
    }
    OidcCallback {
        code: code.expect("authorization code"),
        state: state.expect("state"),
        iss,
    }
}

#[sqlx::test(migrations = "../db/migrations")]
async fn full_auth_code_pkce_login_with_groups_and_jit(pool: PgPool) {
    let kc = keycloak().await;
    let keyring = FieldKeyring::load(&pool, &dev_kek()).await.unwrap();
    seed_provider(&pool, &keyring, &kc.issuer).await;
    let svc = services(&pool, keyring);

    let provider = svc
        .registry
        .discovered("keycloak")
        .await
        .expect("discovery");
    let auth_request = auth::begin_login(&provider, REDIRECT_URI, "/".to_owned()).unwrap();
    let callback = headless_login(&auth_request.authorize_url, "bob", "bobpw").await;

    // RFC 9207: Keycloak returns iss; it must match the issuer.
    assert_eq!(callback.iss.as_deref(), Some(kc.issuer.as_str()));

    let identity = auth::complete_login(
        &provider,
        &svc.http,
        REDIRECT_URI,
        callback,
        auth_request.transaction,
    )
    .await
    .expect("login completes");

    assert!(!identity.subject.is_empty(), "subject (sub) present");
    assert_eq!(identity.email.as_deref(), Some("bob@example.test"));
    assert!(identity.email_verified);
    assert_eq!(
        identity.groups,
        vec!["staff".to_owned()],
        "groups from UserInfo"
    );

    let user_id = auth::provision_user(&pool, &provider, &identity)
        .await
        .expect("provision");
    let linked: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM oidc_identities oi JOIN users u ON u.id = oi.user_id \
         WHERE u.email = 'bob@example.test'::citext",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(linked, 1, "JIT created and linked the user");
    let _ = user_id;
}

#[sqlx::test(migrations = "../db/migrations")]
async fn state_mismatch_is_rejected(pool: PgPool) {
    let kc = keycloak().await;
    let keyring = FieldKeyring::load(&pool, &dev_kek()).await.unwrap();
    seed_provider(&pool, &keyring, &kc.issuer).await;
    let svc = services(&pool, keyring);
    let provider = svc
        .registry
        .discovered("keycloak")
        .await
        .expect("discovery");
    let auth_request = auth::begin_login(&provider, REDIRECT_URI, "/".to_owned()).unwrap();
    let mut callback = headless_login(&auth_request.authorize_url, "bob", "bobpw").await;

    // Tamper with the returned state; the facade must reject before token exchange.
    callback.state = "forged-state".to_owned();
    let err = auth::complete_login(
        &provider,
        &svc.http,
        REDIRECT_URI,
        callback,
        auth_request.transaction,
    )
    .await
    .expect_err("a forged state must be rejected");
    assert!(matches!(err, auth::OidcError::StateMismatch));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn verified_email_links_to_existing_account(pool: PgPool) {
    let kc = keycloak().await;
    let keyring = FieldKeyring::load(&pool, &dev_kek()).await.unwrap();
    seed_provider(&pool, &keyring, &kc.issuer).await;
    let svc = services(&pool, keyring);

    // A pre-existing local account with the same (verified) email.
    let existing: Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, display_name, webauthn_user_handle) \
         VALUES ('linkme@example.test'::citext, 'Link Me', $1) RETURNING id",
    )
    .bind(Uuid::new_v4().as_bytes().to_vec())
    .fetch_one(&pool)
    .await
    .unwrap();

    let provider = svc
        .registry
        .discovered("keycloak")
        .await
        .expect("discovery");
    let auth_request = auth::begin_login(&provider, REDIRECT_URI, "/".to_owned()).unwrap();
    let callback = headless_login(&auth_request.authorize_url, "linkme", "linkmepw").await;
    let identity = auth::complete_login(
        &provider,
        &svc.http,
        REDIRECT_URI,
        callback,
        auth_request.transaction,
    )
    .await
    .expect("login completes");
    let user_id = auth::provision_user(&pool, &provider, &identity)
        .await
        .expect("provision");

    assert_eq!(
        user_id.as_uuid(),
        existing,
        "linked to the existing account"
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "no duplicate account created");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn rp_initiated_logout_url_is_built(pool: PgPool) {
    let kc = keycloak().await;
    let keyring = FieldKeyring::load(&pool, &dev_kek()).await.unwrap();
    seed_provider(&pool, &keyring, &kc.issuer).await;
    let svc = services(&pool, keyring);
    let provider = svc
        .registry
        .discovered("keycloak")
        .await
        .expect("discovery");
    let auth_request = auth::begin_login(&provider, REDIRECT_URI, "/".to_owned()).unwrap();
    let callback = headless_login(&auth_request.authorize_url, "bob", "bobpw").await;
    let identity = auth::complete_login(
        &provider,
        &svc.http,
        REDIRECT_URI,
        callback,
        auth_request.transaction,
    )
    .await
    .expect("login completes");

    let url = auth::logout_url(
        &provider,
        &identity.id_token_compact,
        "http://localhost:3000/auth/keycloak/logged-out",
        "logout-state",
    )
    .expect("a logout URL when end_session_endpoint is advertised");
    assert!(url.contains("/protocol/openid-connect/logout"), "{url}");
    assert!(url.contains("id_token_hint="));
    assert!(url.contains("post_logout_redirect_uri="));
}
