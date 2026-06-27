//! OIDC relying-party facade (P7): Authorization Code + PKCE S256, discovery,
//! JWKS, full ID-token validation, just-in-time provisioning, account linking,
//! group→role mapping and RP-initiated logout — entirely on the `openidconnect`
//! and `oauth2` crates, which never leak past this module group.
//!
//! The flow: `ProviderRegistry::discovered` resolves and caches a provider;
//! `begin_login` builds the authorize URL and the transaction to persist;
//! `complete_login` validates the callback and ID token; `provision_user` maps the
//! verified identity to a local user; the server then mints its own session via
//! `login_verified_user`.

pub(crate) mod error;
pub(crate) mod flow;
pub(crate) mod http;
pub(crate) mod logout;
pub(crate) mod provider;
pub(crate) mod provision;
pub(crate) mod session;
