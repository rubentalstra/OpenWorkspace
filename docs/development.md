# Development setup

The Rust toolchain is pinned by `rust-toolchain.toml` (1.96.0, edition 2024,
`wasm32-unknown-unknown`, rustfmt + clippy) — rustup installs it automatically on
the first `cargo` run.

## Required CLIs

Install once:

```sh
cargo install cargo-leptos --locked      # build/watch the Leptos app (server + wasm + CSS)
cargo install leptosfmt                   # format the view! macro
cargo install sqlx-cli --version ^0.9 --no-default-features --features rustls,postgres
cargo install cargo-nextest --locked      # test runner
cargo install cargo-deny                  # licence / advisory / ban policy
cargo install cargo-audit                 # RustSec vulnerability scan
cargo install cargo-chef --locked         # cached container builds (deploy/containers)
```

Node (≥ 20) is required for the Playwright end-to-end tests in `end2end/` and,
later, the Tailwind v4 CSS build via the rust-ui CLI.

## Local dev services (podman)

```sh
podman compose -f deploy/dev/compose.yaml up -d     # PostgreSQL 18, Keycloak, SeaweedFS, Mailpit
podman compose -f deploy/dev/compose.yaml down      # stop (add -v to wipe volumes)
```

PostgreSQL: `postgres://openworkspace:dev@localhost:5432/openworkspace`.

## Database migrations (sqlx-cli)

Migrations live in `crates/db/migrations/` (reversible `-r`: a `.up.sql` + `.down.sql`).
**Always create migrations with the CLI — never hand-name files.** The DDL inside is
authored by hand; the CLI owns file creation and timestamp versioning.

```sh
export DATABASE_URL=postgres://openworkspace:dev@localhost:5432/openworkspace
sqlx migrate add -r <name> --source crates/db/migrations   # scaffold, then edit the SQL
sqlx migrate run    --source crates/db/migrations           # apply pending
sqlx migrate revert --source crates/db/migrations           # roll back the latest
```

Once compile-time `query!`/`query_as!` macros exist, regenerate the committed offline
cache (`.sqlx/`); CI builds with `SQLX_OFFLINE=true`:

```sh
cargo sqlx prepare --workspace
```

## Quality gates (run before pushing — CI runs the same)

```sh
cargo fmt --all --check
leptosfmt --check apps crates
cargo clippy --workspace --all-targets -- -D warnings    # NOT --all-features (ssr+hydrate conflict)
cargo nextest run --workspace                            # needs the dev DB up for integration tests
cargo deny check && cargo audit
cargo leptos build                                        # full build: server ssr + frontend wasm + CSS
```

## Run the app

```sh
cargo leptos watch        # serves http://127.0.0.1:3000 with hot reload
```

Configuration is profile-based (`APP_PROFILE`, default `dev`) from `config/app.toml`
plus `APP_*` env overrides (`APP_DATABASE__URL`; `__` denotes nesting). See `crates/config`.
