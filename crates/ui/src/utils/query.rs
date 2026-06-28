//! URL query-parameter helpers backing the date pickers and pagination, built on
//! `leptos_router`'s reactive query map.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsValue;
use leptos_router::hooks::use_query_map;

/// Query-parameter keys the data components read from, and write to, the URL.
pub(crate) struct Query;

impl Query {
    pub(crate) const PAGE: &'static str = "page";
    pub(crate) const START_DATE: &'static str = "start_date";
    pub(crate) const END_DATE: &'static str = "end_date";
}

/// Stateless helpers for reading and updating URL query parameters.
pub(crate) struct QueryUtils;

impl QueryUtils {
    /// A reactive view of the named query parameter; `None` while it is absent.
    pub(crate) fn extract(key: String) -> Signal<Option<String>> {
        let query = use_query_map();
        Signal::derive(move || query.read().get(&key))
    }

    /// Replaces the `start_date`/`end_date` query parameters in the address bar
    /// without adding a history entry (`Some` sets, `None` removes). Client-only;
    /// a no-op when there is no `window`.
    pub(crate) fn update_dates_url(start: Option<String>, end: Option<String>) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let search = window.location().search().unwrap_or_default();
        let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) else {
            return;
        };

        match start {
            Some(value) => params.set(Query::START_DATE, &value),
            None => params.delete(Query::START_DATE),
        }
        match end {
            Some(value) => params.set(Query::END_DATE, &value),
            None => params.delete(Query::END_DATE),
        }

        let query = String::from(params.to_string());
        let path = window.location().pathname().unwrap_or_default();
        let url = if query.is_empty() {
            path
        } else {
            format!("{path}?{query}")
        };

        if let Ok(history) = window.history() {
            _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&url));
        }
    }
}
