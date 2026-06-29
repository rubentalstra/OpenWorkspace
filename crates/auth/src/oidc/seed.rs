//! Dev-only OIDC provider seeding.

use config::AuthConfig;
use db::Db;
use secrecy::SecretString;

use crate::FieldKeyring;
use crate::oidc::error::OidcError;
use crate::oidc::provider::seal_client_secret;

/// Client secret of the dev Keycloak realm (`deploy/dev/compose.yaml`). Matches
/// the `openworkspace` client in `deploy/dev/keycloak/openworkspace-realm.json`.
/// Dev-only; production providers carry their own operator-supplied secret.
const DEV_KEYCLOAK_SECRET: &str = "test-client-secret-change-me";

/// Seed the local Keycloak SSO provider when [`AuthConfig::dev_seed_keycloak`] is
/// set, idempotently. A no-op in production (the flag defaults to `false`) and on
/// every boot after the first. The client secret is sealed with the field keyring
/// before storage, exactly as a real provider would be.
///
/// Run during privileged startup (owner pool, keyring loaded). Pointing at the
/// compose realm's issuer/client, it makes `/login` show a working "Continue with
/// Keycloak" button and lets the Authorization Code + PKCE flow complete.
///
/// # Errors
///
/// - [`OidcError::Crypto`] if sealing the secret fails.
/// - [`OidcError::Db`] on a database error.
pub async fn seed_dev_oidc_provider(
    db: &Db,
    keyring: &FieldKeyring,
    cfg: &AuthConfig,
) -> Result<(), OidcError> {
    if !cfg.dev_seed_keycloak {
        return Ok(());
    }
    let sealed = seal_client_secret(keyring, &SecretString::from(DEV_KEYCLOAK_SECRET))?;
    let created = db::insert_oidc_provider_if_absent(
        db,
        &db::OidcProviderSeed {
            slug: "keycloak",
            display_name: "Keycloak",
            issuer_url: "http://localhost:8080/realms/openworkspace",
            client_id: "openworkspace",
            client_secret_encrypted: &sealed,
            groups_claim: Some("groups"),
            button_label: Some("Keycloak"),
        },
    )
    .await?;
    if created {
        tracing::info!(
            slug = "keycloak",
            "seeded dev Keycloak OIDC provider; sign in via SSO at /login"
        );
    }
    Ok(())
}
