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

Node (≥ 20) is required for the Playwright end-to-end tests in `end2end/`,
including the `@axe-core/playwright` accessibility gate over the `/ui` showcase.

## Local workflow hooks

Committed git hooks enforce the commit conventions locally so a bad commit never
reaches CI. Activate them once per clone:

```sh
git config core.hooksPath .githooks
```

- `commit-msg` validates the message: Conventional Commits, no attribution
  trailers, no trailing period (subject over 50 chars is a warning).
- `pre-push` blocks pushing to `main`/`master` and runs the cheap fmt gates.

Editor-agent automation lives under `.claude/` (hooks, a Rust reviewer, and
commit / pre-pr / migration / crate skills) — see `.claude/README.md`. The rules
themselves are defined in this repo's `CLAUDE.md` and `docs/rust-style.md`.

## UI design system (`crates/ui`)

A first-party, **stable-Rust** component kit (no `leptos/nightly`) — 83 components + 18 hooks:

- **Styling:** `tw_merge` behind the first-party `cn!` facade, plus our own `clx!`/`void!`/`variants!` macros under `tw/` (not leptos_ui's). Lucide icons via `leptos_icons` + `icondata`, used directly. `variants!` generates plain `Copy` enums + a `match`-based `class()` — no `tw_merge` derive macros.
- **Zero JavaScript:** every interactive component (dialog, sheet, popover, tooltip, menus, command, select, carousel, drawer, …) is pure Leptos — `RwSignal` + `<Show>` + `window_event_listener`, focus/scroll-lock/ARIA. No `<script>` blocks, no `.js` assets.
- **Tailwind v4 (Pipeline A):** cargo-leptos's bundled binary builds `[[workspace.metadata.leptos]] tailwind-input-file = apps/web/style/tailwind.css`; the `@source` globs there must cover `apps/web/app/src` and `crates/ui/src`. `crates/ui` must **not** carry its own `[package.metadata.leptos]` (cargo-leptos would treat it as a second lib-package and the build fails).
- **Dark mode:** `ui::ThemeMode::init()` in the app root — light default, resolves from `localStorage` / `prefers-color-scheme`, toggles the document `dark` class (pure Leptos, no inline theme script). The `/ui` route renders the component showcase.
- **Lint:** run clippy per-feature (`--features ssr` *or* `--features hydrate`), never `--all-features` (the two conflict).
- **Note:** `AutoForm`'s `#[derive(AutoForm)]` is consumer-side tooling — a future proc-macro crate generates the `AutoFormFields` impl; the component itself is generic over that trait.

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
