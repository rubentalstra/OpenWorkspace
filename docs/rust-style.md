# OpenWorkspace Rust style & standards

The enforced coding standard. The lint policy in the root `Cargo.toml`
(`[workspace.lints.*]`) + `clippy.toml` is the machine-checked half of this; this
document is the rationale and the conventions lints can't express. Everything here
is mandatory and gated in CI.

## 1. Enforcement

- Policy lives in `Cargo.toml [workspace.lints.{rust,rustdoc,clippy}]` + `clippy.toml`.
  Every member crate sets `[lints]\nworkspace = true` — a crate without it silently escapes all of this.
- CI is the source of truth and runs with **`-D warnings`**, so a `warn`-level lint is effectively
  mandatory: `cargo fmt --all --check`, `leptosfmt --check apps crates`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`,
  `cargo deny check`, `cargo audit`, `cargo leptos build`.
- **Suppress only with `#[expect(lint, reason = "…")]`** — never bare `#[allow]`
  (`allow_attributes` + `allow_attributes_without_reason` are denied). `#[expect]` self-removes
  when the violation goes away, so suppressions can't rot. Every suppression states *why*.

## 2. Edition & formatting

- Edition 2024; rustfmt defaults (only stable keys in `rustfmt.toml`). Format `view!` with `leptosfmt`.

## 3. Errors

- Libraries (`crates/*`): typed `thiserror` enums, `#[non_exhaustive]`, one per crate/module.
  `Display` lowercase, no trailing period; carry the cause via `#[source]`/`#[from]`.
  Never expose `anyhow::Error` or `Box<dyn Error>` in a public library signature.
- Binaries (`apps/*`): `anyhow::Result` with `.context()` at each fallible step.
- Public errors are `Error + Send + Sync + 'static`. Propagate with `?`, never match-and-rewrap.
- A facade may deliberately opaque an upstream error (e.g. `crypto` never leaks argon2 internals);
  that is intentional, not an oversight.

## 4. Panics & reliability

- No `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!`/`unreachable!`/`dbg!` in committed code
  (all denied; `unwrap`/`expect`/print allowed only in `#[cfg(test)]` via `clippy.toml`).
- Prefer `?` + `Result`/`Option`; `.get()` over `[i]`. Reserve panics for truly unreachable invariants
  and document them with a `# Panics` section.
- No silent drops: `let_underscore_must_use`/`let_underscore_future`/`mem_forget` are denied —
  handle the value (`get_or_init`, `if let`, `?`, explicit match).
- No ad-hoc output: use `tracing`, not `print*`/`eprintln!` (denied outside tests).

## 5. Type design & API idioms

- Newtype IDs over bare primitives (`UserId(Uuid)`, not `Uuid`); `NonZero*` for invariantly-positive counts.
- Private struct fields; `#[non_exhaustive]` on public enums/error/config types; `#[must_use]` on pure getters/builders.
- Conversions via `From`/`TryFrom` (never hand-impl `Into`); accept `impl Into<T>`/`impl AsRef<str>`.
- Replace boolean parameters with two-variant enums where it aids the call site.
- `Arc::clone(&x)` over `x.clone()` for ref-counted pointers (`clone_on_ref_ptr` denied); avoid needless clones/allocations.

## 6. Visibility & modules

- `pub(crate)` is the default; bare `pub` only for a crate's deliberate facade surface (`unreachable_pub` denied).
- Every third-party crate sits behind a thin first-party facade; wrapped types don't leak across it.
- No glob re-exports except a single named `prelude` module per crate.

## 7. Documentation

- `///` on public items; runnable doctests use `?`, not `unwrap`. `rustdoc::broken_intra_doc_links` is denied.
- (Ratchet, not yet enforced: `missing_docs`, `missing_errors_doc`/`missing_panics_doc` — enable per-crate as surfaces stabilise.)

## 8. Testing

- Unit tests colocated in `#[cfg(test)] mod tests { use super::*; }` (private-API access).
- Public-API/integration tests in `crates/<name>/tests/`; DB tests use `#[sqlx::test]` (isolated, auto-migrated).
- `proptest` for pure invariants (booking-overlap maths, crypto round-trips); commit `proptest-regressions/`.
- Deterministic, no flakiness; the no-double-booking constraint has both a property test and a DB integration test.

## 9. unsafe

- `unsafe_code = "deny"` workspace-wide; no crate currently needs an exception. If one ever does,
  opt out with a crate-level `#[expect(unsafe_code, reason = "…")]`, one `// SAFETY:` comment per `unsafe` block.

## 10. Dependencies

- Crate-specific dependencies live in the owning crate's `Cargo.toml`; only deps shared by 2+ members
  (or foundational, e.g. `serde`) go in `[workspace.dependencies]`. Pin versions; `cargo-deny` enforces
  permissive licences (no GPL/AGPL) and `cargo-audit` scans advisories.
