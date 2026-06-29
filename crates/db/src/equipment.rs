//! The equipment catalog (`equipment_items`): simple named items the builder assigns
//! to resources with a quantity (see `resource_equipment`).

use domain::EquipmentItemId;
use uuid::Uuid;

use crate::{Db, DbError, classify};

/// A catalog equipment item.
#[derive(Clone, Debug)]
pub struct EquipmentItem {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

/// All catalog items, by name.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn list_equipment_items(pool: &Db) -> Result<Vec<EquipmentItem>, DbError> {
    let rows = sqlx::query!(
        r#"SELECT id, name::text AS "name!", description FROM equipment_items ORDER BY name"#,
    )
    .fetch_all(pool)
    .await
    .map_err(classify)?;
    Ok(rows
        .into_iter()
        .map(|r| EquipmentItem {
            id: r.id,
            name: r.name,
            description: r.description,
        })
        .collect())
}

/// Adds a catalog item. The name is unique (case-insensitive).
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error (incl. a duplicate name).
pub async fn create_equipment_item(
    pool: &Db,
    name: &str,
    description: Option<&str>,
) -> Result<EquipmentItemId, DbError> {
    let id = sqlx::query_scalar!(
        r#"INSERT INTO equipment_items (name, description) VALUES ($1, $2) RETURNING id"#,
        name,
        description,
    )
    .fetch_one(pool)
    .await
    .map_err(classify)?;
    Ok(EquipmentItemId::new(id))
}

/// Renames / re-describes a catalog item.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn update_equipment_item(
    pool: &Db,
    id: EquipmentItemId,
    name: &str,
    description: Option<&str>,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE equipment_items SET name = $2, description = $3 WHERE id = $1"#,
        id.as_uuid(),
        name,
        description,
    )
    .execute(pool)
    .await
    .map_err(classify)?;
    Ok(())
}

/// Deletes a catalog item. Fails (FK `RESTRICT`) if any resource still has it.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error (incl. the item still in use).
pub async fn delete_equipment_item(pool: &Db, id: EquipmentItemId) -> Result<(), DbError> {
    sqlx::query!(r#"DELETE FROM equipment_items WHERE id = $1"#, id.as_uuid())
        .execute(pool)
        .await
        .map_err(classify)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    #[sqlx::test]
    async fn create_list_update_delete(pool: Db) {
        let id = create_equipment_item(&pool, "Dock", Some("USB-C"))
            .await
            .unwrap();
        let items = list_equipment_items(&pool).await.unwrap();
        assert!(
            items
                .iter()
                .any(|i| i.id == id.as_uuid() && i.name == "Dock")
        );

        update_equipment_item(&pool, id, "Dock Gen 2", None)
            .await
            .unwrap();
        let items = list_equipment_items(&pool).await.unwrap();
        assert!(items.iter().any(|i| i.name == "Dock Gen 2"));

        delete_equipment_item(&pool, id).await.unwrap();
        assert!(list_equipment_items(&pool).await.unwrap().is_empty());
    }

    #[sqlx::test]
    async fn duplicate_name_is_rejected(pool: Db) {
        create_equipment_item(&pool, "Monitor", None).await.unwrap();
        // Case-insensitive unique (citext).
        assert!(create_equipment_item(&pool, "monitor", None).await.is_err());
    }
}
