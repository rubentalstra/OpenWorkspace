//! Authentication facade: local Argon2id accounts on server-side Postgres
//! sessions, with CSRF protection.
//!
//! This is the single crate that depends on `axum-login`, `tower-sessions` and
//! `sqlx` for sessions; the session store is a first-party [`store::PgSessionStore`]
//! over the workspace's sqlx 0.9 `db::Db` pool, so none of those vendor types
//! appear in any other crate's public API. The session/auth machinery is gated
//! behind the `ssr` feature so the hydrate (wasm) build never pulls it in.
//!
//! P5 implements only **authentication** (`AuthnBackend`). Authorization
//! enforcement wires `domain::authz` into requests in P8 and is intentionally
//! absent here.

use secrecy::SecretString;

/// The account status the facade exposes — a domain mirror of the persistence
/// `user_status` so callers never see the `db`/vendor enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UserStatus {
    /// Active; may authenticate.
    Active,
    /// Suspended; rejected at login.
    Suspended,
    /// Deactivated; rejected at login.
    Deactivated,
}

impl From<db::UserStatusRow> for UserStatus {
    fn from(value: db::UserStatusRow) -> Self {
        match value {
            db::UserStatusRow::Active => Self::Active,
            db::UserStatusRow::Suspended => Self::Suspended,
            db::UserStatusRow::Deactivated => Self::Deactivated,
        }
    }
}

/// Login credentials. `Debug` deliberately omits the password so it never
/// reaches logs or panic messages.
#[derive(Clone, serde::Deserialize)]
pub struct Credentials {
    /// The login identifier (email).
    pub email: String,
    /// The plaintext password.
    pub password: SecretString,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("email", &self.email)
            .field("password", &"<redacted>")
            .finish()
    }
}

/// An authenticated user. The password hash is private and excluded from
/// `Debug`; it backs `session_auth_hash` so changing the password invalidates
/// live sessions.
#[derive(Clone)]
pub struct User {
    /// Strongly-typed user id.
    pub id: domain::UserId,
    /// The user's email.
    pub email: String,
    /// Account status.
    pub status: UserStatus,
    /// Whether the account is flagged to change its password before normal use
    /// (e.g. the bootstrapped instance admin).
    ///
    /// NOTE: this flag is *surfaced* but not yet *enforced*. Forced-change
    /// enforcement at login requires the password-change flow, which is a later
    /// phase; it is intentionally deferred here rather than silently pretended.
    pub must_change_password: bool,
    password_hash: String,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("email", &self.email)
            .field("status", &self.status)
            .field("must_change_password", &self.must_change_password)
            .field("password_hash", &"<redacted>")
            .finish()
    }
}

impl User {
    /// Whether this user may authenticate (only [`UserStatus::Active`]).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }

    /// Builds a [`User`] from a credential row but with an explicit password hash,
    /// used after rehash-on-login to bind `session_auth_hash` to the hash that was
    /// actually persisted (not the stale row value). See `backend::Backend::rehash`.
    #[cfg(feature = "ssr")]
    pub(crate) fn from_row_with_hash(row: db::CredentialRow, password_hash: String) -> Self {
        Self {
            id: domain::UserId::new(row.id),
            email: row.email,
            status: row.status.into(),
            must_change_password: row.must_change,
            password_hash,
        }
    }
}

impl From<db::CredentialRow> for User {
    fn from(row: db::CredentialRow) -> Self {
        Self {
            id: domain::UserId::new(row.id),
            email: row.email,
            status: row.status.into(),
            must_change_password: row.must_change,
            password_hash: row.password_hash,
        }
    }
}

// Submodule groups — see CLAUDE.md "Module layout". `password` carries the
// always-available policy plus the ssr-only backend/bootstrap; `session` and
// `mfa` are ssr-only. The public surface is re-exported flat at the crate root.
#[cfg(feature = "ssr")]
mod mfa;
mod password;
#[cfg(feature = "ssr")]
mod session;

pub use password::policy::{MIN_PASSWORD_LENGTH, PasswordPolicyError, validate_password};

#[cfg(feature = "ssr")]
pub use mfa::keyring::FieldKeyring;
#[cfg(feature = "ssr")]
pub use mfa::recovery::{
    RECOVERY_CODE_COUNT, RecoveryCodes, generate_recovery_codes, hash_submitted_code,
};
#[cfg(feature = "ssr")]
pub use mfa::state::{MfaSession, PendingMfa, second_factor_required};
#[cfg(feature = "ssr")]
pub use mfa::totp::{StoredTotp, TotpEnrollment, TotpService};
#[cfg(feature = "ssr")]
pub use mfa::webauthn::{
    AuthOutcome, CreationChallengeResponse, DiscoverableAuthentication, PasskeyAuthentication,
    PasskeyCandidate, PasskeyRegistration, PublicKeyCredential, RegisterPublicKeyCredential,
    RegisteredPasskey, RequestChallengeResponse, WebauthnService,
};
#[cfg(feature = "ssr")]
pub use password::backend::{AuthError, Backend};
#[cfg(feature = "ssr")]
pub use password::bootstrap::bootstrap_admin;
#[cfg(feature = "ssr")]
pub use session::csrf::{CsrfError, CsrfToken, csrf_layer, hidden_field, rotate_csrf_token};
#[cfg(feature = "ssr")]
pub use session::layer::{
    AuthSession, ReauthError, build_auth_layer, cycle_session_id, login_verified_user,
    rebind_after_password_change,
};
#[cfg(feature = "ssr")]
pub use session::store::{PgSessionStore, spawn_session_reaper};
