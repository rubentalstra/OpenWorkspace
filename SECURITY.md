# Security policy

## Reporting a vulnerability

Please report security issues **privately**, not via public issues or pull requests.

Use GitHub's private vulnerability reporting: open the repository's **Security** tab
→ **Report a vulnerability**. We aim to acknowledge a report within a few working days
and will coordinate a fix and disclosure timeline with you.

## Supported versions

OpenWorkspace is pre-1.0 and under active development toward its first release.
Until V1 ships, only the latest `main` is supported; security fixes target `main`.

## Scope and posture

- The web tier and worker are written in safe Rust (`unsafe` is denied workspace-wide;
  the sole exception is the wasm hydration entry point, which is isolated and documented).
- No-double-booking is enforced by a database constraint, independent of application code.
- Secrets are typed and zeroized, sourced from the environment or a manager, never logged.
- Supply chain is gated in CI by `cargo-deny` (licenses, bans, advisories) and `cargo-audit`
  (RustSec), with an SBOM and CBOM produced per release.

See `OpenWorkspace-architecture-plan.md` (§4, §7, §6.7) for the full security model.
