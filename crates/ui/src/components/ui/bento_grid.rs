use crate::clx;
use leptos::prelude::*;

clx! {
    /// Responsive feature-tile grid: four columns from the `md` breakpoint up,
    /// single column below.
    BentoGrid, div, "grid gap-2 md:grid-cols-4"
}
clx! {
    /// Six-tile variant of [`BentoGrid`]: two columns at `sm`, four at `md`.
    BentoGrid6, div, "grid gap-2 sm:grid-cols-2 md:grid-cols-4"
}
clx! {
    /// Tile slot within a bento grid; supplies the minimum height and rounding.
    BentoRow, div, "p-1 min-h-32 rounded-lg"
}
clx! {
    /// Filled tile body that centers its content and fills its slot.
    BentoCell, div, "text-xl rounded-lg size-full flex items-center justify-center bg-zinc-200 dark:bg-zinc-700"
}
