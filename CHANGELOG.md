# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Pre-1.0, breaking changes bump the MINOR version.

## [Unreleased]

### Added

- Platform foundation: workspace scaffold, configuration, and observability.
- Data model and booking engine with database-enforced no-double-booking.
- Segmentation and permissions (authorization decision layer).
- Authentication: local accounts (Argon2id) with sessions, passkeys (WebAuthn)
  and TOTP MFA, and OIDC single sign-on.
- Internationalization foundation.
- First-party UI component library (`crates/ui`).
- Repository documentation and metadata: README with badges, code of conduct,
  issue/PR templates, and editor configuration.

[Unreleased]: https://github.com/rubentalstra/OpenWorkspace/commits/main
