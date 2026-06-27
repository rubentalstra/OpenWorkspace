use crate::components::ui::data_grid::DataGridColumn;
use leptos::prelude::*;

/// Shared in-place cell-edit state for a data grid, distributed via Leptos
/// context so a cell and its editor can coordinate without prop drilling.
///
/// Tracks which cell (if any) is open for editing and the live value of its
/// editor. `Copy` so call sites can capture it freely into closures.
#[derive(Clone, Copy, Debug)]
pub struct CellEditContext<C: DataGridColumn> {
    editing_cell: RwSignal<Option<(usize, C)>>,
    /// Live value bound to the active editor; the input writes each keystroke here.
    pub edit_value: RwSignal<String>,
}

impl<C: DataGridColumn> CellEditContext<C> {
    /// Returns whether the cell at `row_idx`/`col` is the one currently open for editing.
    pub fn is_editing(&self, row_idx: usize, col: C) -> bool {
        self.editing_cell.get() == Some((row_idx, col))
    }

    /// Opens the cell at `row_idx`/`col` for editing, seeding the editor with `initial_value`.
    pub fn start_edit(&self, row_idx: usize, col: C, initial_value: String) {
        self.editing_cell.set(Some((row_idx, col)));
        self.edit_value.set(initial_value);
    }

    /// Closes the active editor, discarding any pending value.
    pub fn cancel_edit(&self) {
        self.editing_cell.set(None);
        self.edit_value.set(String::new());
    }

    /// Closes the active editor and returns the edited `(row_idx, col, value)`,
    /// or `None` if no cell was being edited.
    pub fn finish_edit(&self) -> Option<(usize, C, String)> {
        let (row_idx, col) = self.editing_cell.get()?;
        let value = self.edit_value.get();
        self.editing_cell.set(None);
        self.edit_value.set(String::new());
        Some((row_idx, col, value))
    }
}

/// Creates a [`CellEditContext`], provides it to descendants, and returns it.
///
/// Call once at the grid root so cells and editors resolve the same instance.
pub fn use_cell_edit<C: DataGridColumn>() -> CellEditContext<C> {
    let ctx = CellEditContext {
        editing_cell: RwSignal::new(None),
        edit_value: RwSignal::new(String::new()),
    };
    provide_context(ctx);
    ctx
}
