//! Scene schema versioning + the forward-migration framework.
//!
//! `floor_plans.scene_schema_version` records the version a scene was written at.
//! [`load_scene`] runs the forward-migration chain (`v_n → … → current`) on the raw
//! JSON before the final typed deserialize, so a scene written by ANY past release
//! always loads. Adding a future format change is one migration step (in [`step`]),
//! a bump of [`CURRENT_SCENE_VERSION`], and a fixture test — "scene migrations are
//! mechanical" (schema comment m12).

use serde_json::Value;

use crate::model::error::SceneError;
use crate::model::scene::Scene;

/// The scene schema version this build writes and understands.
pub const CURRENT_SCENE_VERSION: u32 = 1;

/// Loads a scene from its stored JSON + schema version: runs the migration chain,
/// deserializes, stamps the current version, and validates.
///
/// # Errors
///
/// [`SceneError::FutureVersion`] if `version` exceeds [`CURRENT_SCENE_VERSION`];
/// [`SceneError::Deserialize`] if the (migrated) JSON does not match the scene
/// shape; or a validation error from [`Scene::validate`].
pub fn load_scene(raw: Value, version: u32) -> Result<Scene, SceneError> {
    if version > CURRENT_SCENE_VERSION {
        return Err(SceneError::FutureVersion {
            found: version,
            supported: CURRENT_SCENE_VERSION,
        });
    }
    let migrated = apply_migrations(raw, version);
    let mut scene: Scene =
        serde_json::from_value(migrated).map_err(|err| SceneError::Deserialize(err.to_string()))?;
    scene.schema_version = CURRENT_SCENE_VERSION;
    scene.validate()?;
    Ok(scene)
}

/// Applies forward steps from `from` up to [`CURRENT_SCENE_VERSION`].
fn apply_migrations(mut value: Value, from: u32) -> Value {
    let mut version = from;
    while version < CURRENT_SCENE_VERSION {
        value = step(value, version);
        version += 1;
    }
    value
}

/// One forward step `from → from + 1`. No steps exist yet (current == 1). Add an
/// arm per schema bump, e.g.:
/// ```ignore
/// match from {
///     1 => migrate_v1_to_v2(value),
///     _ => value,
/// }
/// ```
fn step(value: Value, _from: u32) -> Value {
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_object_loads_as_empty_scene() {
        let scene = load_scene(json!({}), CURRENT_SCENE_VERSION).unwrap();
        assert!(scene.nodes.is_empty());
        assert_eq!(scene.schema_version, CURRENT_SCENE_VERSION);
    }

    #[test]
    fn future_version_is_rejected() {
        let err = load_scene(json!({}), CURRENT_SCENE_VERSION + 1).unwrap_err();
        assert!(matches!(err, SceneError::FutureVersion { .. }));
    }

    #[test]
    fn current_version_is_identity() {
        // With no migration steps, the chain is a no-op at the current version.
        let value = json!({ "view_box": { "min_x": 0, "min_y": 0, "width": 10, "height": 10 } });
        assert_eq!(
            apply_migrations(value.clone(), CURRENT_SCENE_VERSION),
            value
        );
    }
}
