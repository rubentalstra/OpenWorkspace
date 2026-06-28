<!--
Title must be a Conventional Commit summary, e.g. `feat(booking): add waitlist`.
See CONTRIBUTING.md and CLAUDE.md.
-->

## What changed

<!-- A concise description of the change. -->

## Why

<!-- The motivation; link the phase/issue (e.g. Closes #123). -->

## How it was verified

<!-- Tests run, manual checks, screenshots. -->

## Checklist

- [ ] Conventional-Commit title; branch off `main`
- [ ] `cargo fmt --all --check` and `leptosfmt --check apps crates` pass
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo nextest run --workspace` passes
- [ ] `cargo deny check` and `cargo audit` pass
- [ ] Docs updated where relevant
