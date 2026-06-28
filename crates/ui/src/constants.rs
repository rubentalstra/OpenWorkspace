//! Shared layout constants for the kit's data components.

/// Layout metrics for paginated and virtualized data grids.
pub(crate) struct Pagination;

impl Pagination {
    /// Fixed row height, in pixels, that virtual scrolling uses to map a scroll
    /// offset onto row indices.
    pub(crate) const ROW_HEIGHT: usize = 48;
}
