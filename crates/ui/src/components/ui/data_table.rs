use crate::clx;
use leptos::prelude::*;

clx! {
    /// Horizontal-scroll wrapper for a data table. Lay a [`Table`](super::Table)
    /// inside so wide column sets scroll within the container instead of forcing
    /// the page body to scroll.
    DataTableContainer, div, "relative w-full overflow-x-auto rounded-md border"
}
clx! {
    /// Summary footer row group for a data table, e.g. column totals. Inverts the
    /// last-row border so the footer reads as a distinct band.
    DataTableFooter, tfoot, "bg-muted/50 border-t font-medium [&>tr]:last:border-b-0"
}
