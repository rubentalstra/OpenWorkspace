//! Thin wrapper over `object_store`: build the configured backend and run
//! put/get/delete + presigned-URL signing. Vendor errors are mapped to
//! [`StorageError`] so no `object_store` type crosses the facade.

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use object_store::aws::{AmazonS3, AmazonS3Builder};
use object_store::local::LocalFileSystem;
use object_store::memory::InMemory;
use object_store::path::Path as ObjectPath;
use object_store::signer::Signer as _;
use object_store::{
    Attribute, AttributeValue, Attributes, ObjectStore, ObjectStoreExt as _, PutOptions, PutPayload,
};
use secrecy::ExposeSecret as _;

use config::StorageConfig;

use crate::StorageError;

/// A constructed backend: the dyn store (all backends) plus the concrete
/// `AmazonS3` when the backend can sign URLs (S3 only).
pub(crate) struct Built {
    pub store: Arc<dyn ObjectStore>,
    pub signer: Option<Arc<AmazonS3>>,
}

/// Builds the configured store.
pub(crate) fn build(cfg: &StorageConfig) -> Result<Built, StorageError> {
    match cfg.kind.as_str() {
        "s3" => {
            let mut builder = AmazonS3Builder::new()
                .with_region(cfg.s3_region.as_str())
                .with_bucket_name(cfg.s3_bucket.as_str())
                .with_access_key_id(cfg.s3_access_key.expose_secret())
                .with_secret_access_key(cfg.s3_secret_key.expose_secret())
                .with_allow_http(cfg.s3_allow_http)
                .with_virtual_hosted_style_request(!cfg.s3_force_path_style);
            if !cfg.s3_endpoint.is_empty() {
                builder = builder.with_endpoint(cfg.s3_endpoint.as_str());
            }
            let s3 = Arc::new(builder.build().map_err(map_err)?);
            let signer = Arc::clone(&s3);
            // Move the concrete Arc into the trait-object binding (unsize coercion).
            let store: Arc<dyn ObjectStore> = s3;
            Ok(Built {
                store,
                signer: Some(signer),
            })
        }
        "local" => {
            let dir = cfg.local_dir.as_deref().ok_or_else(|| {
                StorageError::Config("local_dir is required for kind=local".to_owned())
            })?;
            let lfs = LocalFileSystem::new_with_prefix(dir).map_err(map_err)?;
            Ok(Built {
                store: Arc::new(lfs),
                signer: None,
            })
        }
        "memory" => Ok(Built {
            store: Arc::new(InMemory::new()),
            signer: None,
        }),
        other => Err(StorageError::Config(format!(
            "unknown storage kind `{other}`"
        ))),
    }
}

pub(crate) async fn put(
    store: &dyn ObjectStore,
    key: &str,
    bytes: Bytes,
    content_type: &str,
) -> Result<(), StorageError> {
    let mut attributes = Attributes::new();
    attributes.insert(
        Attribute::ContentType,
        AttributeValue::from(content_type.to_owned()),
    );
    let options = PutOptions {
        attributes,
        ..Default::default()
    };
    store
        .put_opts(&ObjectPath::from(key), PutPayload::from(bytes), options)
        .await
        .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn get(store: &dyn ObjectStore, key: &str) -> Result<Bytes, StorageError> {
    let result = store.get(&ObjectPath::from(key)).await.map_err(map_err)?;
    result.bytes().await.map_err(map_err)
}

pub(crate) async fn delete(store: &dyn ObjectStore, key: &str) -> Result<(), StorageError> {
    store.delete(&ObjectPath::from(key)).await.map_err(map_err)
}

pub(crate) async fn signed_get_url(
    s3: &AmazonS3,
    key: &str,
    ttl: Duration,
) -> Result<String, StorageError> {
    let url = s3
        .signed_url(http::Method::GET, &ObjectPath::from(key), ttl)
        .await
        .map_err(|err| StorageError::Sign(err.to_string()))?;
    Ok(url.to_string())
}

fn map_err(err: object_store::Error) -> StorageError {
    match err {
        object_store::Error::NotFound { .. } => StorageError::NotFound,
        other => StorageError::Backend(other.to_string()),
    }
}
