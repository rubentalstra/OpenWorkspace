# OpenWorkspace

Self-hosted, MIT, multi-site workspace-booking platform in Rust on Leptos (SSR) + Axum, sqlx + PostgreSQL 18, apalis worker. Cargo workspace, edition 2024. First release is desk-only.

**Source of truth:** `OpenWorkspace-architecture-plan.md` (master spec). Per-phase specs live under `docs/phases/`. Read the relevant section before building — do not infer requirements.

## Hard rules (non-negotiable)

- **Never mention Claude, AI, or any assistant** anywhere — commits, PR titles/bodies, code, comments, docs. No "Generated with" / "Co-Authored-By" trailers of any kind. Author commits as the human only.
- **DOCS-FIRST.** Never guess an API or behaviour. Look it up in the pinned version's official docs first (use the context7 MCP for library docs; fetch live pages for current guidance). Versions are pinned exactly (plan §5) and re-pinned at install — check the docs for the *pinned* version, not memory.
- **Conventional Commits** for every commit; **SemVer** for every version bump (see Workflow below).
- **Minimal comments.** Write self-explanatory code. A comment explains *why*, never *what*. The only required comments: `// SAFETY:` on any `unsafe`, and doc comments (`///`) on public items. Delete commentary that restates the code.
- Every third-party crate stays behind a thin first-party facade (plan §6.1) — never call a vendor API directly from app code.

## Workflow: commits, PRs, versioning

- **Commits** — Conventional Commits 1.0.0: `type(scope): subject`. Types: `feat` `fix` `docs` `style` `refactor` `perf` `test` `build` `ci` `chore`. Scope = crate/app (e.g. `feat(booking):`, `fix(auth):`). Subject imperative, ≤50 chars, no trailing period.
- **Breaking change** — append `!` after type/scope and/or a `BREAKING CHANGE:` footer.
- **PRs** — title is a Conventional-Commit summary; body states what changed and why, links the phase/issue, and lists how it was verified. Branch off `main`; never commit to `main` directly. Open with `gh`.
- **SemVer 2.0.0** — `fix` → PATCH, `feat` → MINOR, breaking → MAJOR. Pre-V1 the crate is `0.y.z`: breaking changes bump MINOR, everything else PATCH. Follow Cargo's SemVer rules for what counts as breaking.

## Rust practices

**Style** — `edition = "2024"` in `Cargo.toml` and `rustfmt.toml`; rely on rustfmt defaults (4 spaces, 100 cols, trailing commas). In `rustfmt.toml` pin only stable keys — never `imports_granularity`/`group_imports`/`wrap_comments` (nightly-only, silently no-op). Format `view!` with `leptosfmt`. Edition-2024: wrap `no_mangle`/`link_section` as `#[unsafe(...)]`; FFI is `unsafe extern "C"`; escape the new keyword as `r#gen`; add `+ use<>` to RPIT only to deliberately narrow lifetime capture.

**Errors** — In `crates/*` libraries define typed errors with `thiserror`; never expose `anyhow::Error` in a public API. In `apps/*` binaries use `anyhow::Result` with `.context()` at each fallible step. Propagate with `?` (auto-`From`), not match-and-return. `panic!`/`unwrap`/`expect` only for broken invariants, never expected failure. Public errors are `Error + Send + Sync + 'static`; `Display` lowercase, no trailing punctuation; carry cause via `#[source]`/`#[from]`.

**API** — RFC 430 casing, acronyms as one word (`Uuid`, not `UUID`). Prefix conversions by cost/ownership: `as_` / `to_` / `into_`. Implement `From`/`TryFrom`/`AsRef`, never `Into`/`TryInto`; derive common traits eagerly and `Debug` on every public type. Keep struct fields private; seal extension traits with a private supertrait.

**Async / safety** — Never block in async: short blocking work → `spawn_blocking`, CPU work → rayon, cap concurrency with a `Semaphore`. Never hold a `std::sync::Mutex` guard across `.await` (drop it first); share state via `Arc`. In `select!` call only cancellation-safe futures; hoist non-cancel-safe ops out of the loop. `unsafe_code` is `deny` workspace-wide — the only exception is the wasm hydration entry crate (`apps/web/frontend`), which carries a documented `#![allow(unsafe_code)]` for the `#[wasm_bindgen]` glue.

**Stack** — Treat every `#[server]` fn as a public endpoint: validate inputs and enforce auth inside; return `Result<T, E: FromServerFnError>`. Keep server-fn args serializable with fixed-width ints (`i32`/`i64`), never `usize`; load async data via `Resource` under `<Suspense/>`. Emit identical server/client HTML (don't branch on `cfg!(target_arch)`); browser-only code goes in `Effect::new`; gate server deps behind the `ssr` feature. Use compile-time `query!`/`query_as!`; regenerate `.sqlx` with `cargo sqlx prepare --workspace` and commit it; CI runs with `SQLX_OFFLINE=true`. Axum handlers return `Result<T, E: IntoResponse>`; one `Clone` `AppState` via `.with_state` with `FromRef` substates; `State` before body extractors.

**Lints** — Define once in root `[workspace.lints]`; every member crate sets `[lints]\nworkspace = true`. Group entries need `priority = -1`. Keep `correctness` at deny, `pedantic` at warn; never enable `restriction`/`nursery`/`cargo` wholesale.

```toml
[workspace.lints.rust]
unsafe_code = "deny"      # wasm `frontend` crate opts out via a documented #![allow]
unused_must_use = "deny"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }      # correctness stays deny
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
must_use_candidate = "allow"     # noisy for Leptos components (`impl IntoView`)
similar_names = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
```
Pair with `clippy.toml`: `allow-unwrap-in-tests`/`allow-expect-in-tests = true`, and `doc-valid-idents` for project nouns (OpenWorkspace, PostgreSQL, OIDC, …). Run clippy without `--all-features` (ssr+hydrate conflict).

## Commands

```
leptosfmt . && cargo clippy --all-targets --all-features && cargo nextest run
cargo deny check && cargo audit
cargo sqlx prepare --workspace   # after changing any query! ; commit .sqlx
cargo leptos watch               # run at 127.0.0.1:3000
```

## Reference docs

- [Rust Style Guide](https://doc.rust-lang.org/style-guide/) · [rustfmt config](https://github.com/rust-lang/rustfmt/blob/master/Configurations.md) · [Rust 2024 Edition](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
- [API Guidelines: Naming](https://rust-lang.github.io/api-guidelines/naming.html) · [Interoperability](https://rust-lang.github.io/api-guidelines/interoperability.html)
- [Clippy lint groups](https://doc.rust-lang.org/clippy/lints.html) · [Cargo workspaces & lints](https://doc.rust-lang.org/cargo/reference/workspaces.html) · [Cargo SemVer](https://doc.rust-lang.org/cargo/reference/semver.html)
- [thiserror](https://docs.rs/thiserror/latest/thiserror/) · [anyhow](https://docs.rs/anyhow/latest/anyhow/)
- [Leptos: Server Functions](https://book.leptos.dev/server/25_server_functions.html) · [tokio::select!](https://docs.rs/tokio/latest/tokio/macro.select.html) · [sqlx query_as!](https://docs.rs/sqlx/latest/sqlx/macro.query_as.html) · [axum error_handling](https://docs.rs/axum/latest/axum/error_handling/index.html)
- [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) · [SemVer 2.0.0](https://semver.org/)
