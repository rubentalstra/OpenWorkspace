//! Identity, credential and key-material queries — the storage the `auth` crate
//! builds on: local password credentials, passkeys, TOTP, recovery codes and the
//! `crypto_keys` envelope.

pub(crate) mod credentials;
pub(crate) mod crypto_keys;
pub(crate) mod mfa;
