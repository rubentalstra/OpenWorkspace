//! Object-storage facade: binary assets over `object_store` (S3-compatible /
//! SeaweedFS, local disk, in-memory) plus the raster upload pipeline over `image`.
//!
//! Vendor types (`object_store`, `image`) never appear in the public API — callers
//! see [`Storage`], [`ProcessedUpload`], [`ImageKind`] and [`StorageError`] only.
//! The security posture (architecture plan §6.6 / Appendix H): raster-only uploads
//! (no SVG, no sanitiser), every upload **decoded, validated and re-encoded to
//! strip metadata**, and access via **short-lived presigned URLs**.

mod backend;
mod key;
mod pipeline;

pub use key::{new_storage_key, thumbnail_key};
pub use pipeline::{ImageKind, ImageLimits, ProcessedUpload, process_upload};

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use object_store::ObjectStore;
use object_store::aws::AmazonS3;

use config::StorageConfig;

/// Object-storage facade errors. `Display` is lowercase, no trailing period.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StorageError {
    /// The bytes are not one of the accepted raster formats (PNG/JPEG/WebP).
    #[error("unsupported image format")]
    UnsupportedFormat,
    /// The upload exceeds the configured byte/dimension/allocation limits.
    #[error("image exceeds the allowed size")]
    TooLarge,
    /// The bytes could not be decoded as a valid image.
    #[error("image could not be decoded")]
    Decode,
    /// Re-encoding the image failed.
    #[error("image could not be re-encoded")]
    Encode,
    /// The object was not found in the backend.
    #[error("object not found")]
    NotFound,
    /// Misconfiguration (bad backend kind, missing local dir, …).
    #[error("storage misconfigured: {0}")]
    Config(String),
    /// The backend rejected or failed an operation (message only — no vendor type).
    #[error("storage backend error: {0}")]
    Backend(String),
    /// Generating a presigned URL failed (or the backend has no presign support).
    #[error("presigned url generation failed: {0}")]
    Sign(String),
}

/// A handle to the configured object store. Clone-cheap (`Arc` inside); build once
/// at startup and share into the web state.
#[derive(Clone)]
pub struct Storage {
    store: Arc<dyn ObjectStore>,
    /// Present only for the S3 backend (the only one that signs URLs).
    signer: Option<Arc<AmazonS3>>,
    presign_ttl: Duration,
}

impl std::fmt::Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage")
            .field("presign_capable", &self.signer.is_some())
            .field("presign_ttl", &self.presign_ttl)
            .finish_non_exhaustive()
    }
}

impl Storage {
    /// Builds the configured backend (`s3` or `local`). The bucket/dir is assumed
    /// to be provisioned out-of-band (dev compose / deploy).
    ///
    /// # Errors
    ///
    /// [`StorageError::Config`] on a bad backend kind, [`StorageError::Backend`]
    /// if the backend client cannot be constructed.
    pub fn from_config(cfg: &StorageConfig) -> Result<Self, StorageError> {
        let built = backend::build(cfg)?;
        Ok(Self {
            store: built.store,
            signer: built.signer,
            presign_ttl: cfg.presign_ttl,
        })
    }

    /// Stores `bytes` at `key` with the given `content_type` (so a later presigned
    /// GET serves the right type).
    ///
    /// # Errors
    ///
    /// [`StorageError::Backend`] on any backend failure.
    pub async fn put(
        &self,
        key: &str,
        bytes: Bytes,
        content_type: &str,
    ) -> Result<(), StorageError> {
        backend::put(self.store.as_ref(), key, bytes, content_type).await
    }

    /// Fetches the object at `key`.
    ///
    /// # Errors
    ///
    /// [`StorageError::NotFound`] if absent, [`StorageError::Backend`] otherwise.
    pub async fn get(&self, key: &str) -> Result<Bytes, StorageError> {
        backend::get(self.store.as_ref(), key).await
    }

    /// Deletes the object at `key` (idempotent on most backends).
    ///
    /// # Errors
    ///
    /// [`StorageError::Backend`] on failure.
    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        backend::delete(self.store.as_ref(), key).await
    }

    /// Generates a short-lived presigned GET URL for `key`, valid for the
    /// configured TTL. Only the S3 backend supports this.
    ///
    /// # Errors
    ///
    /// [`StorageError::Sign`] if the backend has no presign support or signing
    /// fails.
    pub async fn signed_get_url(&self, key: &str) -> Result<String, StorageError> {
        let signer = self.signer.as_ref().ok_or_else(|| {
            StorageError::Sign("backend does not support presigned urls".to_owned())
        })?;
        backend::signed_get_url(signer.as_ref(), key, self.presign_ttl).await
    }
}
