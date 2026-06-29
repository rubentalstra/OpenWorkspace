//! The authorization error and its HTTP mapping.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// An authorization failure. `Denied` is the deny-by-default outcome; the rest are
/// resolution/infrastructure failures. `Display` is lowercase, no trailing period.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AuthzError {
    /// The actor is not permitted to perform the action on the target.
    #[error("insufficient permissions for this action")]
    Denied,
    /// The target of the action does not exist.
    #[error("the target of the action was not found")]
    NotFound,
    /// The action is not applicable to the target kind.
    #[error("the action is not valid for this target")]
    InvalidTarget,
    /// A database error while loading authorization facts or recording the outcome.
    #[error(transparent)]
    Db(#[from] db::DbError),
}

impl IntoResponse for AuthzError {
    fn into_response(self) -> Response {
        // Log the specific variant server-side; return a generic body so internals
        // never leak to the client (mirrors `session::csrf::CsrfError`).
        tracing::warn!(error = %self, "authorization rejected");
        let status = match self {
            Self::Denied => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InvalidTarget => StatusCode::BAD_REQUEST,
            Self::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, status.canonical_reason().unwrap_or("Error")).into_response()
    }
}
