use anyhow::Context;
use app::{App, shell};
use axum::Router;
use axum::routing::get;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conf = get_configuration(None).context("loading Leptos configuration")?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let router = Router::new()
        .route("/health", get(|| async { "ok" }))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    log!("listening on http://{addr}");
    axum::serve(listener, router.into_make_service())
        .await
        .context("serving HTTP")?;
    Ok(())
}
