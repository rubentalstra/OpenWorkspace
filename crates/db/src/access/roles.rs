//! Idempotent system-role seed.
//!
//! The built-in `owner`/`admin`/`member` permission sets live in `domain`
//! ([`PermissionSet::builtin_owner`] etc.) — the single source of truth. This
//! seeds `roles` + `role_permissions` from them so memberships referencing a
//! system role confer the right actions. Sourcing from the domain sets (not a
//! hard-coded token list) means the two can never drift; a unit test in `auth`
//! pins it.

use domain::authz::PermissionSet;

use crate::{Db, DbError};

/// Upserts the three built-in system roles and synchronizes their permission rows
/// to the current domain builtins. Idempotent: safe to run on every boot.
///
/// # Errors
///
/// [`DbError::Sqlx`] on any database error.
pub async fn seed_system_roles(pool: &Db) -> Result<(), DbError> {
    let roles = [
        ("owner", "Owner", PermissionSet::builtin_owner()),
        ("admin", "Admin", PermissionSet::builtin_admin()),
        ("member", "Member", PermissionSet::builtin_member()),
    ];

    let mut tx = pool.begin().await?;
    for (key, name, perms) in roles {
        let role_id = sqlx::query_scalar!(
            r#"
            INSERT INTO roles (key, name, is_system)
            VALUES ($1::citext, $2, true)
            ON CONFLICT (key) DO UPDATE SET name = EXCLUDED.name, is_system = true
            RETURNING id
            "#,
            key,
            name,
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"DELETE FROM role_permissions WHERE role_id = $1"#,
            role_id,
        )
        .execute(&mut *tx)
        .await?;

        for action in perms.iter() {
            sqlx::query!(
                r#"INSERT INTO role_permissions (role_id, permission) VALUES ($1, $2)"#,
                role_id,
                action.token(),
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}
