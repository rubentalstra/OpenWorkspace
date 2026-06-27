//! Multi-factor authentication: passkeys (WebAuthn), TOTP, recovery codes, the
//! field-encryption keyring, and the MFA session state.

pub(crate) mod keyring;
pub(crate) mod recovery;
pub(crate) mod state;
pub(crate) mod totp;
pub(crate) mod webauthn;
