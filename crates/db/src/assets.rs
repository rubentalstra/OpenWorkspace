//! Asset persistence: the `assets` table (uploaded binary objects and their
//! variants). The bytes live in object storage (`crates/storage`); this records
//! the metadata row and the `storage_key` that locates them.

use chrono::{DateTime, Utc};
use domain::AssetId;
use uuid::Uuid;

use crate::{Db, DbError};

/// Persistence-mapped mirror of the `asset_kind` enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "asset_kind", rename_all = "snake_case")]
pub enum AssetKindRow {
    /// Floor-builder reference underlay (tracing aid; never served to end users).
    ReferenceImage,
    /// Campus/site-level map image.
    CampusMap,
    /// Floor background under the SVG scene.
    FloorBackground,
    /// Bookable-resource photo.
    ObjectPhoto,
    /// Instance branding logo.
    Logo,
    /// A generated export (GDPR Article 15); content type is free-form.
    Export,
}

/// A row from `assets`.
#[derive(Clone, Debug)]
pub struct AssetRow {
    pub id: Uuid,
    pub kind: AssetKindRow,
    pub storage_key: String,
    pub content_type: String,
    pub byte_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub checksum: Option<Vec<u8>>,
    pub original_filename: Option<String>,
    pub alt_text: Option<String>,
    pub parent_asset_id: Option<Uuid>,
    pub variant: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// The fields needed to record a new asset (original or variant).
pub struct NewAsset<'a> {
    pub kind: AssetKindRow,
    pub storage_key: &'a str,
    pub content_type: &'a str,
    pub byte_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub checksum: Option<&'a [u8]>,
    pub original_filename: Option<&'a str>,
    pub alt_text: Option<&'a str>,
    pub uploaded_by: Option<Uuid>,
}

/// Inserts a top-level asset (no parent). Returns its id.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error (including the `content_type` CHECK
/// rejecting a non-raster type for a non-export kind).
pub async fn insert_asset(pool: &Db, new: &NewAsset<'_>) -> Result<AssetId, DbError> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO assets
            (kind, storage_key, content_type, byte_size, width, height, checksum,
             original_filename, alt_text, uploaded_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
        new.kind as _,
        new.storage_key,
        new.content_type,
        new.byte_size,
        new.width,
        new.height,
        new.checksum,
        new.original_filename,
        new.alt_text,
        new.uploaded_by,
    )
    .fetch_one(pool)
    .await?;
    Ok(AssetId::new(id))
}

/// Inserts a derived variant (e.g. a thumbnail) of `parent`, tagged `variant`.
/// Returns its id. Deleting the parent cascades to its variants.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn insert_variant(
    pool: &Db,
    parent: AssetId,
    new: &NewAsset<'_>,
    variant: &str,
) -> Result<AssetId, DbError> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO assets
            (kind, storage_key, content_type, byte_size, width, height, checksum,
             parent_asset_id, variant, uploaded_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
        new.kind as _,
        new.storage_key,
        new.content_type,
        new.byte_size,
        new.width,
        new.height,
        new.checksum,
        parent.as_uuid(),
        variant,
        new.uploaded_by,
    )
    .fetch_one(pool)
    .await?;
    Ok(AssetId::new(id))
}

/// Loads an asset by id. Returns `None` if absent.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_asset(pool: &Db, id: AssetId) -> Result<Option<AssetRow>, DbError> {
    let row = sqlx::query_as!(
        AssetRow,
        r#"
        SELECT id, kind AS "kind: AssetKindRow", storage_key, content_type, byte_size,
               width, height, checksum, original_filename, alt_text, parent_asset_id,
               variant, uploaded_by, created_at
        FROM assets WHERE id = $1
        "#,
        id.as_uuid(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Loads a named variant of a parent asset (e.g. the `thumb`). Returns `None` if
/// absent.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn load_variant(
    pool: &Db,
    parent: AssetId,
    variant: &str,
) -> Result<Option<AssetRow>, DbError> {
    let row = sqlx::query_as!(
        AssetRow,
        r#"
        SELECT id, kind AS "kind: AssetKindRow", storage_key, content_type, byte_size,
               width, height, checksum, original_filename, alt_text, parent_asset_id,
               variant, uploaded_by, created_at
        FROM assets WHERE parent_asset_id = $1 AND variant = $2
        "#,
        parent.as_uuid(),
        variant,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Deletes an asset and (via `ON DELETE CASCADE`) its variants.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn delete_asset(pool: &Db, id: AssetId) -> Result<(), DbError> {
    sqlx::query!(r#"DELETE FROM assets WHERE id = $1"#, id.as_uuid())
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn logo<'a>(key: &'a str, content_type: &'a str) -> NewAsset<'a> {
        NewAsset {
            kind: AssetKindRow::Logo,
            storage_key: key,
            content_type,
            byte_size: 123,
            width: Some(64),
            height: Some(64),
            checksum: Some(&[7u8; 32]),
            original_filename: Some("logo.png"),
            alt_text: None,
            uploaded_by: None,
        }
    }

    #[sqlx::test]
    async fn insert_and_load_round_trip(pool: Db) {
        let id = insert_asset(&pool, &logo("assets/a", "image/png"))
            .await
            .unwrap();
        let row = load_asset(&pool, id).await.unwrap().unwrap();
        assert_eq!(row.id, id.as_uuid());
        assert_eq!(row.kind, AssetKindRow::Logo);
        assert_eq!(row.content_type, "image/png");
        assert_eq!(row.byte_size, 123);
        assert_eq!(row.width, Some(64));
        assert_eq!(row.checksum.as_deref(), Some(&[7u8; 32][..]));
        assert!(row.parent_asset_id.is_none());
    }

    #[sqlx::test]
    async fn variant_loads_and_cascades_on_parent_delete(pool: Db) {
        let parent = insert_asset(&pool, &logo("assets/p", "image/png"))
            .await
            .unwrap();
        let thumb = NewAsset {
            storage_key: "assets/p/thumb",
            content_type: "image/webp",
            byte_size: 10,
            width: Some(32),
            height: Some(32),
            ..logo("assets/p/thumb", "image/webp")
        };
        let vid = insert_variant(&pool, parent, &thumb, "thumb")
            .await
            .unwrap();

        let loaded = load_variant(&pool, parent, "thumb").await.unwrap().unwrap();
        assert_eq!(loaded.id, vid.as_uuid());
        assert_eq!(loaded.parent_asset_id, Some(parent.as_uuid()));
        assert_eq!(loaded.variant.as_deref(), Some("thumb"));

        delete_asset(&pool, parent).await.unwrap();
        assert!(load_asset(&pool, parent).await.unwrap().is_none());
        assert!(
            load_variant(&pool, parent, "thumb")
                .await
                .unwrap()
                .is_none(),
            "deleting the parent must cascade to its variants"
        );
    }

    #[sqlx::test]
    async fn content_type_check_rejects_non_raster_image(pool: Db) {
        assert!(
            insert_asset(&pool, &logo("assets/svg", "image/svg+xml"))
                .await
                .is_err(),
            "the CHECK must reject a non-raster type for an image kind"
        );
        let export = NewAsset {
            kind: AssetKindRow::Export,
            content_type: "application/json",
            ..logo("assets/exp", "application/json")
        };
        assert!(
            insert_asset(&pool, &export).await.is_ok(),
            "the export kind is exempt from the raster constraint"
        );
    }
}
