//! The catalog registry: a static table of `CatalogEntry` (metadata + render fn).
//!
//! This is the single extension point. A new component is one render fn (in a
//! category module) plus one `entry!(…)` line in [`CATALOG`] plus its
//! [`CatalogKind`] variant — the completeness test keeps the enum and the table in
//! lockstep. The same table feeds both this renderer (`render_node`) and the P11
//! builder palette (`entries`/`by_category`).

use domain::SpaceState;
use leptos::prelude::*;

use crate::catalog::{annotation, bookable, structure, wayfinding, zoning};
use crate::model::{CatalogKind, SceneNode, SceneNodeId};

/// Palette grouping for a catalog component.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    Structure,
    Bookable,
    Furniture,
    Functional,
    Zoning,
    Wayfinding,
    Annotation,
    Overlay,
}

/// Metadata describing a catalog component (what the P11 palette renders).
#[derive(Clone, Copy, Debug)]
pub struct CatalogMeta {
    pub kind: CatalogKind,
    pub category: Category,
    pub label: &'static str,
    pub bookable: bool,
    pub icon: icondata::Icon,
    /// Default footprint `(width, height)` the builder places (scene units).
    pub default_size: (f64, f64),
}

/// Everything a component's render fn needs. A struct (not loose args) so future
/// render inputs (theme, locale) are added in one place.
pub struct RenderCtx<'a> {
    pub node: &'a SceneNode,
    pub state: Signal<SpaceState>,
    pub on_select: Option<Callback<SceneNodeId>>,
}

/// A registry row: metadata + the render fn producing the node's inline SVG.
pub struct CatalogEntry {
    pub meta: CatalogMeta,
    pub render: fn(&RenderCtx<'_>) -> AnyView,
}

/// Builds a [`CatalogEntry`]; `bookable` is derived from the kind so it can't drift.
macro_rules! entry {
    ($kind:expr, $cat:expr, $label:literal, $icon:expr, $w:literal, $h:literal, $render:path) => {
        CatalogEntry {
            meta: CatalogMeta {
                kind: $kind,
                category: $cat,
                label: $label,
                bookable: $kind.bookable(),
                icon: $icon,
                default_size: ($w, $h),
            },
            render: $render,
        }
    };
}

/// The registry. Add a component here (+ its render fn + `CatalogKind` variant).
static CATALOG: &[CatalogEntry] = &[
    // Structure and shell.
    entry!(
        CatalogKind::Wall,
        Category::Structure,
        "Wall",
        icondata::LuMinus,
        0.0,
        0.0,
        structure::wall
    ),
    entry!(
        CatalogKind::Door,
        Category::Structure,
        "Door",
        icondata::LuDoorOpen,
        0.0,
        0.0,
        structure::door
    ),
    entry!(
        CatalogKind::Window,
        Category::Structure,
        "Window",
        icondata::LuSquare,
        0.0,
        0.0,
        structure::window
    ),
    entry!(
        CatalogKind::Column,
        Category::Structure,
        "Column",
        icondata::LuCircle,
        2.0,
        2.0,
        structure::column
    ),
    entry!(
        CatalogKind::RoomEnclosure,
        Category::Structure,
        "Room",
        icondata::LuSquare,
        0.0,
        0.0,
        structure::room_enclosure
    ),
    // Bookable resources.
    entry!(
        CatalogKind::Desk,
        Category::Bookable,
        "Desk",
        icondata::LuSquare,
        8.0,
        6.0,
        bookable::desk
    ),
    entry!(
        CatalogKind::DeskBench,
        Category::Bookable,
        "Bench desk",
        icondata::LuSquare,
        16.0,
        6.0,
        bookable::desk_bench
    ),
    entry!(
        CatalogKind::MeetingRoom,
        Category::Bookable,
        "Meeting room",
        icondata::LuUsers,
        0.0,
        0.0,
        bookable::meeting_room
    ),
    entry!(
        CatalogKind::ParkingSpace,
        Category::Bookable,
        "Parking space",
        icondata::LuSquare,
        5.0,
        10.0,
        bookable::parking_space
    ),
    // Zoning.
    entry!(
        CatalogKind::Zone,
        Category::Zoning,
        "Zone",
        icondata::LuGroup,
        0.0,
        0.0,
        zoning::zone
    ),
    // Annotation.
    entry!(
        CatalogKind::Label,
        Category::Annotation,
        "Label",
        icondata::LuType,
        0.0,
        0.0,
        annotation::label
    ),
    // Wayfinding.
    entry!(
        CatalogKind::Entrance,
        Category::Wayfinding,
        "Entrance",
        icondata::LuLogIn,
        4.0,
        4.0,
        wayfinding::entrance
    ),
    entry!(
        CatalogKind::Exit,
        Category::Wayfinding,
        "Exit",
        icondata::LuLogOut,
        4.0,
        4.0,
        wayfinding::exit
    ),
    entry!(
        CatalogKind::Amenity,
        Category::Wayfinding,
        "Amenity",
        icondata::LuMapPin,
        4.0,
        4.0,
        wayfinding::amenity
    ),
];

/// The registry entry for `kind`, if registered.
#[must_use]
pub fn lookup(kind: CatalogKind) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|entry| entry.meta.kind == kind)
}

/// All registered entries (for the P11 palette).
#[must_use]
pub fn entries() -> &'static [CatalogEntry] {
    CATALOG
}

/// Registered entries in a category (for grouped palette sections).
pub fn by_category(category: Category) -> impl Iterator<Item = &'static CatalogEntry> {
    CATALOG
        .iter()
        .filter(move |entry| entry.meta.category == category)
}

/// Renders one node to inline SVG via its registered component. An unregistered or
/// `Unknown` kind renders a neutral placeholder (fail-soft).
#[must_use]
pub fn render_node(
    node: &SceneNode,
    state: Signal<SpaceState>,
    on_select: Option<Callback<SceneNodeId>>,
) -> AnyView {
    let ctx = RenderCtx {
        node,
        state,
        on_select,
    };
    match lookup(node.kind) {
        Some(entry) => (entry.render)(&ctx),
        None => placeholder(&ctx),
    }
}

fn placeholder(ctx: &RenderCtx<'_>) -> AnyView {
    let transform = ctx.node.transform.to_attr();
    let (x, y) = super::geometry::anchor(&ctx.node.geometry).unwrap_or((0.0, 0.0));
    view! {
        <g
            data-slot="floor-node"
            data-kind="unknown"
            class="cn-floor-node cn-floor-unknown"
            transform=transform
        >
            <rect x=x - 2.0 y=y - 2.0 width="4" height="4" />
        </g>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator as _;

    #[test]
    fn every_kind_except_unknown_is_registered_exactly_once() {
        for kind in CatalogKind::iter() {
            if kind == CatalogKind::Unknown {
                assert!(lookup(kind).is_none(), "Unknown must not be registered");
                continue;
            }
            let hits = CATALOG.iter().filter(|e| e.meta.kind == kind).count();
            assert_eq!(hits, 1, "{kind:?} must have exactly one registry entry");
        }
    }

    #[test]
    fn registry_meta_is_consistent() {
        for entry in CATALOG {
            // `bookable` flag matches the kind's own classification.
            assert_eq!(entry.meta.bookable, entry.meta.kind.bookable());
            assert!(!entry.meta.label.is_empty());
        }
    }

    #[test]
    fn by_category_groups_entries() {
        assert!(by_category(Category::Bookable).all(|e| e.meta.bookable));
        assert!(by_category(Category::Bookable).count() >= 1);
    }
}
