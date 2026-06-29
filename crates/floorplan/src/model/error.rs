//! Typed scene errors. `Display` lowercase, no trailing period.

/// What can go wrong loading or validating a [`crate::Scene`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SceneError {
    /// The (possibly migrated) JSON did not match the current scene shape.
    #[error("scene deserialization failed: {0}")]
    Deserialize(String),
    /// The stored schema version is newer than this build understands.
    #[error("scene schema version {found} is newer than supported {supported}")]
    FutureVersion {
        /// The version found on the stored scene.
        found: u32,
        /// The newest version this build can load.
        supported: u32,
    },
    /// Two nodes share a `scene_node_id`.
    #[error("duplicate scene node id `{0}`")]
    DuplicateNodeId(String),
    /// A node's geometry is malformed (e.g. a polygon with < 3 points).
    #[error("invalid geometry for node `{0}`")]
    InvalidGeometry(String),
    /// The scene's view box is degenerate (non-finite or non-positive size).
    #[error("invalid view box")]
    InvalidViewBox,
}
