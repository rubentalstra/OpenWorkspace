//! Authorization enforcement — the single `AuthzBackend` where permissions are
//! decided (architecture plan §6.2).
//!
//! The pure decision lives in `domain` (`authorize`, `Delegation::as_principal`,
//! `segmentation::visible`); this submodule wires it to the database: it loads the
//! actor's [`AuthzContext`](domain::authz::AuthzContext), resolves the
//! [`Target`] to a `ManagementTarget`, applies delegation, calls the pure
//! decision, and records every outcome to the audit log. RLS in the database is
//! the defence-in-depth backstop; this is the authoritative gate.

mod backend;
mod error;
mod target;

pub use backend::AuthzBackend;
pub use error::AuthzError;
pub use target::Target;
