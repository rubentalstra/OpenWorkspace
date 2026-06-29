//! Per-request Leptos context for SSR renders AND server functions.
//!
//! `leptos_routes_with_context` runs [`provider`] for every server-rendered page
//! and every `#[server]` call, so app-side server functions can `expect_context`
//! the services they need — `Db`, `AuthzBackend`, `WebauthnService`, `TotpService`,
//! `ProviderRegistry`, the public base URL — plus the CSRF token, without ever
//! naming the server's `AppState`.

use app::{CsrfToken, PublicBaseUrl};
use leptos::prelude::*;

use crate::AppState;

/// Build the per-request context-provider closure for `leptos_routes_with_context`.
pub(crate) fn provider(state: &AppState) -> impl Fn() + Clone + Send + 'static {
    let db = state.db.clone();
    let authz = state.authz.clone();
    let webauthn = state.webauthn.clone();
    let totp = state.totp.clone();
    let registry = state.oidc.registry.clone();
    let base_url = state.oidc.base_url.to_string();
    move || {
        provide_context(db.clone());
        provide_context(authz.clone());
        provide_context(webauthn.clone());
        provide_context(totp.clone());
        provide_context(registry.clone());
        provide_context(PublicBaseUrl(base_url.clone()));
        provide_csrf();
    }
}

/// Surface the CSRF token (set in request extensions by the CSRF middleware) so the
/// `App` shell can render it into a `<meta>` tag and mutations can echo it.
fn provide_csrf() {
    if let Some(parts) = use_context::<axum::http::request::Parts>()
        && let Some(token) = parts.extensions.get::<auth::CsrfToken>()
    {
        provide_context(CsrfToken(token.as_str().to_owned()));
    }
}
