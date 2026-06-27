//! RP-initiated logout URL construction (OpenID Connect RP-Initiated Logout 1.0).

use crate::oidc::provider::DiscoveredProvider;

/// Build the IdP logout URL, or `None` when the provider has RP-initiated logout
/// disabled or advertises no `end_session_endpoint`.
///
/// `post_logout_redirect_uri` must be one the provider has pre-registered. The
/// caller clears the local session first, then redirects the browser here.
#[must_use]
pub fn logout_url(
    provider: &DiscoveredProvider,
    id_token_hint: &str,
    post_logout_redirect_uri: &str,
    state: &str,
) -> Option<String> {
    if !provider.config.rp_initiated_logout {
        return None;
    }
    let endpoint = provider.end_session_endpoint()?;
    let mut url = url::Url::parse(endpoint).ok()?;
    url.query_pairs_mut()
        .append_pair("id_token_hint", id_token_hint)
        .append_pair("client_id", &provider.config.client_id)
        .append_pair("post_logout_redirect_uri", post_logout_redirect_uri)
        .append_pair("state", state);
    Some(url.to_string())
}
