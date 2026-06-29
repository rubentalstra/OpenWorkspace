//! Access-enforcement storage: the database half of P8.
//!
//! - [`principal`] — loaders that materialize the pure `domain` authorization
//!   inputs (`AuthzContext`, `ViewerSegmentation`, `Delegation`) from the schema.
//! - [`segmentation`] — a resource's org/team bindings and the instance mode.
//! - [`context`] — the RLS connection-context (transaction-local GUC) helpers.
//! - [`audit`] — the append-only audit-log writer and its row enums.
//! - [`roles`] — the idempotent system-role seed, sourced from the domain builtins.
//!
//! The pure decision lives in `domain`; this module never decides, it only loads
//! facts and records outcomes.

pub(crate) mod audit;
pub(crate) mod context;
pub(crate) mod principal;
pub(crate) mod roles;
pub(crate) mod segmentation;
pub(crate) mod target;
