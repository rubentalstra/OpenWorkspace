use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::utils::query::{Query, QueryUtils};

const FIRST_PAGE: u32 = 1;

/// Reactive pagination derived from the `page` URL query parameter.
#[derive(Clone, Copy)]
pub struct PaginationContext {
    /// Current 1-based page, defaulting to the first page when absent or invalid.
    pub current_page: Memo<u32>,
    /// Builds an href for `page`, preserving every other query parameter.
    pub page_href: Callback<u32, String>,
    /// Href for the previous page, or `#` when already on the first page.
    pub prev_href: Signal<String>,
    /// Href for the next page.
    pub next_href: Signal<String>,
    /// Whether the current page is the first.
    pub is_first_page: Signal<bool>,
    /// `"page"` for the active page (for `aria-current`), `""` otherwise.
    pub aria_current: Callback<u32, &'static str>,
}

/// Reads the current page from the URL and builds page hrefs that change only
/// the `page` parameter, leaving any other query parameters intact.
pub fn use_pagination() -> PaginationContext {
    let location = use_location();
    let current_page_str = QueryUtils::extract(Query::PAGE.to_string());

    let current_page = Memo::new(move |_| {
        current_page_str
            .get()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(FIRST_PAGE)
    });

    let page_href = Callback::new(move |page: u32| {
        location.query.with(|q| {
            let mut params: Vec<String> = q
                .to_query_string()
                .trim_start_matches('?')
                .split('&')
                .filter(|pair| !pair.is_empty() && !pair.starts_with(&format!("{}=", Query::PAGE)))
                .map(ToString::to_string)
                .collect();
            params.push(format!("{}={page}", Query::PAGE));
            format!("?{}", params.join("&"))
        })
    });

    let prev_href = Signal::derive(move || {
        let current = current_page.get();
        if current > FIRST_PAGE {
            page_href.run(current - 1)
        } else {
            "#".to_string()
        }
    });

    let next_href = Signal::derive(move || page_href.run(current_page.get() + 1));

    let is_first_page = Signal::derive(move || current_page.get() <= FIRST_PAGE);

    let aria_current = Callback::new(move |page: u32| {
        if current_page.get() == page {
            Query::PAGE
        } else {
            ""
        }
    });

    PaginationContext {
        current_page,
        page_href,
        prev_href,
        next_href,
        is_first_page,
        aria_current,
    }
}
