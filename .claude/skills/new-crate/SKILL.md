---
name: new-crate
description: Scaffold a new workspace crate or app the OpenWorkspace way — edition 2024, inherited lints, thiserror error type, thin facade lib.rs. Use when adding a new member under crates/ or apps/.
argument-hint: "<crate_name>"
---

Add a new workspace member that matches the existing crates' shape. Read an existing crate of the same kind first (e.g. `crates/crypto` for a library, `crates/db` for a data-access facade) and mirror its layout rather than inventing a new one.

## Steps

1. Create `crates/<name>/Cargo.toml` (or under `apps/`):
   - `edition = "2024"`, version inherited from the workspace where applicable.
   - `[lints]` → `workspace = true` (so the pragmatic-strict policy applies).
   - Depend on first-party crates and pinned workspace deps; do not add a new third-party crate without checking `deny.toml` license/advisory policy and pinning the exact version.
2. Add the member to the root `Cargo.toml` `[workspace] members` list (keep it grouped with its peers).
3. `src/lib.rs` is a thin facade: `mod` declarations plus the `pub use` re-exports that form the public surface. Keep `src/` shallow; group cohesive modules into subdirectories once it outgrows ~5 root files.
4. Define a typed error with `thiserror` (`Error + Send + Sync + 'static`, lowercase `Display`, cause via `#[source]`/`#[from]`); never expose `anyhow` in the public API. Put any third-party crate behind a first-party facade — no vendor types in the public surface.
5. Add a doc comment (`///`) to every public item. Keep other comments to *why*-only.
6. Confirm it builds and lints: `cargo build -p <name>` and `cargo clippy -p <name> --all-targets -- -D warnings`.

RustRover discovers new crates via `.idea/` — commit any `.idea/` changes alongside the crate.
