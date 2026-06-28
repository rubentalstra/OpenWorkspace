<div align="center">

# OpenWorkspace

**Self-hosted, open-source workspace booking — desks today, with meeting rooms and parking on the roadmap, across multiple sites and entities.**

[![CI](https://github.com/rubentalstra/OpenWorkspace/actions/workflows/ci.yml/badge.svg)](https://github.com/rubentalstra/OpenWorkspace/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.96-blue.svg)](rust-toolchain.toml)
[![Rust edition](https://img.shields.io/badge/edition-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18-336791.svg)](https://www.postgresql.org/)
[![Status](https://img.shields.io/badge/status-pre--1.0-informational.svg)](#status)

</div>

---

OpenWorkspace is deployed once per customer as a production-grade, privacy-respecting
alternative to commercial workspace-booking suites. It is built in Rust on Leptos
(server-side rendered) with an Axum server and a PostgreSQL data tier. The first
release is **desk-only and complete for desks**; **meeting rooms** and **parking**
follow as additive modules on the roadmap.

## Status

Pre-1.0 and under active development toward the first release. Until V1 ships, only
the latest `main` is supported. Expect breaking changes between `0.y` versions.

## Roadmap

- **Now** — desks (complete).
- **Next** — meeting rooms.
- **Planned** — parking.

## Highlights

- **No double-booking, ever** — overlap is rejected by a PostgreSQL exclusion
  constraint, independent of application code.
- **Multi-site, multi-entity** — one deployment models a whole estate with
  per-organization segmentation and a location hierarchy.
- **Safe by construction** — `unsafe` is denied workspace-wide; secrets are typed,
  zeroized and never logged; the supply chain is gated by `cargo-deny` + `cargo-audit`.
- **Authentication built in** — local accounts with Argon2id, passkeys (WebAuthn)
  and TOTP MFA, plus OIDC single sign-on.
- **Localized** — first-class internationalization from the ground up.

## Tech stack

| Layer        | Choice                                              |
| ------------ | --------------------------------------------------- |
| Language     | Rust (edition 2024, MSRV 1.96)                      |
| Web / UI     | Leptos (SSR + hydration), Axum                      |
| Database     | PostgreSQL 18 via `sqlx` (compile-time checked SQL) |
| Background   | apalis worker                                       |
| Styling      | Tailwind CSS v4 (first-party `crates/ui` kit)       |

## Repository layout

```
apps/
  web/        Leptos app (app · frontend · server)
  worker/     Background job runner
crates/
  domain      Booking model, authorization, segmentation
  db          Schema, migrations, queries
  auth        Local auth, sessions, MFA, OIDC SSO
  ui          First-party component library (pure Leptos)
  crypto      Field encryption / key management
  i18n        Internationalization
  config · observability · floorplan · notify · jobs
```

## Getting started

Rust is pinned by [`rust-toolchain.toml`](rust-toolchain.toml). Install the dev tools once:

```sh
cargo install --locked cargo-leptos leptosfmt cargo-nextest cargo-deny cargo-audit
```

Run the full local gate (CI runs the same):

```sh
cargo fmt --all --check \
  && leptosfmt --check apps crates \
  && cargo clippy --workspace --all-targets -- -D warnings \
  && cargo nextest run --workspace \
  && cargo deny check && cargo audit \
  && cargo leptos build
```

Required services (PostgreSQL via Podman), migrations and the complete command list
are documented in [`docs/development.md`](docs/development.md).

## Documentation

- [`OpenWorkspace-architecture-plan.md`](OpenWorkspace-architecture-plan.md) — the master specification (source of truth).
- [`docs/`](docs/) — development setup, Rust style guide, and per-phase specs.

## Contributing

Contributions are welcome — please read [`CONTRIBUTING.md`](CONTRIBUTING.md) and the
project rules in [`CLAUDE.md`](CLAUDE.md) first.

## Security

Found a vulnerability? Please report it privately — see [`SECURITY.md`](SECURITY.md).

## License

Licensed under the [MIT License](LICENSE).
