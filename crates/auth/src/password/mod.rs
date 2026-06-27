//! Local password authentication: the Argon2id backend, first-boot admin
//! bootstrap, and the password policy.

pub(crate) mod policy;

#[cfg(feature = "ssr")]
pub(crate) mod backend;
#[cfg(feature = "ssr")]
pub(crate) mod bootstrap;
