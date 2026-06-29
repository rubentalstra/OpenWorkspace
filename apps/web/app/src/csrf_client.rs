//! The server-fn client that attaches the CSRF token to mutating requests.
//!
//! # Convention for `#[server]` functions
//!
//! The server enforces synchronizer-token CSRF on every unsafe method (see
//! `auth::csrf_layer`). To pass it, annotate the client per the verb:
//!
//! - **mutations** (state-changing) use a `POST` and
//!   `#[server(client = CsrfClient)]` — [`CsrfClient`] reads
//!   `<meta name="csrf-token">` (rendered into `<head>` by the SSR shell) and sets
//!   the `X-CSRF-Token` header on the outgoing request;
//! - **read-only queries** use a real `GET` via `#[server(input = GetUrl)]` —
//!   safe methods are CSRF-exempt by method, so no token is needed.
//!
//! P5 ships no `#[server]` functions yet, so there is nothing to annotate; this
//! type exists so the first mutation can opt in without further plumbing.
//!
//! ## Why this lives in `app`, not `auth`
//!
//! [`CsrfClient`] is the single touchpoint on `server_fn` — the Leptos server-fn
//! machinery that every `#[server]` already uses. The generated client code runs
//! in the browser (wasm), where the `auth` facade (an ssr-only crate over
//! `axum-login`/`tower-sessions`/`sqlx`) cannot and must not be linked. Placing it
//! here, behind the `hydrate` feature, keeps the vendor session stack out of the
//! wasm bundle while still routing the one server-fn header through a named,
//! documented first-party type.
//!
//! The struct is declared **unconditionally** so `#[server(client = CsrfClient)]`
//! name-resolves in both the ssr and hydrate builds; only the `Client` impl and
//! the `<meta>` reader are `hydrate`-gated (the ssr build never sends a
//! client-side request, so it needs no impl).

/// The first-party server-fn [`Client`](server_fn::client::Client): a thin
/// wrapper over `server_fn`'s `BrowserClient` that adds the `X-CSRF-Token` header
/// read from `<meta name="csrf-token">` before sending.
///
/// Use as `#[server(client = CsrfClient)]` on mutating server functions.
pub struct CsrfClient;

/// Server-side `Client` impl. The server never invokes the client (it runs the
/// server-fn body directly), but `ServerFn::Client` is a required bound on every
/// build, so we satisfy it by delegating to `server_fn`'s reqwest client. The
/// browser CSRF behaviour lives in the `hydrate` impl below.
#[cfg(all(feature = "ssr", not(feature = "hydrate")))]
mod imp_ssr {
    use std::future::Future;

    use bytes::Bytes;
    use futures::{Sink, Stream};
    use leptos::server_fn::client::Client;
    use leptos::server_fn::client::reqwest::ReqwestClient;
    use leptos::server_fn::error::FromServerFnError;

    use super::CsrfClient;

    impl<E, InputStreamError, OutputStreamError> Client<E, InputStreamError, OutputStreamError>
        for CsrfClient
    where
        E: FromServerFnError,
        InputStreamError: FromServerFnError,
        OutputStreamError: FromServerFnError,
    {
        type Request = <ReqwestClient as Client<E, InputStreamError, OutputStreamError>>::Request;
        type Response = <ReqwestClient as Client<E, InputStreamError, OutputStreamError>>::Response;

        fn send(req: Self::Request) -> impl Future<Output = Result<Self::Response, E>> + Send {
            <ReqwestClient as Client<E, InputStreamError, OutputStreamError>>::send(req)
        }

        fn open_websocket(
            path: &str,
        ) -> impl Future<
            Output = Result<
                (
                    impl Stream<Item = Result<Bytes, Bytes>> + Send + 'static,
                    impl Sink<Bytes> + Send + 'static,
                ),
                E,
            >,
        > + Send {
            <ReqwestClient as Client<E, InputStreamError, OutputStreamError>>::open_websocket(path)
        }

        fn spawn(future: impl Future<Output = ()> + Send + 'static) {
            <ReqwestClient as Client<E, InputStreamError, OutputStreamError>>::spawn(future);
        }
    }
}

#[cfg(feature = "hydrate")]
mod imp {
    use std::future::Future;

    use bytes::Bytes;
    use futures::{Sink, Stream};
    use server_fn::client::Client;
    use server_fn::client::browser::BrowserClient;
    use server_fn::error::FromServerFnError;
    use server_fn::request::browser::BrowserRequest;
    use server_fn::response::browser::BrowserResponse;
    use wasm_bindgen::JsCast as _;

    use super::CsrfClient;

    /// Header carrying the CSRF token (must match `auth::csrf`'s `CSRF_HEADER`).
    const CSRF_HEADER: &str = "X-CSRF-Token";

    /// Reads the per-page CSRF token from `<meta name="csrf-token">`, if present.
    /// Returns `None` outside a browser context or when the tag is absent (e.g. a
    /// page that was not server-rendered with a token).
    fn meta_token() -> Option<String> {
        let document = web_sys::window()?.document()?;
        let element = document
            .query_selector(r#"meta[name="csrf-token"]"#)
            .ok()??;
        let meta = element.dyn_into::<web_sys::HtmlMetaElement>().ok()?;
        let content = meta.content();
        if content.is_empty() {
            None
        } else {
            Some(content)
        }
    }

    impl<E, InputStreamError, OutputStreamError> Client<E, InputStreamError, OutputStreamError>
        for CsrfClient
    where
        E: FromServerFnError,
        InputStreamError: FromServerFnError,
        OutputStreamError: FromServerFnError,
    {
        type Request = BrowserRequest;
        type Response = BrowserResponse;

        fn send(req: Self::Request) -> impl Future<Output = Result<Self::Response, E>> + Send {
            // Set the header on the underlying request before delegating. The
            // request derefs to gloo's `Request`, whose `headers()` returns the
            // live `web_sys::Headers` backing the request, so `set` mutates the
            // request that `BrowserClient::send` then dispatches.
            if let Some(token) = meta_token() {
                req.headers().set(CSRF_HEADER, &token);
            }
            <BrowserClient as Client<E, InputStreamError, OutputStreamError>>::send(req)
        }

        fn open_websocket(
            path: &str,
        ) -> impl Future<
            Output = Result<
                (
                    impl Stream<Item = Result<Bytes, Bytes>> + Send + 'static,
                    impl Sink<Bytes> + Send + 'static,
                ),
                E,
            >,
        > + Send {
            <BrowserClient as Client<E, InputStreamError, OutputStreamError>>::open_websocket(path)
        }

        fn spawn(future: impl Future<Output = ()> + Send + 'static) {
            <BrowserClient as Client<E, InputStreamError, OutputStreamError>>::spawn(future);
        }
    }
}
