//! Asset upload + presigned-serve endpoints.
//!
//! `POST /api/assets` accepts a multipart upload (a `kind`, an optional `target`
//! id, the `file`, and optional `alt_text`), authorizes it per kind against the P8
//! `AuthzBackend`, re-encodes + thumbnails it via the storage pipeline (in a
//! blocking task), stores both objects, and records the asset rows. CSRF is
//! enforced by the surrounding `auth::csrf_layer` (the client sends `X-CSRF-Token`).
//!
//! `GET /api/assets/{id}` (optionally `?variant=thumb`) authorizes the caller and
//! 302-redirects to a short-lived presigned URL — assets are never served from a
//! public path.

use axum::Router;
use axum::extract::{DefaultBodyLimit, Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};

use auth::{AuthSession, AuthzBackend, AuthzError, Target};
use db::{AssetKindRow, Db, NewAsset};
use domain::authz::Action;
use domain::{AssetId, LocationId, ResourceId};
use storage::{ImageLimits, Storage};
use uuid::Uuid;

use crate::AppState;

/// Registers the asset endpoints. `max_upload_bytes` caps the request body so an
/// oversized upload is rejected before it is buffered.
pub(crate) fn routes(max_upload_bytes: u64) -> Router<AppState> {
    let limit = usize::try_from(max_upload_bytes).unwrap_or(usize::MAX);
    Router::new()
        .route(
            "/api/assets",
            post(upload_handler).layer(DefaultBodyLimit::max(limit)),
        )
        .route("/api/assets/{id}", get(serve_handler))
}

#[derive(serde::Serialize)]
struct UploadResponse {
    asset_id: Uuid,
}

#[derive(serde::Deserialize)]
struct ServeQuery {
    variant: Option<String>,
}

#[derive(Debug, thiserror::Error)]
enum UploadError {
    #[error("authentication required")]
    Unauthenticated,
    #[error("asset not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(&'static str),
    #[error(transparent)]
    Authz(#[from] AuthzError),
    #[error(transparent)]
    Storage(#[from] storage::StorageError),
    #[error(transparent)]
    Db(#[from] db::DbError),
    #[error("internal error")]
    Internal,
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        tracing::warn!(error = %self, "asset endpoint error");
        match self {
            Self::Authz(err) => err.into_response(),
            Self::Unauthenticated => StatusCode::UNAUTHORIZED.into_response(),
            Self::NotFound => StatusCode::NOT_FOUND.into_response(),
            Self::BadRequest(_) => StatusCode::BAD_REQUEST.into_response(),
            Self::Storage(storage::StorageError::UnsupportedFormat) => {
                StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response()
            }
            Self::Storage(storage::StorageError::TooLarge) => {
                StatusCode::PAYLOAD_TOO_LARGE.into_response()
            }
            Self::Storage(_) | Self::Db(_) | Self::Internal => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

/// The parsed multipart fields of an upload.
#[derive(Default)]
struct UploadForm {
    kind: Option<String>,
    target: Option<String>,
    alt_text: Option<String>,
    filename: Option<String>,
    file: Option<axum::body::Bytes>,
}

async fn parse_form(mut multipart: Multipart) -> Result<UploadForm, UploadError> {
    let mut form = UploadForm::default();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| UploadError::BadRequest("malformed multipart body"))?
    {
        let name = field.name().map(str::to_owned);
        match name.as_deref() {
            Some("kind") => form.kind = Some(field_text(field).await?),
            Some("target") => form.target = Some(field_text(field).await?),
            Some("alt_text") => form.alt_text = Some(field_text(field).await?),
            Some("file") => {
                form.filename = field.file_name().map(str::to_owned);
                form.file = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| UploadError::BadRequest("could not read file field"))?,
                );
            }
            _ => {
                // Drain any unknown field so the multipart stream advances.
                field.bytes().await.ok();
            }
        }
    }
    Ok(form)
}

async fn field_text(field: axum::extract::multipart::Field<'_>) -> Result<String, UploadError> {
    field
        .text()
        .await
        .map_err(|_| UploadError::BadRequest("invalid text field"))
}

/// Resolves an upload `kind` (and optional `target` id) to the asset kind plus the
/// authorization action/target gating the upload. `export` is system-only.
fn resolve_kind(
    kind: &str,
    target: Option<&str>,
) -> Result<(AssetKindRow, Action, Target), UploadError> {
    let location = || -> Result<LocationId, UploadError> {
        let raw = target.ok_or(UploadError::BadRequest("missing target location"))?;
        let id = Uuid::parse_str(raw).map_err(|_| UploadError::BadRequest("invalid target id"))?;
        Ok(LocationId::new(id))
    };
    Ok(match kind {
        "logo" => (
            AssetKindRow::Logo,
            Action::InstanceConfigure,
            Target::Instance,
        ),
        "campus_map" => (
            AssetKindRow::CampusMap,
            Action::HierarchyEdit,
            Target::Location(location()?),
        ),
        "floor_background" => (
            AssetKindRow::FloorBackground,
            Action::FloorBuild,
            Target::Location(location()?),
        ),
        "reference_image" => (
            AssetKindRow::ReferenceImage,
            Action::FloorBuild,
            Target::Location(location()?),
        ),
        "object_photo" => {
            let raw = target.ok_or(UploadError::BadRequest("missing target resource"))?;
            let id =
                Uuid::parse_str(raw).map_err(|_| UploadError::BadRequest("invalid target id"))?;
            (
                AssetKindRow::ObjectPhoto,
                Action::ResourceManage,
                Target::Resource(ResourceId::new(id)),
            )
        }
        "export" => {
            return Err(UploadError::BadRequest(
                "export assets are system-generated",
            ));
        }
        _ => return Err(UploadError::BadRequest("unknown asset kind")),
    })
}

async fn upload_handler(
    State(db): State<Db>,
    State(authz): State<AuthzBackend>,
    State(storage): State<Storage>,
    State(limits): State<ImageLimits>,
    auth_session: AuthSession,
    multipart: Multipart,
) -> Result<axum::Json<UploadResponse>, UploadError> {
    let actor = auth_session
        .user
        .map(|user| user.id)
        .ok_or(UploadError::Unauthenticated)?;

    let form = parse_form(multipart).await?;
    let kind = form.kind.ok_or(UploadError::BadRequest("missing kind"))?;
    let (asset_kind, action, target) = resolve_kind(&kind, form.target.as_deref())?;
    authz.authorize(actor, action, target, None).await?;

    let raw = form.file.ok_or(UploadError::BadRequest("missing file"))?;
    let processed = tokio::task::spawn_blocking(move || storage::process_upload(&raw, limits))
        .await
        .map_err(|_| UploadError::Internal)?
        .map_err(UploadError::Storage)?;

    // Capture metadata before the byte buffers are moved into the store.
    let content_type = processed.kind.content_type();
    let thumbnail_content_type = processed.thumbnail_content_type();
    let checksum = processed.checksum;
    let original_size = i64::try_from(processed.original.len()).unwrap_or(i64::MAX);
    let thumbnail_size = i64::try_from(processed.thumbnail.len()).unwrap_or(i64::MAX);
    let width = i32::try_from(processed.width).ok();
    let height = i32::try_from(processed.height).ok();
    let thumbnail_width = i32::try_from(processed.thumbnail_width).ok();
    let thumbnail_height = i32::try_from(processed.thumbnail_height).ok();

    let key = storage::new_storage_key();
    let thumbnail_key = storage::thumbnail_key(&key);
    storage.put(&key, processed.original, content_type).await?;
    storage
        .put(&thumbnail_key, processed.thumbnail, thumbnail_content_type)
        .await?;

    let asset_id = db::insert_asset(
        &db,
        &NewAsset {
            kind: asset_kind,
            storage_key: &key,
            content_type,
            byte_size: original_size,
            width,
            height,
            checksum: Some(&checksum),
            original_filename: form.filename.as_deref(),
            alt_text: form.alt_text.as_deref(),
            uploaded_by: Some(actor.as_uuid()),
        },
    )
    .await?;
    db::insert_variant(
        &db,
        asset_id,
        &NewAsset {
            kind: asset_kind,
            storage_key: &thumbnail_key,
            content_type: thumbnail_content_type,
            byte_size: thumbnail_size,
            width: thumbnail_width,
            height: thumbnail_height,
            checksum: None,
            original_filename: None,
            alt_text: None,
            uploaded_by: Some(actor.as_uuid()),
        },
        "thumb",
    )
    .await?;

    Ok(axum::Json(UploadResponse {
        asset_id: asset_id.as_uuid(),
    }))
}

async fn serve_handler(
    State(db): State<Db>,
    State(storage): State<Storage>,
    auth_session: AuthSession,
    Path(id): Path<Uuid>,
    Query(query): Query<ServeQuery>,
) -> Result<Redirect, UploadError> {
    if auth_session.user.is_none() {
        return Err(UploadError::Unauthenticated);
    }
    let asset_id = AssetId::new(id);
    let key = if query.variant.as_deref() == Some("thumb") {
        db::load_variant(&db, asset_id, "thumb")
            .await?
            .ok_or(UploadError::NotFound)?
            .storage_key
    } else {
        db::load_asset(&db, asset_id)
            .await?
            .ok_or(UploadError::NotFound)?
            .storage_key
    };
    let url = storage.signed_get_url(&key).await?;
    Ok(Redirect::temporary(&url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_maps_to_action_and_target() {
        let (kind, action, target) = resolve_kind("logo", None).unwrap();
        assert_eq!(kind, AssetKindRow::Logo);
        assert_eq!(action, Action::InstanceConfigure);
        assert_eq!(target, Target::Instance);

        let id = Uuid::now_v7();
        let (kind, action, target) = resolve_kind("object_photo", Some(&id.to_string())).unwrap();
        assert_eq!(kind, AssetKindRow::ObjectPhoto);
        assert_eq!(action, Action::ResourceManage);
        assert_eq!(target, Target::Resource(ResourceId::new(id)));

        let (_, action, target) = resolve_kind("floor_background", Some(&id.to_string())).unwrap();
        assert_eq!(action, Action::FloorBuild);
        assert_eq!(target, Target::Location(LocationId::new(id)));
    }

    #[test]
    fn rejects_export_unknown_and_missing_or_bad_target() {
        assert!(matches!(
            resolve_kind("export", None),
            Err(UploadError::BadRequest(_))
        ));
        assert!(matches!(
            resolve_kind("nope", None),
            Err(UploadError::BadRequest(_))
        ));
        // location-scoped kinds require a target id...
        assert!(matches!(
            resolve_kind("campus_map", None),
            Err(UploadError::BadRequest(_))
        ));
        // ...and it must be a valid uuid.
        assert!(matches!(
            resolve_kind("object_photo", Some("not-a-uuid")),
            Err(UploadError::BadRequest(_))
        ));
    }
}
