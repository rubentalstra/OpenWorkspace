use crate::{cn, slot};
use leptos::prelude::*;

/// Table — shadcn Base UI `table`. Wraps the `<table>` in a horizontally
/// scrollable container.
#[component]
pub fn Table(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <div data-slot="table-container" class="cn-table-container">
            <table data-slot="table" class=move || cn!("cn-table", class.get())>
                {children()}
            </table>
        </div>
    }
}

slot! { TableHeader, thead, "table-header", "cn-table-header" }
slot! { TableBody, tbody, "table-body", "cn-table-body" }
slot! { TableFooter, tfoot, "table-footer", "cn-table-footer" }
slot! { TableRow, tr, "table-row", "cn-table-row has-aria-expanded:bg-muted/50" }
slot! { TableHead, th, "table-head", "cn-table-head" }
slot! { TableCell, td, "table-cell", "cn-table-cell" }
slot! { TableCaption, caption, "table-caption", "cn-table-caption" }
