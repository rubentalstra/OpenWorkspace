# Contributing to OpenWorkspace

The project rules live in [`CLAUDE.md`](CLAUDE.md) — read it first. The essentials:

## Workflow

- Branch off `main`; never push to `main` directly.
- **Conventional Commits** for every commit (`feat(scope): …`, `fix: …`, `docs: …`).
  Breaking changes use `!` and/or a `BREAKING CHANGE:` footer.
- **SemVer** for version bumps. Pre-V1 the crate is `0.y.z` (breaking → MINOR).
- Open a PR with a Conventional-Commit title; describe what changed, why, and how you verified it.
- **DOCS-FIRST**: never guess an API — check the pinned version's official docs.

## Toolchain

Rust is pinned by `rust-toolchain.toml` (1.96.0, edition 2024, `wasm32-unknown-unknown`).
Install the dev tools once:

```sh
cargo install --locked cargo-leptos leptosfmt cargo-nextest cargo-deny cargo-audit
```

## Quality gates (run before pushing — CI runs the same)

```sh
cargo fmt --all --check
leptosfmt --check apps crates
cargo clippy --workspace --all-targets -- -D warnings   # NOT --all-features (ssr+hydrate conflict)
cargo nextest run --workspace --no-tests=pass
cargo deny check && cargo audit
cargo leptos build                                       # full app build (server ssr + frontend wasm)
```

## Conventions

- Errors: typed (`thiserror`) in `crates/*` libraries; `anyhow` with context in `apps/*` binaries.
- `unsafe` is denied workspace-wide; the wasm hydration entry point is the only documented exception.
- Comments explain *why*, not *what*; document public items with `///`.
- Every third-party crate sits behind a thin first-party facade (see the architecture plan, §6.1).

Run the app locally with `cargo leptos watch` (serves at `127.0.0.1:3000`).
