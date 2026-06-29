//! The interactive floor builder: pure scene-edit operations ([`ops`]), an editable
//! builder state, and the `FloorBuilder` editor component. Reuses the catalog and
//! the read-only renderer's viewport math; carries no `db`/`auth` dependency (the
//! app wires persistence + authorization around it).

pub mod editor;
pub mod ops;

pub use editor::{FloorBuilder, Tool};
