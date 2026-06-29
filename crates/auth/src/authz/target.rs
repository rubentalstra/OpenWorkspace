//! What an action acts upon, before resolution to a `domain::ManagementTarget`.

use domain::{LocationId, OrganizationId, ResourceId};

/// The subject of an authorization check. The backend resolves the location-scoped
/// variants to a `ManagementTarget` (org + location node) via the database.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// A bookable resource (booking actions, resource management).
    Resource(ResourceId),
    /// A location node (hierarchy edits, floor building, scoped admin).
    Location(LocationId),
    /// An organization (org-level management).
    Organization(OrganizationId),
    /// The instance itself (instance configuration).
    Instance,
}

impl Target {
    /// The `(target_type, target_id)` pair recorded in the audit log.
    pub(crate) fn audit_parts(self) -> (Option<&'static str>, Option<uuid::Uuid>) {
        match self {
            Self::Resource(id) => (Some("resource"), Some(id.as_uuid())),
            Self::Location(id) => (Some("location"), Some(id.as_uuid())),
            Self::Organization(id) => (Some("organization"), Some(id.as_uuid())),
            Self::Instance => (Some("instance"), None),
        }
    }
}
