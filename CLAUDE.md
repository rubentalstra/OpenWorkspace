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
- **Merge** — once every check is green, squash-merge and clean up in one step: `gh pr merge <n> --squash --delete-branch` (removes the local and remote branch and returns you to `main`), then `git checkout main && git pull --ff-only` to sync. The squash keeps one Conventional-Commit per PR on `main`.
- **SemVer 2.0.0** — `fix` → PATCH, `feat` → MINOR, breaking → MAJOR. Pre-V1 the crate is `0.y.z`: breaking changes bump MINOR, everything else PATCH. Follow Cargo's SemVer rules for what counts as breaking.

## Rust practices

**Idiomatic baseline (always apply)** — All Rust code adheres to the vendored `rust-skills` ruleset (`.claude/skills/rust-skills`; invoke `/rust-skills`): 265 rules across ownership, errors, async, concurrency, unsafe, API design, memory, performance, types, serde, testing, and anti-patterns, current for Rust 1.96 / edition 2024. When writing, reviewing, or refactoring Rust, load the relevant rules and follow them — this is the default standard for idiomatic, fast, safe code. **On any conflict, this file and [`docs/rust-style.md`](docs/rust-style.md) win** — they pin our stack-specific choices and deliberate lint allows; the generic ruleset never overrides them.

**Style** — `edition = "2024"` in `Cargo.toml` and `rustfmt.toml`; rely on rustfmt defaults (4 spaces, 100 cols, trailing commas). In `rustfmt.toml` pin only stable keys — never `imports_granularity`/`group_imports`/`wrap_comments` (nightly-only, silently no-op). Format `view!` with `leptosfmt`. Edition-2024: wrap `no_mangle`/`link_section` as `#[unsafe(...)]`; FFI is `unsafe extern "C"`; escape the new keyword as `r#gen`; add `+ use<>` to RPIT only to deliberately narrow lifetime capture.

**Module layout** — keep `src/` shallow. Once a crate outgrows a handful of files (~5 `.rs` at the root), group cohesive modules into subdirectories (`feature/{mod.rs, …}`) rather than piling flat files at the root. The submodule's `mod.rs` only declares its children (`pub(crate) mod …`); `lib.rs` stays a thin facade of `mod` declarations plus the `pub use` re-exports that form the crate's public surface. Re-export that surface (and any cross-module types) at the crate root so call sites stay path-stable (`crate::Foo`, `othercrate::Foo`) no matter which file a symbol lives in. Current groupings: `auth` → `password`/`session`/`mfa`; `db` → `identity` (+ root `bookings`); `domain` → `model`/`booking`/`access`.

**Errors** — In `crates/*` libraries define typed errors with `thiserror`; never expose `anyhow::Error` in a public API. In `apps/*` binaries use `anyhow::Result` with `.context()` at each fallible step. Propagate with `?` (auto-`From`), not match-and-return. `panic!`/`unwrap`/`expect` only for broken invariants, never expected failure. Public errors are `Error + Send + Sync + 'static`; `Display` lowercase, no trailing punctuation; carry cause via `#[source]`/`#[from]`.

**API** — RFC 430 casing, acronyms as one word (`Uuid`, not `UUID`). Prefix conversions by cost/ownership: `as_` / `to_` / `into_`. Implement `From`/`TryFrom`/`AsRef`, never `Into`/`TryInto`; derive common traits eagerly and `Debug` on every public type. Keep struct fields private; seal extension traits with a private supertrait.

**Async / safety** — Never block in async: short blocking work → `spawn_blocking`, CPU work → rayon, cap concurrency with a `Semaphore`. Never hold a `std::sync::Mutex` guard across `.await` (drop it first); share state via `Arc`. In `select!` call only cancellation-safe futures; hoist non-cancel-safe ops out of the loop. `unsafe_code` is `deny` workspace-wide (no crate currently needs an exception).

**Stack** — Treat every `#[server]` fn as a public endpoint: validate inputs and enforce auth inside; return `Result<T, E: FromServerFnError>`. Keep server-fn args serializable with fixed-width ints (`i32`/`i64`), never `usize`; load async data via `Resource` under `<Suspense/>`. Emit identical server/client HTML (don't branch on `cfg!(target_arch)`); browser-only code goes in `Effect::new`; gate server deps behind the `ssr` feature. Use compile-time `query!`/`query_as!`; regenerate `.sqlx` with `cargo sqlx prepare --workspace` and commit it; CI runs with `SQLX_OFFLINE=true`. Axum handlers return `Result<T, E: IntoResponse>`; one `Clone` `AppState` via `.with_state` with `FromRef` substates; `State` before body extractors.

**Lints** — Pragmatic-strict policy in root `[workspace.lints]` + `clippy.toml`; every crate sets `[lints]\nworkspace = true` (groups need `priority = -1`). Clippy groups (`correctness`/`suspicious`/`complexity`/`perf`/`style`/`all`) are **deny**; `pedantic` is **warn** with the noisy ones allowed. No `unwrap`/`expect`/`panic`/`todo`/`dbg`/`print` in committed code (denied; relaxed in `#[cfg(test)]`). Suppress only with `#[expect(…, reason = "…")]` — bare `#[allow]` is denied. **Full rules + rationale: [`docs/rust-style.md`](docs/rust-style.md).** Run clippy without `--all-features` (ssr+hydrate conflict).

**Security advisories** — `cargo deny check` + `cargo audit` gate every build. **Fix first**: when a patched release resolves an advisory, upgrade to it — a *pre-release is acceptable when it IS the security fix and resolves the issue* (this is the one allowed exception to the no-pre-release rule). **Never ignore an exploitable, fixable vulnerability.** An advisory may be added to the ignore list — in **both** `deny.toml` and `.cargo/audit.toml` (keep them in sync) — ONLY when it cannot be fixed and is justified, each entry carrying a written reason **and** a revisit trigger. Two classes qualify: (a) an `unmaintained`/notice advisory on a transitive dependency we don't control (e.g. via Leptos); (b) a vulnerability whose vulnerable code path our usage provably never exercises **and** for which no compatible patched release exists. Re-evaluate every ignore when the upstream ships a compatible fix. (Current example: `RUSTSEC-2023-0071` in `rsa` — a private-key-only timing attack; we are an OIDC RP that only verifies with public keys, `rsa` is a non-optional transitive dep of `openidconnect`, and no semver-compatible fix exists.)

## Commands

Required CLIs, dev services (podman), migrations and the full gate list: **[`docs/development.md`](docs/development.md)**.

Quick gate (run before pushing — CI runs the same):

```
cargo fmt --all --check && leptosfmt --check apps crates && cargo clippy --workspace --all-targets -- -D warnings && cargo nextest run --workspace && cargo deny check && cargo audit && cargo leptos build
```

## Reference docs

- [Rust Style Guide](https://doc.rust-lang.org/style-guide/) · [rustfmt config](https://github.com/rust-lang/rustfmt/blob/master/Configurations.md) · [Rust 2024 Edition](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
- [API Guidelines: Naming](https://rust-lang.github.io/api-guidelines/naming.html) · [Interoperability](https://rust-lang.github.io/api-guidelines/interoperability.html)
- [Clippy lint groups](https://doc.rust-lang.org/clippy/lints.html) · [Cargo workspaces & lints](https://doc.rust-lang.org/cargo/reference/workspaces.html) · [Cargo SemVer](https://doc.rust-lang.org/cargo/reference/semver.html)
- [thiserror](https://docs.rs/thiserror/latest/thiserror/) · [anyhow](https://docs.rs/anyhow/latest/anyhow/)
- [Leptos: Server Functions](https://book.leptos.dev/server/25_server_functions.html) · [tokio::select!](https://docs.rs/tokio/latest/tokio/macro.select.html) · [sqlx query_as!](https://docs.rs/sqlx/latest/sqlx/macro.query_as.html) · [axum error_handling](https://docs.rs/axum/latest/axum/error_handling/index.html)
- [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) · [SemVer 2.0.0](https://semver.org/)
