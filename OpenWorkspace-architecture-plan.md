# OpenWorkspace: requirements and architecture

OpenWorkspace is a self-hosted, open-source workspace booking platform for desks, rooms, parking, vehicles and equipment across a multi-site, multi-entity estate, built in Rust on Leptos. It is deployed once per customer and is a production-grade alternative to Gingco Share. The first release is desk-only and complete for desks; rooms and the rest follow as additive modules (Section 2, Appendix D). Versions are grounded in official sources as of 26 June 2026 and should be re-pinned at install.

## 1. Summary

A Leptos server-rendered app on Axum holds the UI and, through isomorphic server functions, its data and auth logic, so there is no hand-written API in between. A separate Rust worker owns all background work via apalis. State lives in PostgreSQL 18, where a GiST exclusion constraint on a time range makes double bookings impossible at the database, independent of application code. Two headline features: single sign-on, OIDC only on the mature `openidconnect` and `oauth2` crates (Authorization Code plus PKCE); and an interactive floor plan, each floor a 2D SVG scene built in-app from a component catalog, with raster-only uploads so there is no untrusted SVG and no sanitiser. Accessibility is a hard release gate.

## 2. Scope

**In scope (first release): the complete desk-sharing product.** OIDC SSO with passkeys and TOTP; the location hierarchy; desks as the bookable type with the booking engine and its database-enforced no-double-booking guarantee, recurrence, time zones, check-in and auto-release; the in-app floor builder and SVG catalog; the three views (map, list, calendar) with progressive zoom; a confirmation email with a one-way `.ics`; the admin module with location-scoped delegated roles; the public API; GDPR export and erasure; seven languages; WCAG 2.1 AA; crypto-agile, post-quantum-ready cryptography; and deployment.

**Deferred to later versions** (each additive, none affecting the core): room booking and meeting features (capacity search, Teams and Meet links); the room-serving physical surfaces (door displays, kiosks with RFID and NFC, MQTT sensors); Microsoft Graph and Google Workspace calendar sync (crates graph-rs-sdk, google-calendar3, yup-oauth2); the parking, vehicle and equipment types (the model already supports them); SAML 2.0 (Rust libraries immature); SCIM provisioning, Users and Groups (needs research); visitor management; catering, cost-allocation or SAP export; Microsoft and Google add-ins; wallet passes and native apps; the 3D occupancy heatmap; CalDAV; indoor positioning; occupancy prediction; and NEN 7510 certification.

The dependency-ordered build plan, with per-phase tests, is in Appendix D; each phase becomes a document under `docs/phases/`.

## 3. Functional requirements

### 3.1 Identity and single sign-on (priority)
Local accounts (Argon2id); passkeys (WebAuthn) as a first-class second factor and passwordless option; TOTP. SSO is OIDC only, against any conformant provider (Entra ID, Okta, Google), Authorization Code with PKCE. An instance admin manages the deployment; location-scoped roles delegate management by hierarchy node (Section 6.4). A legal entity is an organization, a department a team within it (roles owner, admin, member, plus custom runtime roles). Resource segmentation is admin-configurable (`open`, `by_organization`, `by_organization_and_team`; Section 6.2). Server-side sessions with secure, http-only, SameSite cookies and CSRF protection; API keys for the public API; an append-only audit log. User profile and preferences match Gingco: time zone, language, default category filter, a Home Zone (default landing) and a default view. Users never pick an organization or team; visibility follows memberships and the Home Zone is the landing.

### 3.2 Locations and resources
Self-referential hierarchy (`Continent -> Country -> City -> Campus -> Building -> Floor -> Object`) with IANA `timezone`, address and JSONB metadata per level. Object types `desk`, `room`, `parking`, `vehicle`, `equipment`; the model supports all, the first release delivers `desk` end to end. Categories, tags, capacity, equipment, accessibility flags, photos. Per-object booking rules (min and max duration, max recurrence, booking window, lead time, cancellation window). Permanent assignment for a validity period; blackouts and maintenance windows; bulk CSV import; floor zones (neighbourhoods) for department scoping. Six UI states, by icon and label never by colour alone: free (green +), partially free (yellow −), not free (red ×), temporarily blocked (red clock), permanent user (person), cannot be booked (grey ×).

### 3.3 Booking engine
Single and recurring bookings: RRULE text plus materialised occurrences, each a time range, so a GiST exclusion constraint makes overlaps impossible at the database (Section 6.3). Recurring exceptions (this, this-and-following, all). Time-zone correct (UTC storage, series zone, viewer-zone rendering). Ad-hoc "book the next free slot"; conflicts return HTTP 409. Best-fit search ranks free resources by capacity, equipment, distance from the Home Zone or a reference object, and recent use. Check-in and check-out happen in the web or mobile app: a check-in window opens a configurable interval before start (default 15 minutes), and after a grace period (default 15 minutes) a worker auto-releases no-shows. Occurrences move `booked -> checked_in -> checked_out`, or `booked -> released`. Checking out early returns the resource for the remaining time; to hold it, do not check out. Private appointments (`public`, `org_visible`, `private`) reveal full detail only to organiser, attendees and delegates. A user can name one or more **delegates** (the assistant or deputy role) who book, modify and cancel in the represented user's name, limited to what that user could do; every booking records both the user it is for and the user who made it, and delegated actions are audited. Users view, modify and cancel their own bookings (cancellation window enforced) via a "my bookings" list.

### 3.4 Floor builder and the SVG component catalog
Each floor is built in-app from a catalog of 2D SVG components; the published floor is pure, app-composed SVG. An admin may upload a reference image (PNG, JPEG, WebP) shown semi-transparent in the builder as a tracing aid only, never served to end users. Uploads are raster images only (reference image, building and campus-map images, object photos, logo); there is no SVG upload and no CAD conversion, so no sanitiser. The builder draws walls and openings, drops rooms, desks, parking and furniture, rotates and snaps to a grid, fills zones, adds labels and wayfinding, binds each bookable component to a resource and its rules, and bulk-places from CSV. The recommended style is a clean, simplified plan for at-a-glance readability.

The catalog (each item an app-defined, themeable, accessible SVG primitive):
- Structure and shell: walls (straight, curved, glass, low partition, railing), column, perimeter, room enclosure, shaft; doors (single, double, sliding, revolving), window; stairs, escalator, lift, ramp, step.
- Bookable resources: desks (single, bench, pod 4/6/8, standing, hot); meeting, conference, huddle, phone booth, wellness room; parking (standard, accessible, EV, motorcycle, bicycle, fleet); bookable equipment slot, locker.
- Furniture and fixtures: chair, armchair, sofa, table; cabinet, shelving, reception desk, coat rail; whiteboard, screen, monitor, printer, server rack; kitchenette, coffee machine, fridge, sink, water cooler, plant, waste.
- Functional rooms and zones: kitchen and break, toilets (M/F/unisex/accessible), shower; server, storage, janitor, mail; reception, open-plan, quiet zone, collaboration zone; first-aid, multifaith, nursing room.
- Zoning: neighbourhood/team polygon (drives segmentation), area fill, group container.
- Wayfinding and safety: entrance, exit, emergency exit, assembly point, extinguisher, AED, first-aid; accessibility marker, "you are here", amenity icons.
- Annotation: text label, callout, north arrow, scale bar, legend.
- Dynamic overlays (runtime): the six states by icon and label, selection highlight, focus ring, tooltip anchor.
- Authoring aids (builder only): reference-image underlay, snap grid, rulers, guides, measure tool, layers.

Campus and site levels are not drawn in SVG: they use an uploaded raster map image with a placed marker per building (the Gingco view). The admin drops one marker per child building and drags it into position; each marker links to its building and shows aggregated state. Positions are stored as image fractions so they scale. The same image-and-marker pattern can apply at city and country levels.

### 3.5 Views and navigation
The core experience is progressive zoom: a breadcrumb from region to country to city to campus to building to floor to object, each level both a map zoom and a quick-jump dropdown. The campus map is the uploaded image with a clickable marker per building showing name and aggregated state (the Pankow Park view); a building opens the exploded 3D floor stack (CSS 3D, with a 2D list as the reduced-motion fallback); a floor opens the 2D SVG with a state pin on every bookable object; an object opens a detail popup with an Object tab (identifier, type, category, breadcrumb, equipment) and an Availability tab (week grid plus rules). A filter bar controls type, persons and the booking date with start and end time. The same data is available as Map, List (sortable, faceted, exportable) and Calendar (week, day, resource timeline). Live changes arrive over SSE and repaint a single node. Full screen inventory in Appendix C.

### 3.6 Notifications and check-in
Every booking emails the booker a confirmation with a one-way RFC 5545 `.ics` (Section 6.6), updated on change and cancelled on cancellation; no external calendar API. Check-in and check-out happen in the responsive web app and mobile PWA. Two-way calendar sync and the physical surfaces (displays, kiosks, sensors) arrive with rooms (Section 2).

### 3.7 Administration, statistics, API, data rights
Administration covers users and roles, the hierarchy editor, resource CRUD with rules, blackout scheduling, OIDC configuration, API keys, the audit-log viewer, the mail-template editor, branding, the floor builder, delegated per-node administration, and `segmentationMode`. A utilisation dashboard (rust-ui charts) and CSV export. GDPR export (Article 15) as JSON plus an ICS, and erasure (Article 17) as anonymisation preserving aggregates. A public REST API at `/api/v1`, OpenAPI 3.1 generated from Rust types and rendered with Scalar, authenticated by API keys.

## 4. Non-functional requirements
- **Performance:** streaming server-rendered first paint; fine-grained reactivity updates only changed nodes; common reads in tens of milliseconds; the conflict check is one indexed range query.
- **Scale:** tens of thousands of resources, hundreds of locations, thousands of concurrent users. The web tier is stateless to N replicas; the worker is a singleton.
- **Availability:** 99.9 percent target on the booking path; the web tier tolerates replica loss; the worker uses a Recreate rollout and an advisory lock so a deploy never runs two schedulers.
- **Security:** Argon2id, passkeys, TOTP, server-side sessions, CSRF, strict CSP, per-deployment secrets, append-only audit log; all cryptography is crypto-agile and quantum-safe (Section 6.7); Rust memory safety removes a class of vulnerabilities.
- **Privacy:** GDPR by design (minimisation, private-appointment confidentiality, retention, export and erasure in V1).
- **Internationalisation:** seven languages (en, de, fr, es, it, nl, pl) via Fluent, `<html lang>` per locale, RTL addable later; all times zone-correct.
- **Accessibility (release gate):** WCAG 2.1 AA and EN 301 549; never colour alone; every bookable SVG object a focusable named node; ARIA live regions for state; reduced-motion 2D fallback; axe-core in CI and NVDA, JAWS and VoiceOver on the manual checklist; vendored rust-ui components audited like first-party code.
- **Observability:** structured tracing with OpenTelemetry, Prometheus metrics, health and readiness endpoints, correlation IDs across web and worker.
- **Testing:** `cargo-nextest` units, `sqlx::test` on real Postgres, `wasm-bindgen-test` components, Playwright end-to-end (with the axe-core pass), `criterion` for hot paths, and SSO flows against a real Keycloak.

## 5. Technology stack
Latest from crates.io, GitHub and npm as of 26 June 2026; re-pin at install.

| Area | Choice | Version |
| --- | --- | --- |
| Language and runtime | Rust 1.96.0 (edition 2024) on Tokio | Rust 1.96.0; Tokio 1.52.3 |
| Web/UI framework | Leptos (SSR, hydration, server functions) | 0.8.20 |
| Build tool | cargo-leptos (build, watch, Lightning CSS, Playwright hook; Tailwind runs via the rust-ui CLI) | 0.3.6 |
| HTTP server | Axum | 0.8.9 |
| UI components, charts, blocks | rust-ui (the single UI source: components, charts and blocks copied into the `ui` crate and owned, via the `ui-cli` tool) + tw_merge + tw-animate-css + icons + Tailwind CSS | rust-ui via ui-cli 0.3.16 (MIT, vendored); tw-animate-css 1.4.0; icons 0.18.3; Tailwind 4.3.1 |
| Reactive hooks | leptos-use (browser-API hooks, e.g. `use_event_source` for SSE; not UI components) | 0.19.0 |
| Internationalisation | leptos-fluent (Mozilla Fluent) | 0.3.1 |
| Database and queries | sqlx (Postgres, compile-time-checked, `PgListener`) + PostgreSQL 18 (`btree_gist`, `pg_trgm`, `citext`) | sqlx 0.9.0; PostgreSQL 18.4 |
| Background jobs and cron | apalis + apalis-sql (Postgres-durable) | 0.7.4 |
| Date/time and recurrence | chrono + chrono-tz + rrule | chrono 0.4.45; chrono-tz 0.10.4; rrule 0.14.0 |
| Sessions and authorization | axum-login + tower-sessions (Postgres store) | axum-login 0.18.0; tower-sessions 0.14.0; tower-sessions-sqlx-store 0.15.0 |
| Passwords, passkeys, TOTP | argon2 (Argon2id) + webauthn-rs + totp-rs | argon2 0.5.3; webauthn-rs 0.5.5; totp-rs 5.7.2 |
| SSO (OIDC, the only SSO for now) | openidconnect + oauth2 (Authorization Code + PKCE) | openidconnect 4.0.1; oauth2 5.0.0 |
| Local OIDC for dev and test | Keycloak (a real OIDC provider for SSO tests; not a runtime dependency) | 26.6.3 |
| Object storage | object_store (S3/GCS/Azure/local) + bundled SeaweedFS | object_store 0.14.0; SeaweedFS 4.36 |
| Image processing | image (decode, re-encode to strip metadata, generate thumbnails) | 0.25.10 |
| Email and calendar invites | lettre (SMTP) + askama templates + icalendar crate | lettre 0.11.22; askama 0.16.0; icalendar 0.17.12 |
| Public API | utoipa + utoipa-axum + utoipa-scalar | utoipa 5.5.0; utoipa-axum 0.2.0; utoipa-scalar 0.3.0 |
| Cryptography | `crates/crypto` facade over aws-lc-rs (and RustCrypto `ml-kem`/`ml-dsa`/`slh-dsa`); rustls with `prefer-post-quantum` (X25519MLKEM768) | rustls 0.23.41; aws-lc-rs 1.17.0; ml-kem 0.3.2; ml-dsa 0.1.1; slh-dsa 0.1.0 |
| Observability | tracing + tracing-subscriber + opentelemetry + metrics-exporter-prometheus | tracing 0.1.44; tracing-subscriber 0.3.23; opentelemetry 0.32.0; metrics-exporter-prometheus 0.18.3 |
| HTTP client, errors, config, secrets | reqwest; thiserror + anyhow; figment; secrecy + zeroize | reqwest 0.13.4; thiserror 2.0.18; anyhow 1.0.103; figment 0.10.19; secrecy 0.10.3; zeroize 1.9.0 |
| Build and deploy | rootless Podman + Buildah with cargo-chef; Helm; `podman kube play`; leptosfmt | cargo-chef 0.1.77; leptosfmt 0.1.33 |
| Licence | MIT | n/a |

The UI is not a crate dependency: rust-ui is a shadcn-style registry whose components are copied into the `ui` crate and owned, so there is no UI framework crate and no UI pre-release pin, only the small MIT `icons` and `tw_merge` utilities. Components are added and updated with the rust-ui CLI (`ui-cli`), whose `ui init` also provisions the Tailwind v4 build (`@tailwindcss/cli` and `tw-animate-css` over npm, with no JavaScript config); that CSS build runs on Node, which is already required for Playwright. apalis is pinned to stable 0.7.4 (a 1.0 RC line exists); there are no pre-release crate pins in the stack.

## 6. Architecture

### 6.1 Workspace and the two deployables
One Cargo workspace, two binaries from shared crates. `apps/web` is the Leptos SSR app on Axum, stateless and horizontally scalable; `apps/worker` is the apalis background processor, a single replica. Every third-party library sits behind a thin first-party facade so it can be swapped without touching call sites: `domain` (entities, booking and segmentation rules), `db` (sqlx, migrations), `auth` (OIDC, sessions, passkeys), `floorplan` (catalog, scene model and renderer, builder), `crypto` (the crypto facade), `notify` (email, `.ics`, outbox), `jobs` (apalis definitions), `ui` (vendored rust-ui), and `config`. Web and worker share `domain`, `db`, `crypto` and `notify`, so rules and email behave identically online and in the background. Layout in Appendix B. The Microsoft and Google calendar crates and the `sensors` crate (MQTT) arrive with the later room and device phases.

### 6.2 Identity and single sign-on (OIDC, the priority)
SSO is OIDC only on the spec-complete `openidconnect` and `oauth2` crates; OpenWorkspace writes no protocol code, only strict configuration: Authorization Code with PKCE (S256), discovery and JWKS, ID-token validation (issuer, audience), `nonce` and `state`, refresh. SAML is deferred (Section 2) behind the same `auth` facade. Local accounts use Argon2id; passkeys use `webauthn-rs`; TOTP is supported. Sessions and authorization run on `axum-login` with `tower-sessions` on Postgres, with one `AuthzBackend` as the single place permissions are decided, including **delegation**: when user A has granted user B an active delegate relationship, B may create and manage bookings in A's name, and the action is evaluated against A's own permissions and segmentation rather than B's. Provisioning is OIDC just-in-time on first sign-in, plus admin invitation and CSV import; SCIM is deferred. For development and integration tests the bundled dev stack runs a real **Keycloak** OIDC provider with a pre-seeded realm, so the full Authorization Code and PKCE flow is exercised against conformant software; production uses the customer's own provider. There is no user-facing organization or team selector: visible resources are the union of memberships, and the Home Zone is the landing. Visibility is governed by one setting, `segmentationMode`, enforced centrally in `domain` and, for defence in depth, optional Postgres row-level security. Each resource carries an `organization_id` and a nullable `team_id`; a floor zone assigns team and organization to everything inside it; a resource with neither inherits its location's organization. `by_organization` matches the user's organizations, `by_organization_and_team` also their teams.

### 6.3 Data model and the no-double-booking guarantee
Core tables: self-referential `locations` (each with a default `organization_id`), `resources` (with `organization_id` and nullable `team_id`), `bookings` (series, RRULE, visibility, and both `booked_for_user_id`, the person it is for, and `booked_by_user_id`, the person who made it, a delegate or the user themselves), `booking_occurrences` (one row per instance), `users`, `organizations`, `teams`, memberships, `role_grants` (location-scoped), `booking_delegates` (who may book in whose name, with a validity period), `floor_plans` (the scene), `resource_positions` (bindings), `audit_log`, and the apalis tables. Overlap prevention is structural: each occurrence stores a `tstzrange` and the table carries an exclusion constraint.

```sql
CREATE EXTENSION IF NOT EXISTS btree_gist;

ALTER TABLE booking_occurrences
  ADD CONSTRAINT no_overlap
  EXCLUDE USING gist (resource_id WITH =, period WITH &&)
  WHERE (status IN ('booked', 'checked_in'));
```

A conflicting insert raises `23P01`, mapped to HTTP 409, so two simultaneous requests for the last slot can never both succeed regardless of timing; multi-row operations use a higher isolation level with retry on serialization failure. `rrule` expands series into occurrence rows, so recurrence and conflict-prevention share one mechanism, with `chrono` and `chrono-tz` keeping it DST-correct. The Postgres driver is pure Rust (no libpq); `tstzrange` maps to sqlx's `PgRange<DateTime<Utc>>`, dates use sqlx's `chrono` feature, and TLS uses `rustls` with `aws-lc-rs` to match Section 6.7. The full schema, with every table, constraint, index and a security and integrity audit, is in Appendix H.

### 6.4 Floor builder, scene model, and scoped administration
A floor is a structured scene: a JSON document of placed component instances, each with catalog type, a stable `scene_node_id`, geometry (point, line, polygon or path), transform and style overrides. The resource binding is normalised into `resource_positions(resource_id, floor_id, scene_node_id)`, not duplicated in the JSON, so a resource can be found, moved or rebound without parsing the scene. The runtime renders the scene to inline SVG via Leptos's `view!`. Structured instances keep floors small, queryable and re-themeable (a deployment can restyle every wall or desk by changing catalog styles). Bookable components carry a reactive `data-state` from Tailwind v4 variables; an SSE change mutates only that node. Pan and zoom are native pointer and wheel handlers; only the active floor mounts, and dense floors split into on-demand zone layers. Because every component is a real DOM node, the floor is keyboard-navigable and bookable, which makes the accessibility gate achievable. The campus map is the uploaded image with a clickable marker per building (state aggregated from its floors); the image reference lives on the campus location and each building stores its marker as x and y fractions, so nothing computes campus geometry. The builder is administration delegated per node: a grant is (a user or group, a role, a node) applying to that node and everything beneath it, evaluated in the same `AuthzBackend` as the rest (a management action is allowed when an instance role, an organization or team role, or a location-scoped grant covers it, a union), while booking visibility stays governed by `segmentationMode`. This hands the builder safely to local facilities staff without instance-wide power.

### 6.5 Real-time, public API, and the background worker
Live updates use server-sent events, not WebSockets, which suit one-way fan-out and traverse proxies cleanly: a commit issues a Postgres `NOTIFY`, an Axum SSE endpoint on sqlx `PgListener` receives it, the browser consumes it with `leptos-use` `use_event_source`, and the affected node repaints. The public API at `/api/v1` is generated as OpenAPI 3.1 from the Rust types with `utoipa` and `utoipa-axum` and served with Scalar, so docs cannot drift. All background work runs in the single worker on apalis with a Postgres queue and persisted cron: dispatch the email outbox every minute; auto-release no-shows every five minutes; at 02:00 materialise recurrences and enforce retention. The worker is one replica with a Recreate rollout; every once-only task wraps its body in `pg_try_advisory_lock` and all jobs are idempotent, so an overlap cannot double-run. Calendar polls and subscription renewals arrive with rooms.

### 6.6 Storage, email and the .ics, and check-in
Binary assets (reference, building and campus-map images, object photos, logo) use `object_store` over S3, GCS, Azure Blob or local disk, with bundled SeaweedFS as the default for dev and on-prem and presigned URLs; SeaweedFS fits the permissive licence and the many-small-files workload. Email is rendered with `askama` and sent with `lettre` over SMTP through the worker outbox, each message idempotent so a retry never double-sends; full template list in Appendix A. Every booking carries a complete RFC 5545 `.ics` produced by the `icalendar` crate behind `notify`: stable `UID`, monotonic `SEQUENCE`, one matching `VTIMEZONE` per zone, `STATUS`, `ORGANIZER` and `ATTENDEE`, a check-in `VALARM`, and per-instance `RECURRENCE-ID` for series. Create and reschedule send `METHOD:REQUEST`; cancellation sends `METHOD:CANCEL` with `STATUS:CANCELLED`; the message is packaged for email per iTIP (RFC 5546) and iMIP (RFC 6047) so Outlook, Google and Apple Calendar apply updates and cancellations correctly. It is one-way and uses no external calendar API. Appendices F and G specify the exact objects emitted, the per-operation lifecycle, and the email packaging. Check-in is driven by the web and mobile app, with auto-release closing no-shows.

### 6.7 Cryptography: crypto-agility and post-quantum readiness
All cryptography lives in `crates/crypto`, exposing intent-named operations (`encrypt_field`, `sign_token`, `hash_password`) rather than algorithms, every artefact tagged by a versioned suite identifier so algorithms rotate centrally and old data still verifies. It is a facade over vetted libraries and implements no primitives: `aws-lc-rs` for mainstream and FIPS-track algorithms, and RustCrypto `ml-kem`, `ml-dsa` and `slh-dsa` for the post-quantum schemes (FIPS 203, 204, 205). Asymmetric cryptography is hybrid post-quantum (classical and PQC combined so a break in either still protects the data), including `X25519MLKEM768` for TLS via `rustls` with `prefer-post-quantum`. The symmetric and hashing primitives in use (AES-256-GCM, ChaCha20-Poly1305, SHA-2, BLAKE3, Argon2id) are quantum-resistant at their sizes and are kept. A CBOM (CycloneDX) per release enumerates every algorithm, key and certificate, so migration is a bounded change in one crate.

### 6.8 Deployment
Images are built with rootless Podman and Buildah using `cargo-chef` for layer-cached multi-stage builds and a slim runtime; the toolchain is pinned with `rust-toolchain.toml`. A Helm chart deploys the web tier as N replicas and the worker as a single-replica Recreate Deployment; a single host uses `podman kube play` with the same manifests, so small and large installs share one definition. PostgreSQL 18 with `btree_gist`, `pg_trgm` and `citext` is a hard requirement, bundled in the dev and compose images, which also run a Keycloak OIDC provider and a mail catcher for local SSO and email testing.

## 7. Security and compliance
GDPR is built in: export (Article 15) as JSON plus an ICS and erasure (Article 17) as aggregate-preserving anonymisation, both in V1, with minimisation, private-appointment confidentiality and configurable retention. Accessibility to WCAG 2.1 AA and EN 301 549 is a release gate (Section 4) and satisfies the European Accessibility Act for the product surface. Secrets are typed with `secrecy`, zeroised with `zeroize`, sourced from the environment or a manager, never logged, with an optional KMS envelope path. The audit log is append-only. CI enforces the supply chain: `cargo-audit` (RustSec), `cargo-deny` (permissive licences only, GPL and AGPL rejected; no duplicate or yanked crates), and an SBOM and CBOM (CycloneDX) per release. The dependencies warranting most attention, all behind facades, are `apalis` (stable 0.7.4, 1.0 ahead), `Leptos` (pre-1.0), and the post-quantum crates (young). The Microsoft and Google SDKs are community-maintained and will be wrapped behind facades when the room-and-calendar phase begins.

## 8. Key risks
- **Pre-1.0 dependencies** (Leptos and others): pin exactly, wrap behind facades, upgrade on a tested schedule.
- **rust-ui is young and single-maintainer:** copy-paste means components are vendored and owned, removing abandonment and treadmill risk; accessibility is not assumed, every adopted component is audited and fixed to the Section 4 gate, verified by axe-core.
- **Community Microsoft and Google SDKs** (room phase): thin facades, a raw `reqwest` fallback, contract tests.
- **apalis 1.0 still in RC:** pin stable 0.7.4 behind the `jobs` facade, keep the queue swappable.
- **WASM bundle size:** server-render first, hydrate selectively, code-split, budget in CI.
- **The worker is a singleton:** advisory locks plus a Recreate rollout make a second instance safe; a scalable worker is a later concern.
- **Uploads:** raster only, so no untrusted-SVG surface; endpoints still validate type and size and re-encode to strip metadata.
- **Floor-builder usability and dense scenes:** a curated catalog, snap-to-grid and templates, mounting only the active floor, measuring node count and frame time in CI.
- **Post-quantum crate maturity:** hybrid mode never regresses below classical security; primitives sit behind the crypto facade.
- **PostgreSQL 18 and extensions required:** documented as a hard requirement and bundled in the dev images.

## 9. Conclusion
OpenWorkspace is a Rust-native, self-hosted booking platform whose first release is complete for desk sharing: OIDC-only SSO on mature crates, an in-app SVG floor builder with raster-only uploads (so no untrusted SVG and no sanitiser), and a no-double-booking guarantee enforced by the database rather than application code. All cryptography is crypto-agile and quantum-safe behind one facade. Rooms, calendar integration, the physical surfaces and the other resource types follow as additive modules (Section 2, Appendix D), so the platform earns memory safety, one language across the stack, small device payloads and correctness guaranteed at the database without over-building the first release.

## Appendix A: Email templates
Rendered in `notify` with `askama`, sent with `lettre` through the idempotent outbox, localized to the seven languages and themed per instance; booking emails carry the one-way `.ics`. "(later)" marks the rooms, devices and calendar phases.

- **Account and identity:** invitation to join; welcome and account ready; email verification; email-change notice (to the old address); password reset; password changed; passkey added or removed; two-factor enabled or disabled; new-sign-in alert (optional); account suspended or reactivated; account offboarded; added or removed as someone's delegate.
- **Booking lifecycle (desk):** confirmed (single); confirmed (recurring series); updated; cancelled by user; cancelled by admin (with reason); moved or reassigned; booked on your behalf; upcoming reminder; check-in reminder; no-show and auto-release notice; check-out reminder (optional); waitlist available (optional).
- **Administration and operations:** blackout or maintenance affecting a booking; resource taken out of service; permanent assignment granted or revoked; admin role granted or revoked; booking approval requested, approved or rejected (if approvals are enabled); admin digest (optional).
- **Data rights (GDPR):** export requested; export ready (time-limited link); erasure confirmed.
- **System:** test email (verify SMTP and branding); generic system notice.
- **Later:** room booking confirmed with Teams or Meet link; calendar connected or disconnected; calendar sync failure; meeting invitation and updates via the connected calendar; device paired or revoked; device offline alert.

## Appendix B: Repository and workspace layout
A single Cargo workspace (Rust 1.96, edition 2024): two binaries under `apps/` and the shared crates of Section 6.1 under `crates/`.

```
openworkspace/
├── Cargo.toml                  # workspace: members, shared deps, workspace lints
├── Cargo.lock
├── rust-toolchain.toml         # pinned toolchain (1.96, edition 2024)
├── rustfmt.toml
├── leptosfmt.toml              # view! macro formatting
├── deny.toml                   # cargo-deny: licences, bans, advisories
├── README.md
├── LICENSE                     # MIT
├── SECURITY.md
├── CONTRIBUTING.md
├── apps/
│   ├── web/                    # Leptos SSR on Axum, stateless, N replicas
│   │   ├── src/
│   │   │   ├── main.rs         # Axum server, Leptos mount, route table
│   │   │   ├── app.rs          # root component, router, app shell
│   │   │   ├── routes/         # one module per screen (Appendix C)
│   │   │   ├── server_fn/      # isomorphic server functions
│   │   │   ├── api/            # REST /api/v1 handlers (utoipa)
│   │   │   └── sse.rs          # SSE endpoint (PgListener fan-out)
│   │   ├── style/              # Tailwind entry, theme tokens
│   │   └── public/             # PWA manifest, service worker, icons, static assets
│   └── worker/                 # apalis processor, single replica
│       └── src/
│           ├── main.rs         # runtime, cron, advisory-lock guards
│           └── jobs/           # outbox, auto-release, recurrence, retention
├── crates/
│   ├── domain/                 # entities, booking and segmentation rules (no I/O)
│   ├── db/                     # sqlx access and queries
│   │   └── migrations/         # schema, GiST exclusion constraint, RLS policies
│   ├── auth/                   # OIDC, sessions, passkeys, TOTP
│   ├── floorplan/              # SVG catalog, scene model and renderer, builder, campus markers
│   ├── crypto/                 # crypto-agile facade (PQC, hashing, KMS), CBOM
│   ├── notify/                 # email rendering (askama), .ics (icalendar), outbox
│   │   └── templates/email/    # the templates of Appendix A, per language
│   ├── jobs/                   # apalis job and schedule definitions, shared by the worker
│   ├── ui/                     # vendored rust-ui components, charts, blocks, shared components
│   └── config/                 # figment config, typed settings, secrets
├── locales/                    # leptos-fluent .ftl resources, one folder per language
├── deploy/
│   ├── helm/                   # Helm chart (web, worker, Postgres, SeaweedFS)
│   ├── containers/             # Containerfile.web, Containerfile.worker (rootless, cargo-chef)
│   ├── kube/                   # podman kube play manifests
│   └── dev/                    # local compose: Postgres, SeaweedFS, Keycloak (realm export), mail catcher
├── docs/
│   ├── architecture-plan.md    # this master plan
│   ├── phases/                 # per-phase specs generated from Appendix D
│   └── adr/                    # architecture decision records
├── .github/workflows/          # CI: fmt, clippy, leptosfmt, audit, deny, test, SBOM and CBOM
└── xtask/                      # optional build and dev automation
```

## Appendix C: Screens and views
First-release screen inventory by surface; the three booking views are detailed in Section 3.5. "(later)" marks the rooms phase.

- **Authentication and onboarding:** sign-in (local, SSO button, passkey); SSO callback; two-factor (TOTP setup and challenge); forgot and reset password; email verification; accept invitation; session expired and sign-out.
- **Booking (end user):** Map view (campus markers, floor SVG with object pins, object detail with Object and Availability tabs, filter bar); List view (sortable, faceted, exportable); Calendar view (week, day, resource timeline); booking dialog (single, recurring, persons, private, book on behalf of someone who named you a delegate); booking confirmation; book the next free slot; best-fit search results; my bookings (view, modify, cancel); Home Zone landing.
- **Profile and self-service:** profile (name, photo, Home Zone, language); delegates (name the assistants who may book in your name, and see whose bookings you may manage); notification preferences; security (passkeys, password, two-factor, active sessions); privacy and data (export, erasure); connected calendars (later).
- **Administration** (instance and delegated per node): overview; users and user detail; roles and grants; organizations and teams; hierarchy editor; resources (objects, rules, categories, tags); floor builder (Section 3.4); campus and site map editor (Section 3.4); blackouts and maintenance; permanent assignments; delegations (who may act for whom); bulk CSV import; utilisation dashboard; reports and CSV export; mail templates; branding; OIDC configuration; API keys; audit-log viewer; settings (segmentationMode).
- **Mobile (PWA):** the responsive end-user surface for booking and check-in on the go (door displays and kiosks arrive with rooms).
- **System and utility:** error pages (403, 404, 500); maintenance and offline pages; the Scalar API reference at `/api/v1`.

## Appendix D: Phased delivery plan and test strategy
This is the dependency-ordered build plan. A phase depends only on earlier phases (the parenthetical lists them), so nothing is built before what it needs. It is greenfield: until V1 ships, breaking changes are free (no migration shims, no API-version churn). Testing is the definition of done: every phase lands with its tests green in CI, and accessibility and i18n are built into each UI phase, not bolted on at the end. V1 (Phases 0 to 20) ships as one complete desk-sharing product; the phases are build order, not interim releases. Each phase becomes a document under `docs/phases/`.

Test layers used throughout: unit (domain rules), integration (sqlx on real Postgres, the exclusion constraint), property-based (proptest for the overlap invariant), component and end-to-end (Leptos tests, Playwright), API contract (OpenAPI), accessibility (axe-core plus manual screen readers), security (cargo-audit, cargo-deny, the authz matrix), and performance (load and bundle budgets). Every release emits an SBOM and CBOM (CycloneDX).

### V1: the complete desk-sharing product (Phases 0 to 20)
- **P0 Workspace skeleton and CI** (none): the Cargo workspace, crate and app stubs, the CI pipeline (fmt, clippy, leptosfmt, cargo-audit, cargo-deny, nextest), a rootless Podman and cargo-chef build, a health endpoint, a skeleton Helm chart. Done: CI is green and the app boots.
- **P1 Platform services** (P0): config (figment), secrets (secrecy, zeroize), observability (tracing, OpenTelemetry, Prometheus), the db connection and migration runner (sqlx, btree_gist), and the crypto facade with Argon2id hashing. Done: migrations apply and roll back; hashing unit tests pass.
- **P2 Data model and migrations** (P1): the full schema (hierarchy, resources, organizations, teams, users, memberships, role_grants, bookings with booked-for and booked-by users, booking_delegates, booking_occurrences with the tstzrange and EXCLUDE USING gist constraint, floor_plans, resource_positions, audit_log). Done: integration tests show the constraint rejects overlapping inserts and maps 23P01 to a conflict.
- **P3 Booking engine, headless** (P2): single and recurring bookings via rrule, time-zone and DST correctness (chrono, chrono-tz), check-in, auto-release, early-checkout-frees-resource, the private flag. Done: unit and property-based tests for the overlap invariant, recurrence, DST and the state machine.
- **P4 Segmentation and permissions** (P2): the segmentationMode model and permission composition (instance, location-scoped, organization and team, union) as pure functions. Done: the authorisation matrix (role by scope by segmentation) passes.
- **P5 Local auth and sessions** (P1, P2): Argon2id accounts, axum-login with tower-sessions on Postgres, secure cookies, CSRF, the single AuthzBackend. Done: login, logout, session and CSRF tests.
- **P6 Passkeys and TOTP** (P5): webauthn-rs passkeys and totp-rs two-factor. Done: registration and challenge tests with a virtual authenticator.
- **P7 OIDC SSO and provisioning** (P5): openidconnect and oauth2 (Authorization Code, PKCE S256, discovery, JWKS, nonce, state) and just-in-time provisioning on first sign-in. Done: a full flow against a real Keycloak instance with a pre-seeded realm, plus JIT.
- **P8 Access enforcement** (P4, P5): RBAC, delegation (acting in another user's name through an active delegate grant, evaluated against the principal), and segmentation visibility wired into requests and queries, with optional Postgres RLS for defence in depth. Done: allow and deny enforced end to end across roles, scopes, delegation and segmentation.
- **P9 Object storage and uploads** (P1): object_store with bundled SeaweedFS and the raster upload pipeline (re-encode with image, presigned URLs, reject non-raster). Done: upload, re-encode and format and size-rejection tests.
- **P10 Scene model, renderer and catalog** (P2, P9): the floorplan scene model, the SVG component catalog, and the read-only inline-SVG renderer with reactive data-state, pan and zoom. Done: scene round-trips and renderer snapshots are stable.
- **P11 Floor builder and campus editor** (P8, P10): the admin floor builder with resource_positions binding, and the campus map-image-and-markers editor. Done: a builder end-to-end (place a desk, bind it), marker-position storage, and builder accessibility checks.
- **P12 Navigation** (P10, P11): progressive zoom, the region-to-object breadcrumb, the campus markers and the exploded 3D floor stack with its 2D-list fallback. Done: navigation end-to-end and the reduced-motion fallback.
- **P13 Booking views** (P3, P8, P10, P12): the Map, List and Calendar views with the object detail popup and the filter bar. Done: view, filter and calendar (time-zone) tests; keyboard and screen-reader checks.
- **P14 Booking actions** (P3, P13): create, edit, recurring, cancel, my bookings, book-the-next-free-slot, best-fit search, the Home Zone landing, delegate management (name your assistants) and booking on behalf of someone who named you. Done: end-to-end booking journeys, including a delegated booking made in another user's name.
- **P15 Live updates** (P3, P13): SSE over Postgres NOTIFY and sqlx PgListener, consumed with leptos-use, repainting a single node. Done: a cross-session live-update end-to-end.
- **P16 Notifications and lifecycle jobs** (P3, P14): the notify crate (askama, the one-way .ics, the idempotent outbox), the worker jobs (outbox, auto-release, recurrence materialisation, retention), the check-in flow, and the Appendix A templates. Done: per-language rendering, RFC 5545 and iTIP `.ics` validity against the Appendix F golden files, outbox idempotency, job and SMTP-catcher tests.
- **P17 Administration core** (P8, P9): users and roles and grants, organizations and teams, the hierarchy editor, resource CRUD with rules, blackouts and maintenance, permanent assignments, and settings (segmentationMode). Done: admin CRUD and the admin authorisation matrix.
- **P18 Administration extras and public API** (P14, P17): CSV bulk import, branding, the mail-template editor, OIDC configuration, API keys, the audit-log viewer, and the public API at /api/v1 (utoipa, Scalar). Done: import validation and API contract tests.
- **P19 Statistics and data rights** (P14, P17): the utilisation dashboard (rust-ui charts), CSV export, GDPR export (JSON and ICS) and erasure (anonymisation preserving aggregates). Done: dashboard accuracy, export completeness and erasure correctness.
- **P20 Hardening and V1 release** (all prior): the full i18n pass, the formal accessibility audit and gate, the post-quantum finalisation and CBOM, a security review, the production Helm and podman kube play with backup and restore and dashboards, and load and performance testing. Done: the audit passes, no missing i18n keys, the security and load targets are met, a backup-restore test passes, and a full release acceptance.

### Beyond V1 (Section 2 expansions, in dependency order)
- **P21 Room booking** (V1): the room type, capacity search, meeting fields, reusing the engine, views and builder. Done: room flows and the no-double-booking guarantee for rooms.
- **P22 Devices: displays, kiosks, sensors** (P21): the door-display PWA (revocable pairing token, offline cache, wake lock), the kiosk PWA (RFID and NFC via the HID-keyboard pattern), the sensors crate (rumqttc) feeding MQTT occupancy to the worker and SSE, and device-pairing administration. Done: pairing, offline, kiosk and sensor-ingest tests.
- **P23 Calendar integration** (P21, P16): the integration crates behind facades (graph-rs-sdk, google-calendar3, yup-oauth2), delegated OAuth, Teams and Meet links, free-busy sync, change webhooks, and the renewal and safety-poll jobs. Done: sync against sandbox tenants, reconciliation, and token and webhook renewal.
- **P24 Additional resource types** (V1): parking, vehicles and equipment as additive types with their catalog components and rules. Done: per-type flows and core regression.
- **P25 Further modules** (V1, each independent): SAML 2.0, SCIM (Users and Groups), visitor management, catering and cost-allocation or SAP export, add-ins, wallet passes and native apps, the 3D heatmap, CalDAV, indoor positioning, occupancy prediction, NEN 7510 certification. Done: per-module suites with no V1 regression.

## Appendix E: Project setup commands
Everything is scaffolded with official CLIs so nothing is hand-written and wrong. Commands are current as of 26 June 2026; pin versions to Section 5 after generation. The container commands use `podman` (swap `docker` if preferred).

**Toolchain**
```
rustup toolchain install 1.96.0
rustup target add wasm32-unknown-unknown          # browser hydration bundle, built alongside the native server target
rustup component add rustfmt clippy
printf '[toolchain]\nchannel = "1.96.0"\ntargets = ["wasm32-unknown-unknown"]\n' > rust-toolchain.toml
```

**Developer CLIs**
```
cargo install cargo-leptos --locked               # build, watch and e2e for Leptos
cargo install leptosfmt                            # format the view! macro
cargo install sqlx-cli --no-default-features --features rustls,postgres
cargo install cargo-nextest --locked              # test runner
cargo install cargo-audit                         # RustSec advisories
cargo install cargo-deny                          # licence, ban and advisory policy
cargo install cargo-chef                          # cached container builds
```

**Scaffold the workspace and crates**
```
# the Leptos + Axum workspace skeleton (SSR, hydration and Playwright wired; Tailwind comes from the rust-ui CLI below)
cargo leptos new --git https://github.com/leptos-rs/start-axum-workspace   # name it openworkspace
cd openworkspace
# the background worker and the shared library crates of Appendix B
cargo new --bin apps/worker
for c in domain db auth floorplan crypto notify jobs ui config; do cargo new --lib crates/$c; done
# rename the template crates to apps/web per Appendix B, add every crate to [workspace] members, then:
cargo build
```

**Add dependencies** (into the right crate with `-p`; versions per Section 5)
```
cargo add sqlx -p db --no-default-features --features runtime-tokio,tls-rustls-aws-lc-rs,postgres,chrono,macros,migrate
cargo add chrono chrono-tz rrule -p domain
cargo add apalis apalis-sql -p jobs
cargo add axum-login tower-sessions argon2 webauthn-rs totp-rs openidconnect oauth2 -p auth
cargo add object_store lettre askama icalendar -p notify
cargo add utoipa utoipa-axum utoipa-scalar -p web
cargo add rustls aws-lc-rs ml-kem ml-dsa slh-dsa -p crypto
cargo add figment secrecy zeroize thiserror anyhow tracing -p config
```

**Styling and components (Tailwind v4, set up by the rust-ui CLI)**
```
cargo install ui-cli --force                       # rust-ui component + Tailwind setup
# run in the component-owning crate (crates/ui per Appendix B):
ui init                                            # writes package.json (@tailwindcss/cli, tailwindcss, tw-animate-css) + style/tailwind.css
ui add button input dialog                         # vendor any number of components at once
# style/tailwind.css is Tailwind v4 (no tailwind.config.js); it must contain:
#   @import "tailwindcss";
#   @import "tw-animate-css";
# plus an @source line (relative to the CSS file) for every crate with view! markup, e.g. apps/web and crates/ui
# compile CSS with the npm CLI, run beside `cargo leptos watch`; cargo-leptos serves and minifies the output:
npx @tailwindcss/cli -i style/tailwind.css -o <web-assets>/app.css --watch
```
The rust-ui CLI provisions Tailwind itself, so there is no separate hand-written config; npm and Node are already required for Playwright (below), so this adds no new toolchain.

**Database (sqlx-cli)**
```
export DATABASE_URL=postgres://openworkspace:dev@localhost:5432/openworkspace
sqlx database create
sqlx migrate add -r init_schema                   # then edit migrations/*.sql (Section 6.3)
sqlx migrate run
cargo sqlx prepare --workspace                    # offline query cache for CI; commit .sqlx
```

**Dev services (podman)**
```
# PostgreSQL 18 (btree_gist, pg_trgm, citext are enabled inside the migrations)
podman run -d --name ow-postgres -p 5432:5432 \
  -e POSTGRES_USER=openworkspace -e POSTGRES_PASSWORD=dev -e POSTGRES_DB=openworkspace \
  docker.io/library/postgres:18

# Keycloak 26 for SSO testing, importing the dev realm on startup
podman run -d --name ow-keycloak -p 127.0.0.1:8080:8080 \
  -e KC_BOOTSTRAP_ADMIN_USERNAME=admin -e KC_BOOTSTRAP_ADMIN_PASSWORD=admin \
  -v ./deploy/dev/keycloak:/opt/keycloak/data/import:Z \
  quay.io/keycloak/keycloak:26.6.3 start-dev --import-realm
# export the realm after configuring it in the admin console:
#   podman exec ow-keycloak /opt/keycloak/bin/kc.sh export --realm openworkspace --dir /opt/keycloak/data/import

# SeaweedFS (S3 API on :8333) for object storage
podman run -d --name ow-seaweedfs -p 8333:8333 -p 9333:9333 \
  docker.io/chrislusf/seaweedfs:4.36 server -s3

# Mailpit mail catcher (SMTP on :1025, web UI on :8025)
podman run -d --name ow-mailpit -p 1025:1025 -p 8025:8025 docker.io/axllent/mailpit
```

**Quality gates and run**
```
cargo deny init                                   # then tune deny.toml to the Section 7 policy
npm --prefix end2end install -D playwright @playwright/test && npx --prefix end2end playwright install
leptosfmt . && cargo clippy --all-targets --all-features && cargo nextest run
cargo deny check && cargo audit
cargo leptos watch                                # run the app with hot reload at 127.0.0.1:3000
```

## Appendix F: iCalendar (RFC 5545) conformance
The booking `.ics` must update and cancel cleanly in Outlook, Google Calendar and Apple Calendar. RFC 5545 defines only the file format; the rules for how a client recognises and applies an update, a cancellation or a reply are iTIP (RFC 5546), and the email packaging is iMIP (RFC 6047), both specified in Appendix G. OpenWorkspace emits 5545-valid objects and follows iTIP and iMIP for transport. Related extensions are drawn on where useful: RFC 7986 (for example `COLOR`, `IMAGE`, `CONFERENCE`), RFC 9074 (alarm `UID` and `ACKNOWLEDGED`), RFC 6868 (parameter-value encoding). The `icalendar` crate handles line folding, escaping and assembly; `notify` adds the scheduling fields the crate does not enforce. Properties below cite the RFC section for traceability.

**Serialization rules.** UTF-8; CRLF line endings; content lines folded at 75 octets and never inside a UTF-8 multi-octet sequence (§3.1). TEXT values escape backslash, semicolon and comma, and encode line breaks as `\n`; a colon is not escaped (§3.3.11). Property names, parameter names and enumerated values are case-insensitive, emitted as canonical uppercase (§3.2). The MIME part sets `charset=UTF-8` (§3.1.4).

**VCALENDAR wrapper (§3.4, §3.6, §3.7).** `VERSION:2.0` and a stable `PRODID` in FPI form (for example `-//OpenWorkspace//Booking 1.0//EN`) are required; `CALSCALE:GREGORIAN` is the default and is emitted explicitly; `METHOD` is set per operation (below). The body holds at least one `VEVENT` plus every `VTIMEZONE` those events reference.

**VEVENT, the booking mapping.** Required on every event: `UID` (§3.8.4.7), `DTSTAMP` (§3.8.7.2), `DTSTART` (§3.8.2.4), and `DTEND` (§3.8.2.2) or `DURATION`.
- `UID`: stable for the life of the booking and identical across update and cancel, in `uuid@host` form for global uniqueness. Every calendar client keys on this to match later updates and cancellations.
- `DTSTART` and `DTEND`: a timed booking uses a DATE-TIME with `TZID` referencing a `VTIMEZONE` (building-local time, so the slot stays correct across DST), or UTC with a `Z` suffix. An all-day booking uses a DATE value with `DTEND` set to the next day, because `DTEND` is non-inclusive (§3.6.1). Bookings never use floating time.
- Versioning: `SEQUENCE` (§3.8.7.4; 0 at creation, incremented only on a significant organizer change) plus the iTIP `METHOD`.
- Descriptive: `SUMMARY` (resource and space), `DESCRIPTION`, `LOCATION` (campus, building, floor, desk), `CATEGORIES`, `URL` (deep link to the booking), `CREATED` and `LAST-MODIFIED` (§3.8.7).
- `STATUS` (§3.8.1.11): `CONFIRMED`, `TENTATIVE` or `CANCELLED`. `TRANSP` (§3.8.2.7): `OPAQUE` for a booking that consumes time. `CLASS` (§3.8.1.3): `PUBLIC` by default, `PRIVATE` or `CONFIDENTIAL` for private appointments.
- `ORGANIZER` (§3.8.4.3) and `ATTENDEE` (§3.8.4.1): a group-scheduled invite carries the organizer plus the booker as an attendee with `CN`, `ROLE`, `PARTSTAT`, `RSVP` and `CUTYPE`; a booked room or asset is an attendee with `CUTYPE=ROOM` or `CUTYPE=RESOURCE`. A plain confirmation that is not group-scheduled omits `ATTENDEE`, which §3.8.4.1 forbids when merely publishing.
- Delegate bookings: when a delegate acts for another user, `SENT-BY` on the `ORGANIZER` or `ATTENDEE` names the delegate while the property value stays the represented user (§3.2.18), so the calendar records who acted on whose behalf.
- `VALARM` child (§3.6.6): one `ACTION:DISPLAY` with a relative `TRIGGER` (for example `-PT30M`) as the check-in reminder; a repeating reminder sets both `DURATION` and `REPEAT`, which are required together or not at all.

**VTIMEZONE (§3.6.5).** Every unique `TZID` used by `DTSTART`, `DTEND`, `EXDATE` or `RDATE` has exactly one matching `VTIMEZONE` in the same object. Each `STANDARD` and `DAYLIGHT` sub-component carries `DTSTART` (local time, no `TZID`), `TZOFFSETFROM` and `TZOFFSETTO` (all three required) plus an `RRULE` or `RDATE` for the onsets, and `TZNAME`. Definitions come from the IANA tz database via `chrono-tz`. UTC values carry a `Z` and no `TZID`, and a `TZID` is never placed on a UTC or DATE value.

**Recurring bookings.** `DTSTART` is the first instance and anchors the pattern (§3.8.5.3). `RRULE` lists `FREQ` first, then `UNTIL` or `COUNT` (never both), plus any `BYxxx`; `WKST` defaults to MO; `UNTIL` is in UTC when `DTSTART` is UTC or zoned. The recurrence set is `RRULE` plus `RDATE` minus `EXDATE`; `EXDATE` wins on a tie; duplicates collapse; instances landing on an invalid date or a nonexistent local time are skipped (§3.3.10). The `rrule` crate expands the set, and a `DTSTART` with `TZID` keeps every instance at the same local time across DST.

**Per-instance change and cancel.** To change or cancel one occurrence of a series, emit a second `VEVENT` with the same `UID`, a `RECURRENCE-ID` equal to that instance's original `DTSTART`, and a higher `SEQUENCE` (§3.8.4.4). `RANGE=THISANDFUTURE` targets that instance and all later ones; the deprecated `THISANDPRIOR` is never generated (§3.2.13). Cancelling one occurrence sends that override with `STATUS:CANCELLED` under `METHOD:CANCEL`; dropping an occurrence the platform removes itself, rather than an attendee-facing cancel, uses `EXDATE`, and the original `DTSTART` is always retained (§3.8.5.1).

**Lifecycle, the exact object per operation.** This is what makes updates and cancels work end to end.
- Create and invite: `METHOD:REQUEST`, `SEQUENCE:0`, `STATUS:CONFIRMED` (or `TENTATIVE`), organizer and attendees with `RSVP`, the full `VEVENT`, its `VTIMEZONE`, and the check-in `VALARM`. iMIP packaging: `Content-Type: text/calendar; method=REQUEST; component=VEVENT; charset=UTF-8`, normally as `multipart/alternative` alongside an HTML part, with a `.ics` file attachment.
- Update and reschedule: same `UID`, `SEQUENCE` incremented, `DTSTAMP` refreshed, the changed `DTSTART`, `DTEND`, `RRULE` or location, and `METHOD:REQUEST` again. Clients replace the event when the `UID` matches and `SEQUENCE` is higher, or `SEQUENCE` is equal and `DTSTAMP` is later. A change is significant, so `SEQUENCE` increments, when start, end, recurrence or another scheduling-material field changes; a cosmetic edit keeps `SEQUENCE` but still refreshes `DTSTAMP`. The precise significance rules are iTIP (RFC 5546).
- Cancel: `METHOD:CANCEL`, same `UID`, `SEQUENCE` incremented, `STATUS:CANCELLED`, attendees retained, MIME `method=CANCEL`. A whole-series cancel omits `RECURRENCE-ID`; a single occurrence carries the `RECURRENCE-ID` of that instance.
- Reply (inbound): an attendee's client sends `METHOD:REPLY` with that attendee's `PARTSTAT` (`ACCEPTED`, `DECLINED` or `TENTATIVE`) for the `UID` and `SEQUENCE` it is answering, and OpenWorkspace ingests it to update participation. A `SEQUENCE` bump is never used to request a reply; `RSVP=TRUE` on the attendee is (§3.8.7.4). The iTIP `COUNTER`, `DECLINECOUNTER` and `REFRESH` methods are recognised conceptually but out of scope for the first release.

**DTSTAMP vs LAST-MODIFIED vs SEQUENCE.** Three distinct fields, and a common interop bug when conflated. `DTSTAMP` (§3.8.7.2) is UTC and, in a method-bearing object, is the moment this object was generated, so it changes on every message. `LAST-MODIFIED` (§3.8.7.3) is UTC and tracks when the underlying booking record last changed. `SEQUENCE` (§3.8.7.4) is the revision counter and moves only on a significant organizer change.

**Validation and golden tests.** Output is checked against the RFC 5545 ABNF and an external validator, and round-tripped through the `icalendar` parser. Golden-file cases: a single timed booking; an all-day booking (DATE value, non-inclusive `DTEND`); a weekly recurring desk booking; a single-instance reschedule (`RECURRENCE-ID`); a single-instance cancel; a full-series cancel; a delegate booking (`SENT-BY`); and a series spanning a DST transition (correct `VTIMEZONE`). QA additionally confirms import, update and cancel behaviour in Outlook, Google Calendar and Apple Calendar.

## Appendix G: scheduling (iTIP, RFC 5546) and email transport (iMIP, RFC 6047)
Appendix F fixes the object format. This appendix fixes the two things it defers to: the scheduling semantics (which method is sent, what each one carries, and how a client decides a message is newer) from iTIP, and the email packaging from iMIP. OpenWorkspace is always the Organizer of a booking; the booker, and any booked rooms or assets, are Attendees. iTIP messages are `text/calendar` entities whose `METHOD` names the operation, and apart from `VTIMEZONE` only one component type appears per message. Only `VEVENT` is used; `VTODO`, `VJOURNAL` and `VFREEBUSY` scheduling are out of scope for the first release.

**Roles and methods (§1.3, §1.4).** The Organizer controls the master object and is the only party that may change it; Attendees never edit the master, they only respond. Methods by sender: the Organizer may send `PUBLISH`, `REQUEST`, `ADD`, `CANCEL` and `DECLINECOUNTER`; an Attendee may send `REPLY`, `REFRESH`, `COUNTER`, and `REQUEST` only when delegating. OpenWorkspace emits `REQUEST` and `CANCEL`, and ingests `REPLY` (and `REFRESH`). `ADD`, `COUNTER`, `DECLINECOUNTER` and `PUBLISH` are recognised but not originated in V1.

**SEQUENCE, the exact increment rule (§2.1.4).** This pins down what Appendix F left to iTIP. The Organizer MUST increment `SEQUENCE` when it changes any of `DTSTART`, `DTEND`, `DURATION`, `DUE`, `RRULE`, `RDATE`, `EXDATE` or `STATUS`. It MAY increment for other changes that jeopardise an Attendee's participation, for example a move to a distant location. It MUST increment on every `ADD` and `CANCEL`. It MUST NOT increment on `REPLY`, `REFRESH`, `COUNTER`, `DECLINECOUNTER` or a delegation `REQUEST`. Receivers SHOULD NOT treat a `SEQUENCE` bump alone as proof of a significant change; they compare the old and new objects. OpenWorkspace therefore bumps `SEQUENCE` exactly on those property changes and on cancel, refreshes `DTSTAMP` on every emission, and keeps cosmetic-only edits at the same `SEQUENCE`.

**Message correlation and out-of-order handling (§2.1.5, §4.7.2).** The primary key is `UID`, or `UID` plus `RECURRENCE-ID` for an instance. The secondary key is `SEQUENCE`: for the same `UID` and `RECURRENCE-ID`, the highest `SEQUENCE` obsoletes all lower ones. `DTSTAMP` is the tie-breaker, latest wins. A `REPLY` carries the `SEQUENCE` of the revision the Attendee answered, and the highest-`SEQUENCE`, latest-`DTSTAMP` reply per Attendee wins. OpenWorkspace persists `UID`, `RECURRENCE-ID`, `SEQUENCE` and `DTSTAMP` for every stored event, and per Attendee the `SEQUENCE` and `DTSTAMP` of their last reply, so a delayed earlier reply is discarded. An incoming object whose `SEQUENCE` is lower than the stored one is ignored; a reply or update whose `RECURRENCE-ID` cannot be matched triggers a `REFRESH`.

**What each message carries (VEVENT restriction tables, §3.2).** Presence values follow iTIP: 1 means exactly one, 1+ at least one, 0 forbidden, 0 or 1 optional.
- `REQUEST` (create, reschedule, update, reconfirm, status poll, delegate, organizer change): `METHOD=REQUEST`; one or more `VEVENT` all sharing the same `UID`; required `ATTENDEE` (1+), `DTSTAMP`, `DTSTART`, `ORGANIZER`, `SUMMARY` (may be empty), `UID`; `SEQUENCE` present when greater than 0; `DTEND` xor `DURATION`; `RECURRENCE-ID` only when addressing one instance; `STATUS` only `TENTATIVE` or `CONFIRMED`; `REQUEST-STATUS` forbidden; `VALARM` 0+; `VTIMEZONE` present whenever a time is zoned.
- `CANCEL` (whole event or instances): `METHOD=CANCEL`; one or more `VEVENT` sharing the `UID`; required `DTSTAMP`, `ORGANIZER`, `SEQUENCE`, `UID`; `ATTENDEE` is some or all of those affected; `STATUS` MUST be `CANCELLED` to cancel the whole event, and MUST be absent when only removing specific Attendees; `RECURRENCE-ID` for one instance, or `RECURRENCE-ID;RANGE=THISANDFUTURE` for an instance and all after; `VALARM` and `REQUEST-STATUS` forbidden.
- `REPLY` (ingested from Attendee clients): `METHOD=REPLY`; exactly one `ATTENDEE`, the replier, with a `PARTSTAT`; required `DTSTAMP`, `ORGANIZER`, `UID` (of the original); `SEQUENCE` equal to the revision being answered; `RECURRENCE-ID` for an instance; `REQUEST-STATUS` 0+; the optional properties MUST NOT differ from the original (a genuine change arrives as `COUNTER`); `VALARM` forbidden. OpenWorkspace maps the returned `PARTSTAT` (`ACCEPTED`, `DECLINED`, `TENTATIVE`, `DELEGATED`) onto the booking's attendee record.
- `ADD`, `REFRESH`, `COUNTER`, `DECLINECOUNTER`, `PUBLISH` (recognised, not originated in V1): `ADD` introduces new instances with `SEQUENCE` greater than 0 and forbids `RRULE`, `RDATE`, `EXDATE` and `RECURRENCE-ID`, each added instance treated as an `RDATE`; `REFRESH` (just `UID`, `ATTENDEE`, `DTSTAMP`, `ORGANIZER`) is answered with a fresh `REQUEST`, or a `CANCEL` if the event is gone; `COUNTER` and `DECLINECOUNTER` are the counter-proposal pair and echo the original `SEQUENCE`; `PUBLISH` is the one-way post that forbids `ATTENDEE`.

**Reschedule versus update (§3.2.2.1, §3.2.2.2, §3.2.2.7).** If the `UID` already exists and the incoming `SEQUENCE` (or `DTSTAMP`) is greater, the `REQUEST` is a reschedule; if the `SEQUENCE` is the same, it is a detail update, not a reschedule. The update `REQUEST` is the correct response to a `REFRESH`. A `REQUEST` that only sets `RSVP=TRUE` with no `SEQUENCE` change is a request for fresh status, not a reschedule.

**Delegation maps to the delegate feature (§2.1.3, §3.2.2.3, §3.2.2.4).** When an Attendee delegates: the delegator forwards the `REQUEST` to the delegate (adding an `ATTENDEE` for the delegate) and sends a `REPLY` to the Organizer with its own `PARTSTAT=DELEGATED`, `DELEGATED-TO` set to the delegate, plus the new delegate `ATTENDEE`; the delegate then `REPLY`s to the Organizer with `DELEGATED-FROM`. A delegator that wants to keep receiving updates sets `ROLE=NON-PARTICIPANT`. OpenWorkspace's delegate model, where a delegate books in the represented user's name, is expressed with `SENT-BY` on the `ORGANIZER` or `ATTENDEE`; responses always return to the Organizer. Replacing the organizer is a new `REQUEST` with a bumped `SEQUENCE` and a changed `ORGANIZER`.

**REQUEST-STATUS codes (§3.6).** When the property is absent, `2.0` (success) is assumed. The codes OpenWorkspace surfaces or emits: `2.x` success, possibly with a caveat; `3.7` invalid calendar user; `3.8` no authority for the operation; `3.14` unsupported capability; `4.0` event conflict, time is busy; `5.x` server or scheduling unavailable. Within one component the top-level digit of every `REQUEST-STATUS` must agree.

**Partial-implementation fallbacks (§5.1.1).** A client that cannot handle `REQUEST` degrades it to `PUBLISH`; `PUBLISH`, `REPLY`, `CANCEL` and `REFRESH` are the required baseline; `COUNTER` may be answered "not supported"; `VTIMEZONE` is required whenever a datetime is zoned; `ATTENDEE` and `ORGANIZER` are required when the method is `REQUEST`. OpenWorkspace always includes `VTIMEZONE` and both `ORGANIZER` and `ATTENDEE`, so low-fidelity clients still render the booking.

**Email packaging (iMIP, RFC 6047).**
- The body part is `text/calendar` and MUST carry a `method=` parameter equal, ignoring case, to the iCalendar `METHOD`; `charset=UTF-8` MUST be present because iCalendar defaults to UTF-8 while `text/*` defaults to US-ASCII; the optional `component=vevent` is set. A representative header is `Content-Type: text/calendar; method=REQUEST; charset=UTF-8; component=vevent`. A `text/calendar` part without `method=` is not an iMIP part (§2.4).
- `ORGANIZER` and `ATTENDEE` values MUST be `mailto:` URIs, and the real organizer or attendee MUST be read from the body, never inferred from the `From`, `Sender` or `Reply-To` header (§2.3).
- Packaging (§2.4): a human-readable `text/plain` or HTML alternative goes in `multipart/alternative`, which MUST NOT be used to carry two slightly different objects; multiple objects with different methods require `multipart/mixed`, each in its own `text/calendar` part; an `ATTACH` by `CID` uses `multipart/related`. A receiver MUST process `text/calendar` parts nested in any `multipart/*`.
- Content-Transfer-Encoding (§2.5): `7bit` only when the transport is 8-bit clean, as SMTP is, otherwise `quoted-printable` or `base64` for any non-ASCII content. Content-Disposition (§2.6) MAY set `filename=event.ics`, but handling keys on `Content-Type`, not the file extension.
- OpenWorkspace sends booking mail as `multipart/alternative` (HTML plus plain text) carrying the `text/calendar` part and an `event.ics` attachment, charset UTF-8, with quoted-printable or base64 applied when needed, and includes any `CID`-referenced part in the same MIME entity (§5.1).

**Security (iTIP §6, iMIP §2.2, §3).** The threats are spoofing the Organizer, spoofing an Attendee, unauthorized replacement of the Organizer, calendar flooding, and unauthorized `REFRESH`. The rules OpenWorkspace follows: only the Organizer may modify or cancel an event, and only the respondent may set its own `PARTSTAT`, so a spoofed `REPLY` purporting to answer for another attendee is ignored. iMIP authentication and confidentiality use S/MIME (RFC 5750 and RFC 5751), and a compliant implementation MUST support signing and encrypting the `text/calendar` part. Processing a signed message follows iMIP's steps: identify the signer from the certificate `rfc822Name`, correlate it to the `ATTENDEE` or `ORGANIZER` for that method, check that party is authorised, then process; otherwise warn and ignore. If a sender previously sent signed mail and an unsigned message later arrives, warn about a possible man-in-the-middle and ignore unless the user overrides. There is no protocol mechanism to verify `SENT-BY` authorisation, so the platform gates delegate sending itself; this is already covered because a delegate is evaluated against the principal's permissions and the action is audited. Within a single mail domain, SMTP STARTTLS MAY substitute for S/MIME confidentiality. Signing and encryption sit in `crates/crypto`, which is where the crypto-agility and post-quantum work lands. For V1, outbound booking mail is signed where an S/MIME identity is configured and is always sent over TLS, and inbound `REPLY` ingestion authenticates the signer and rejects unauthenticated changes.

## Appendix H: Data model
This is the complete logical schema. Migrations live in the `db` crate (sqlx, reversible) and the types map onto sqlx with `chrono`, `PgRange<DateTime<Utc>>` for `tstzrange`, and `uuid`. It extends and is consistent with Section 6.3. The schema targets PostgreSQL 18 and uses `uuidv7()` primary keys, `timestamptz` throughout (all times stored in UTC), and `citext` for case-insensitive natural keys. A security and integrity audit follows the tables.

**Conventions and extensions.** Every table has `id uuid PRIMARY KEY DEFAULT uuidv7()` unless noted, plus `created_at timestamptz NOT NULL DEFAULT now()` and, where mutable, `updated_at timestamptz NOT NULL DEFAULT now()` maintained by a shared `set_updated_at()` trigger. Encrypted columns are `bytea` holding a versioned AEAD envelope (suite id, key id, nonce, ciphertext, tag) decrypted through the `crypto` facade; the wrapping keys live in `crypto_keys`. Hashed secrets are stored as the hash only.
```sql
CREATE EXTENSION IF NOT EXISTS btree_gist;   -- GiST equality for the booking exclusion constraint
CREATE EXTENSION IF NOT EXISTS pg_trgm;      -- trigram search on names
CREATE EXTENSION IF NOT EXISTS citext;       -- case-insensitive email, slugs, keys

CREATE TYPE location_kind   AS ENUM ('continent','country','city','campus','building','floor');
CREATE TYPE resource_kind   AS ENUM ('desk','room','parking','vehicle','equipment');
CREATE TYPE resource_status AS ENUM ('active','inactive','maintenance');
CREATE TYPE booking_status  AS ENUM ('booked','checked_in','checked_out','cancelled','no_show');
CREATE TYPE segmentation_mode AS ENUM ('open','by_organization','by_organization_and_team');
CREATE TYPE asset_kind      AS ENUM ('reference_image','campus_map','floor_background','object_photo','logo','export');
CREATE TYPE outbox_status   AS ENUM ('pending','sent','failed','cancelled');
CREATE TYPE actor_kind      AS ENUM ('user','api_key','system');
CREATE TYPE dsr_kind        AS ENUM ('export','erasure','rectification');
CREATE TYPE dsr_status      AS ENUM ('received','in_progress','completed','rejected');
CREATE TYPE token_kind      AS ENUM ('invitation','password_reset','email_verification');
CREATE TYPE import_kind     AS ENUM ('users','resources');
CREATE TYPE import_status   AS ENUM ('pending','processing','completed','failed');
```

**Identity and credentials.** Users hold no secret material directly; passwords, passkeys and TOTP live in dedicated tables so that an account can exist with any subset (a federated or passkey-only user has no password row). The WebAuthn user handle is a random opaque value, never the email, so authenticators never carry personal data.
```sql
CREATE TABLE users (
  id                 uuid PRIMARY KEY DEFAULT uuidv7(),
  email              citext UNIQUE NOT NULL,
  display_name       text NOT NULL,
  status             text NOT NULL DEFAULT 'active'
                       CHECK (status IN ('active','suspended','deactivated')),
  locale             text NOT NULL DEFAULT 'en',
  timezone           text NOT NULL DEFAULT 'Europe/Amsterdam',
  default_view       text NOT NULL DEFAULT 'map'
                       CHECK (default_view IN ('map','list','calendar')),
  home_zone_id       uuid REFERENCES locations(id) ON DELETE SET NULL,
  default_category_id uuid REFERENCES resource_categories(id) ON DELETE SET NULL,
  webauthn_user_handle bytea UNIQUE NOT NULL,        -- random, opaque, not the email
  is_instance_admin  boolean NOT NULL DEFAULT false, -- the single top-level role; all else via roles and grants
  email_verified_at  timestamptz,
  last_login_at      timestamptz,
  failed_login_count integer NOT NULL DEFAULT 0,      -- reset on success; drives lockout
  locked_until       timestamptz,                     -- set by the lockout policy after repeated failures
  invited_by         uuid REFERENCES users(id) ON DELETE SET NULL,
  notification_prefs jsonb NOT NULL DEFAULT '{}'::jsonb,  -- which emails the user wants
  anonymized_at      timestamptz,                    -- set by GDPR erasure (Article 17)
  created_at         timestamptz NOT NULL DEFAULT now(),
  updated_at         timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE password_credentials (             -- present only for local-password accounts
  user_id            uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  password_hash      text NOT NULL,             -- Argon2id PHC string (algorithm, params, salt, hash)
  must_change        boolean NOT NULL DEFAULT false,
  password_changed_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE oidc_identities (                  -- federated logins (JIT provisioning)
  id                 uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id            uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider_id        uuid NOT NULL REFERENCES oidc_providers(id) ON DELETE RESTRICT,
  subject            text NOT NULL,             -- the IdP "sub" claim
  email_at_link      citext,
  last_login_at      timestamptz,
  created_at         timestamptz NOT NULL DEFAULT now(),
  UNIQUE (provider_id, subject)
);

CREATE TABLE passkeys (                          -- WebAuthn credentials (webauthn-rs Passkey)
  id                 uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id            uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  credential_id      bytea UNIQUE NOT NULL,      -- globally unique across all accounts
  passkey            jsonb NOT NULL,             -- serialized webauthn-rs Passkey (COSE key, flags)
  sign_count         bigint NOT NULL DEFAULT 0,  -- updated post-auth; detects cloned authenticators
  aaguid             uuid,
  transports         text[],
  label              text,                       -- user-given name for the credential
  backup_eligible    boolean NOT NULL DEFAULT false,
  backup_state       boolean NOT NULL DEFAULT false,
  created_at         timestamptz NOT NULL DEFAULT now(),
  last_used_at       timestamptz
);
CREATE INDEX passkeys_user_idx ON passkeys (user_id);

CREATE TABLE totp_credentials (
  user_id            uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  secret_encrypted   bytea NOT NULL,             -- AEAD envelope, never plaintext
  digits             smallint NOT NULL DEFAULT 6,
  period_seconds     smallint NOT NULL DEFAULT 30,
  algorithm          text NOT NULL DEFAULT 'SHA1',
  confirmed_at       timestamptz,
  created_at         timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE recovery_codes (                    -- MFA fallback, single use
  id                 uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id            uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  code_hash          bytea NOT NULL,             -- hash only
  used_at            timestamptz,
  created_at         timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX recovery_codes_user_idx ON recovery_codes (user_id);

CREATE TABLE user_tokens (                       -- one-time tokens: invite, password reset, email verification
  id          uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  kind        token_kind NOT NULL,
  token_hash  bytea UNIQUE NOT NULL,              -- hash only; the raw token is emailed once
  expires_at  timestamptz NOT NULL,
  used_at     timestamptz,
  created_by  uuid REFERENCES users(id) ON DELETE SET NULL,  -- the inviting admin, when applicable
  created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX user_tokens_user_idx ON user_tokens (user_id);

-- Sessions are owned by tower-sessions (PostgresStore): table tower_sessions.session
-- (id text PK, data bytea, expiry_date timestamptz). The session holds auth state, the
-- CSRF token, and transient WebAuthn ceremony state (PasskeyRegistration / PasskeyAuthentication),
-- which MUST be kept server-side to prevent replay. The session id is the only client-held secret.

CREATE TABLE api_keys (
  id                 uuid PRIMARY KEY DEFAULT uuidv7(),
  owner_user_id      uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name               text NOT NULL,
  prefix             text NOT NULL,             -- short, displayed, e.g. "owk_ab12"
  token_hash         bytea UNIQUE NOT NULL,     -- SHA-256 of the full key (high entropy, fast hash is fine)
  scopes             text[] NOT NULL DEFAULT '{}',
  expires_at         timestamptz,
  last_used_at       timestamptz,
  last_used_ip       inet,
  rate_limit_per_minute integer,                  -- null = the instance default
  revoked_at         timestamptz,
  revoked_by         uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at         timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX api_keys_owner_idx  ON api_keys (owner_user_id);
CREATE INDEX api_keys_prefix_idx ON api_keys (prefix);
```

**Organizations, teams, roles, grants.** Roles carry a permission set; authorization is the union of the instance-admin flag, organization or team membership roles, location-scoped grants, and the booking delegate relationship. The `role_grants` and `booking_delegates` subjects use explicit nullable foreign keys with a `num_nonnulls` check rather than an un-referenceable polymorphic column.
```sql
CREATE TABLE organizations (
  id     uuid PRIMARY KEY DEFAULT uuidv7(),
  name   text NOT NULL,
  slug   citext UNIQUE NOT NULL,
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','archived')),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE teams (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  name            text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (organization_id, name)
);

CREATE TABLE roles (
  id          uuid PRIMARY KEY DEFAULT uuidv7(),
  key         citext UNIQUE NOT NULL,            -- owner, admin, member, or a custom key
  name        text NOT NULL,
  description text,
  is_system   boolean NOT NULL DEFAULT false,    -- system roles cannot be deleted
  created_at  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE role_permissions (
  role_id    uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  permission text NOT NULL,                      -- from the fixed application catalog
  PRIMARY KEY (role_id, permission)
);

CREATE TABLE memberships (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id         uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  team_id         uuid REFERENCES teams(id) ON DELETE CASCADE,   -- null = organization-level
  role_id         uuid NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
  created_at      timestamptz NOT NULL DEFAULT now()
);
-- one organization-level row and one row per team per user (nulls are distinct in a plain UNIQUE,
-- so enforce with partial unique indexes):
CREATE UNIQUE INDEX memberships_org_unique  ON memberships (user_id, organization_id)
  WHERE team_id IS NULL;
CREATE UNIQUE INDEX memberships_team_unique ON memberships (user_id, team_id)
  WHERE team_id IS NOT NULL;

CREATE TABLE role_grants (                       -- location-scoped, delegated per-node administration
  id               uuid PRIMARY KEY DEFAULT uuidv7(),
  subject_user_id  uuid REFERENCES users(id) ON DELETE CASCADE,
  subject_team_id  uuid REFERENCES teams(id) ON DELETE CASCADE,
  role_id          uuid NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
  location_id      uuid NOT NULL REFERENCES locations(id) ON DELETE CASCADE,  -- applies here and below
  valid_from       timestamptz NOT NULL DEFAULT now(),
  valid_to         timestamptz,
  created_by        uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at       timestamptz NOT NULL DEFAULT now(),
  CHECK (num_nonnulls(subject_user_id, subject_team_id) = 1)   -- exactly one subject
);
CREATE INDEX role_grants_location_idx ON role_grants (location_id);

CREATE TABLE booking_delegates (                 -- the Stellvertreter relationship
  id               uuid PRIMARY KEY DEFAULT uuidv7(),
  principal_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  delegate_user_id  uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  valid_from       timestamptz NOT NULL DEFAULT now(),
  valid_to         timestamptz,
  created_by        uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at       timestamptz NOT NULL DEFAULT now(),
  CHECK (principal_user_id <> delegate_user_id)
);
CREATE INDEX booking_delegates_principal_idx ON booking_delegates (principal_user_id);
CREATE INDEX booking_delegates_delegate_idx  ON booking_delegates (delegate_user_id);
```

**Location hierarchy, floor plans and assets.** Locations are one adjacency-list tree with a materialized `path` (maintained by trigger) for fast subtree queries, so a grant on a node covers all descendants without a recursive scan on every check. Kind-specific fields (the campus map image, a building marker) live on the location as the document specifies, guarded by checks so they only appear on the right kind. Uploaded files are raster images only, re-encoded to strip metadata; no SVG is ever stored, which removes the script-injection surface and the need for a sanitiser.
```sql
CREATE TABLE assets (
  id            uuid PRIMARY KEY DEFAULT uuidv7(),
  kind          asset_kind NOT NULL,
  storage_key   text NOT NULL,                  -- object_store key (presigned URLs generated on demand)
  content_type  text NOT NULL,
  byte_size     bigint NOT NULL,
  width         integer,
  height        integer,
  checksum      bytea,
  original_filename text,
  alt_text      text,                            -- accessibility text for image assets
  parent_asset_id uuid REFERENCES assets(id) ON DELETE CASCADE,  -- derived variants (thumbnails)
  variant       text,                           -- e.g. 'thumb'
  uploaded_by   uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at    timestamptz NOT NULL DEFAULT now(),
  CHECK (kind = 'export' OR content_type IN ('image/png','image/jpeg','image/webp','image/avif'))
);

CREATE TABLE locations (
  id                 uuid PRIMARY KEY DEFAULT uuidv7(),
  parent_id          uuid REFERENCES locations(id) ON DELETE RESTRICT,  -- archive, never orphan
  kind               location_kind NOT NULL,
  name               text NOT NULL,
  slug               citext,
  path               text NOT NULL,             -- materialized '/ancestor/.../self/', trigger-kept
  depth              integer NOT NULL,
  sort_order         integer NOT NULL DEFAULT 0,
  code               text,                       -- external identifier, e.g. 'BLD-A' or 'L3'
  timezone           text,                       -- IANA zone for this node; inherits down to resources
  status             text NOT NULL DEFAULT 'active' CHECK (status IN ('active','archived')),
  archived_at        timestamptz,
  address            text,                       -- campus or building postal address
  latitude           numeric(9,6),               -- optional, for external maps and directions
  longitude          numeric(9,6),
  organization_id    uuid REFERENCES organizations(id) ON DELETE SET NULL,  -- default org for the subtree
  map_image_asset_id uuid REFERENCES assets(id) ON DELETE SET NULL,         -- campus only
  marker_x           numeric(6,5),              -- building only, fraction 0..1 on the parent campus map
  marker_y           numeric(6,5),
  created_at         timestamptz NOT NULL DEFAULT now(),
  updated_at         timestamptz NOT NULL DEFAULT now(),
  CHECK (map_image_asset_id IS NULL OR kind = 'campus'),
  CHECK ((marker_x IS NULL AND marker_y IS NULL) OR kind = 'building'),
  CHECK (marker_x IS NULL OR (marker_x BETWEEN 0 AND 1 AND marker_y BETWEEN 0 AND 1))
);
CREATE INDEX locations_parent_idx ON locations (parent_id);
CREATE INDEX locations_path_idx   ON locations (path text_pattern_ops);  -- subtree via path LIKE '/a/b/%'
CREATE INDEX locations_kind_idx   ON locations (kind);

CREATE TABLE floor_plans (                       -- one per floor location
  floor_id          uuid PRIMARY KEY REFERENCES locations(id) ON DELETE CASCADE,
  scene             jsonb NOT NULL DEFAULT '{}'::jsonb,   -- placed component instances
  background_asset_id uuid REFERENCES assets(id) ON DELETE SET NULL,  -- optional raster under the SVG
  viewbox           text,
  status            text NOT NULL DEFAULT 'published' CHECK (status IN ('draft','published')),
  published_at      timestamptz,
  version           integer NOT NULL DEFAULT 1,
  updated_by        uuid REFERENCES users(id) ON DELETE SET NULL,
  updated_at        timestamptz NOT NULL DEFAULT now()
);
```

**Resources.** A resource is a bookable object on a floor; its placement is normalised into `resource_positions` so a desk can be moved or rebound without rewriting the scene. Categories and tags drive filtering and search; rules constrain how a resource can be booked.
```sql
CREATE TABLE resource_categories (
  id    uuid PRIMARY KEY DEFAULT uuidv7(),
  name  text NOT NULL,
  color text,
  icon  text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE resources (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  floor_id        uuid NOT NULL REFERENCES locations(id) ON DELETE RESTRICT,
  kind            resource_kind NOT NULL,
  name            text NOT NULL,
  code            text,                          -- human label, unique per floor
  category_id     uuid REFERENCES resource_categories(id) ON DELETE SET NULL,
  organization_id uuid REFERENCES organizations(id) ON DELETE SET NULL,  -- null inherits the location
  team_id         uuid REFERENCES teams(id) ON DELETE SET NULL,
  description     text,
  photo_asset_id  uuid REFERENCES assets(id) ON DELETE SET NULL,  -- the object photo
  capacity        integer,                       -- rooms
  status          resource_status NOT NULL DEFAULT 'active',
  bookable        boolean NOT NULL DEFAULT true,
  requires_checkin boolean NOT NULL DEFAULT true,
  attributes      jsonb NOT NULL DEFAULT '{}'::jsonb,   -- equipment specifics
  created_at      timestamptz NOT NULL DEFAULT now(),
  updated_at      timestamptz NOT NULL DEFAULT now(),
  archived_at     timestamptz,
  UNIQUE (floor_id, code)
);
CREATE INDEX resources_floor_idx    ON resources (floor_id);
CREATE INDEX resources_category_idx ON resources (category_id);
CREATE INDEX resources_name_trgm    ON resources USING gin (name gin_trgm_ops);  -- type-ahead search

CREATE TABLE resource_positions (                -- resource_positions(resource_id, floor_id, scene_node_id)
  id            uuid PRIMARY KEY DEFAULT uuidv7(),
  resource_id   uuid NOT NULL UNIQUE REFERENCES resources(id) ON DELETE CASCADE,
  floor_id      uuid NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
  scene_node_id text NOT NULL,
  created_at    timestamptz NOT NULL DEFAULT now(),
  UNIQUE (floor_id, scene_node_id)               -- one resource per scene node
);

CREATE TABLE resource_tags (
  id   uuid PRIMARY KEY DEFAULT uuidv7(),
  name citext UNIQUE NOT NULL
);
CREATE TABLE resource_tag_map (
  resource_id uuid NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
  tag_id      uuid NOT NULL REFERENCES resource_tags(id) ON DELETE CASCADE,
  PRIMARY KEY (resource_id, tag_id)
);

CREATE TABLE resource_rules (                    -- booking policy for a resource (V1 scope)
  id                  uuid PRIMARY KEY DEFAULT uuidv7(),
  resource_id         uuid NOT NULL UNIQUE REFERENCES resources(id) ON DELETE CASCADE,
  max_advance_days    integer,                   -- furthest ahead a booking may start
  min_advance_minutes integer,                   -- lead time; cannot book closer than this
  min_duration_minutes integer,
  max_duration_minutes integer,
  slot_granularity_minutes integer,              -- snap start and end to this grid
  buffer_minutes      integer,                   -- gap enforced between bookings (e.g. room turnover)
  max_per_user_per_day integer,
  max_active_per_user integer,                   -- cap on concurrent future bookings
  cancellation_deadline_minutes integer,         -- cannot cancel within this window of the start
  allow_recurrence    boolean NOT NULL DEFAULT true,
  require_approval    boolean NOT NULL DEFAULT false,
  allowed_window      jsonb,                     -- opening hours, weekday rules
  created_at          timestamptz NOT NULL DEFAULT now(),
  updated_at          timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE permanent_assignments (             -- a resource assigned to a user for a period
  id          uuid PRIMARY KEY DEFAULT uuidv7(),
  resource_id uuid NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
  user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  validity    daterange NOT NULL DEFAULT daterange(CURRENT_DATE, NULL, '[)'),
  created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at  timestamptz NOT NULL DEFAULT now(),
  EXCLUDE USING gist (resource_id WITH =, validity WITH &&)   -- no overlapping assignment per resource
);

CREATE TABLE blackouts (                          -- maintenance or closure, on a resource or a node
  id          uuid PRIMARY KEY DEFAULT uuidv7(),
  resource_id uuid REFERENCES resources(id) ON DELETE CASCADE,
  location_id uuid REFERENCES locations(id) ON DELETE CASCADE,
  title       text,
  period      tstzrange NOT NULL,
  recurrence_rule text,                          -- optional RRULE for repeating maintenance
  reason      text,
  created_by  uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at  timestamptz NOT NULL DEFAULT now(),
  CHECK (num_nonnulls(resource_id, location_id) = 1),
  CHECK (NOT isempty(period) AND lower(period) IS NOT NULL AND upper(period) IS NOT NULL)
);
```

**Bookings.** A booking is the master record (a single reservation or a recurring series with an RFC 5545 `RRULE`); concrete instances are materialised into `booking_occurrences`, which is what actually occupies a resource in time and therefore carries the overlap guarantee. The iCalendar identity fields live on the master so updates and cancellations stay stable (Appendices F and G).
```sql
CREATE TABLE bookings (
  id                 uuid PRIMARY KEY DEFAULT uuidv7(),
  resource_id        uuid NOT NULL REFERENCES resources(id) ON DELETE RESTRICT,
  booked_for_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,  -- the principal
  booked_by_user_id  uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,  -- actor (delegate or self)
  title              text,
  description        text,
  source             text NOT NULL DEFAULT 'web'
                       CHECK (source IN ('web','api','import','delegate')),
  ical_uid           text UNIQUE NOT NULL,       -- stable across update and cancel
  sequence           integer NOT NULL DEFAULT 0, -- iTIP revision (Appendix G)
  dtstamp            timestamptz NOT NULL DEFAULT now(),
  recurrence_rule    text,                       -- RRULE; null for a single booking
  recurrence_until   timestamptz,
  status             booking_status NOT NULL DEFAULT 'booked',
  cancelled_at       timestamptz,
  cancelled_by       uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at         timestamptz NOT NULL DEFAULT now(),
  updated_at         timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX bookings_for_idx      ON bookings (booked_for_user_id);
CREATE INDEX bookings_resource_idx ON bookings (resource_id);

CREATE TABLE booking_occurrences (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  booking_id      uuid NOT NULL REFERENCES bookings(id) ON DELETE CASCADE,
  resource_id     uuid NOT NULL REFERENCES resources(id) ON DELETE RESTRICT,  -- local, for the constraint
  period          tstzrange NOT NULL,            -- half-open [start, end)
  recurrence_id   timestamptz,                   -- the instance's original start, for overrides and cancels
  is_override     boolean NOT NULL DEFAULT false,
  status          booking_status NOT NULL DEFAULT 'booked',
  check_in_at     timestamptz,
  checked_in_by   uuid REFERENCES users(id) ON DELETE SET NULL,
  checked_out_at  timestamptz,
  auto_released_at timestamptz,
  cancelled_at    timestamptz,
  cancellation_reason text,
  created_at      timestamptz NOT NULL DEFAULT now(),
  updated_at      timestamptz NOT NULL DEFAULT now(),
  CHECK (NOT isempty(period) AND lower(period) IS NOT NULL AND upper(period) IS NOT NULL),
  -- the no-double-booking guarantee (Section 6.3); needs btree_gist:
  EXCLUDE USING gist (resource_id WITH =, period WITH &&)
    WHERE (status IN ('booked','checked_in'))
);
CREATE INDEX booking_occurrences_booking_idx ON booking_occurrences (booking_id);
-- the worker's sweeps (auto-release no-shows, check-in reminders) hit only live rows:
CREATE INDEX booking_occurrences_live_idx ON booking_occurrences (period)
  WHERE status IN ('booked','checked_in');
```

**Notifications.** Mail is queued in an outbox with a unique idempotency key so a retry never double-sends; the worker dispatches pending rows every minute. Admin-editable content overrides the built-in templates per locale.
```sql
CREATE TABLE email_outbox (
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  to_address      citext NOT NULL,
  to_user_id      uuid REFERENCES users(id) ON DELETE SET NULL,
  cc              text,
  reply_to        text,                          -- defaults to the instance reply-to
  message_id      text,                          -- the email Message-ID, for iMIP threading
  template_key    text NOT NULL,
  locale          text NOT NULL DEFAULT 'en',
  subject         text NOT NULL,
  body_html       text NOT NULL,
  body_text       text NOT NULL,
  ics_body        text,                          -- the rendered .ics, when applicable
  related_booking_id uuid REFERENCES bookings(id) ON DELETE SET NULL,
  idempotency_key text UNIQUE NOT NULL,
  status          outbox_status NOT NULL DEFAULT 'pending',
  attempts        integer NOT NULL DEFAULT 0,
  last_error      text,
  scheduled_for   timestamptz NOT NULL DEFAULT now(),
  sent_at         timestamptz,
  created_at      timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX email_outbox_pending_idx ON email_outbox (scheduled_for)
  WHERE status = 'pending';

CREATE TABLE mail_templates (                    -- overrides the built-in askama defaults
  id               uuid PRIMARY KEY DEFAULT uuidv7(),
  key              citext NOT NULL,
  locale           text NOT NULL DEFAULT 'en',
  subject_template text NOT NULL,
  html_template    text NOT NULL,
  text_template    text NOT NULL,
  enabled          boolean NOT NULL DEFAULT true,
  updated_by       uuid REFERENCES users(id) ON DELETE SET NULL,
  updated_at       timestamptz NOT NULL DEFAULT now(),
  UNIQUE (key, locale)
);
```

**Instance settings, OIDC providers, crypto keys, data-subject requests.** A singleton settings row holds deployment-wide policy. Provider client secrets and other secrets are encrypted through the `crypto` facade, whose wrapping keys are tracked in `crypto_keys` so the deployment can rotate keys without rewriting ciphertext shapes.
```sql
CREATE TABLE instance_settings (
  id                    boolean PRIMARY KEY DEFAULT true CHECK (id),   -- single row
  segmentation_mode     segmentation_mode NOT NULL DEFAULT 'open',
  default_locale        text NOT NULL DEFAULT 'en',
  default_timezone      text NOT NULL DEFAULT 'Europe/Amsterdam',
  product_name          text NOT NULL DEFAULT 'OpenWorkspace',
  logo_asset_id         uuid REFERENCES assets(id) ON DELETE SET NULL,
  primary_color         text,
  checkin_window_minutes integer NOT NULL DEFAULT 15,
  checkin_grace_minutes  integer NOT NULL DEFAULT 15,
  booking_horizon_days   integer NOT NULL DEFAULT 90,
  audit_retention_days   integer NOT NULL DEFAULT 365,
  booking_retention_days integer NOT NULL DEFAULT 730,
  -- default booking policy, overridden per resource in resource_rules
  default_slot_granularity_minutes integer NOT NULL DEFAULT 30,
  default_max_advance_days integer NOT NULL DEFAULT 90,
  -- authentication policy
  local_login_enabled    boolean NOT NULL DEFAULT true,
  passkeys_enabled       boolean NOT NULL DEFAULT true,
  totp_enabled           boolean NOT NULL DEFAULT true,
  allow_self_registration boolean NOT NULL DEFAULT false,
  require_mfa            boolean NOT NULL DEFAULT false,
  min_password_length    integer NOT NULL DEFAULT 12,   -- length over composition, breach-list checked
  session_idle_minutes   integer NOT NULL DEFAULT 480,
  session_absolute_hours integer NOT NULL DEFAULT 24,
  lockout_threshold      integer NOT NULL DEFAULT 10,   -- failed logins before lock
  lockout_minutes        integer NOT NULL DEFAULT 15,
  -- outbound email (SMTP); the password is held as an AEAD envelope
  smtp_host             text,
  smtp_port             integer,
  smtp_username         text,
  smtp_password_encrypted bytea,
  smtp_use_starttls     boolean NOT NULL DEFAULT true,
  smtp_from_address     text,
  smtp_reply_to         text,
  updated_at            timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE oidc_providers (
  id                   uuid PRIMARY KEY DEFAULT uuidv7(),

  -- identity and discovery
  display_name         text NOT NULL,
  slug                 citext UNIQUE NOT NULL,           -- stable key in the callback path /auth/{slug}/callback
  issuer_url           text NOT NULL,
  use_discovery        boolean NOT NULL DEFAULT true,    -- read .well-known/openid-configuration
  authorization_endpoint text,                           -- manual overrides, used when discovery is off
  token_endpoint       text,
  userinfo_endpoint    text,
  jwks_uri             text,
  end_session_endpoint text,
  metadata_cache_seconds integer NOT NULL DEFAULT 3600,  -- discovery and JWKS cache lifetime

  -- client authentication to the token endpoint
  client_id            text NOT NULL,
  client_auth_method   text NOT NULL DEFAULT 'client_secret_basic'
                         CHECK (client_auth_method IN
                           ('client_secret_basic','client_secret_post','client_secret_jwt','private_key_jwt','none')),
  client_secret_encrypted bytea,                         -- AEAD envelope, for the client_secret_* methods
  client_assertion_key_id uuid REFERENCES crypto_keys(id) ON DELETE RESTRICT,  -- for private_key_jwt

  -- request shape; Authorization Code only, and PKCE S256, state and nonce are always enforced, never settings
  scopes               text[] NOT NULL DEFAULT ARRAY['openid','email','profile'],
  response_mode        text NOT NULL DEFAULT 'query' CHECK (response_mode IN ('query','form_post')),
  prompt               text,                             -- e.g. 'select_account', 'login'
  acr_values           text[],                           -- request an authentication context, e.g. MFA at the IdP
  max_age_seconds      integer,                          -- force re-authentication past this session age

  -- ID-token validation
  id_token_signed_response_alg text[] NOT NULL DEFAULT ARRAY['RS256'],  -- allowlist; 'none' is never accepted
  clock_skew_seconds   integer NOT NULL DEFAULT 60,

  -- claim mapping; the federated key is always issuer + sub and is not configurable
  email_claim          text NOT NULL DEFAULT 'email',
  email_verified_claim text NOT NULL DEFAULT 'email_verified',
  name_claim           text NOT NULL DEFAULT 'name',
  username_claim       text NOT NULL DEFAULT 'preferred_username',
  groups_claim         text,                             -- source for role mapping, e.g. 'groups' or 'roles'

  -- provisioning and account lifecycle
  enabled              boolean NOT NULL DEFAULT true,
  jit_provisioning     boolean NOT NULL DEFAULT true,
  default_role_id      uuid REFERENCES roles(id) ON DELETE SET NULL,
  default_organization_id uuid REFERENCES organizations(id) ON DELETE SET NULL,
  allowed_email_domains text[] NOT NULL DEFAULT '{}',    -- empty = any; else only these domains may sign in
  account_linking      text NOT NULL DEFAULT 'verified_email'
                         CHECK (account_linking IN ('disabled','verified_email')),  -- never link an unverified email
  update_profile_on_login boolean NOT NULL DEFAULT true,
  sync_roles_on_login  boolean NOT NULL DEFAULT true,    -- reapply the group mappings below on each login

  -- logout and login-page presentation
  rp_initiated_logout  boolean NOT NULL DEFAULT true,
  button_label         text,
  icon                 text,
  sort_order           integer NOT NULL DEFAULT 0,

  created_at           timestamptz NOT NULL DEFAULT now(),
  updated_at           timestamptz NOT NULL DEFAULT now(),

  CHECK (use_discovery
         OR (authorization_endpoint IS NOT NULL AND token_endpoint IS NOT NULL AND jwks_uri IS NOT NULL)),
  CHECK ((client_auth_method LIKE 'client_secret_%' AND client_secret_encrypted IS NOT NULL)
      OR (client_auth_method = 'private_key_jwt'    AND client_assertion_key_id IS NOT NULL)
      OR (client_auth_method = 'none'))
);

CREATE TABLE oidc_role_mappings (                 -- map a value from groups_claim onto an internal role
  id              uuid PRIMARY KEY DEFAULT uuidv7(),
  provider_id     uuid NOT NULL REFERENCES oidc_providers(id) ON DELETE CASCADE,
  external_value  text NOT NULL,                  -- a group or role value asserted by the IdP
  role_id         uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  organization_id uuid REFERENCES organizations(id) ON DELETE CASCADE,  -- optional scope for the granted role
  team_id         uuid REFERENCES teams(id) ON DELETE CASCADE,
  created_at      timestamptz NOT NULL DEFAULT now(),
  UNIQUE (provider_id, external_value, role_id, organization_id, team_id)
);
CREATE INDEX oidc_role_mappings_provider_idx ON oidc_role_mappings (provider_id);

CREATE TABLE crypto_keys (                        -- wrapped data-encryption keys for crypto-agility
  id          uuid PRIMARY KEY DEFAULT uuidv7(),
  purpose     text NOT NULL,                     -- e.g. 'field-encryption', 'imip-signing'
  suite_id    text NOT NULL,                     -- versioned algorithm identifier (see Section 6.7)
  wrapped_key bytea NOT NULL,                    -- DEK wrapped by the KEK from config or a KMS
  kek_label   text,                              -- which key-encryption key wrapped this DEK
  active      boolean NOT NULL DEFAULT true,
  created_at  timestamptz NOT NULL DEFAULT now(),
  retired_at  timestamptz
);
CREATE UNIQUE INDEX crypto_keys_active_purpose ON crypto_keys (purpose) WHERE active;  -- one active key per purpose

CREATE TABLE data_subject_requests (             -- GDPR Articles 15 and 17 tracking
  id            uuid PRIMARY KEY DEFAULT uuidv7(),
  user_id       uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  kind          dsr_kind NOT NULL,
  status        dsr_status NOT NULL DEFAULT 'received',
  requested_by  uuid REFERENCES users(id) ON DELETE SET NULL,
  export_asset_id uuid REFERENCES assets(id) ON DELETE SET NULL,
  notes         text,
  identity_verified boolean NOT NULL DEFAULT false,
  requested_at  timestamptz NOT NULL DEFAULT now(),
  due_at        timestamptz,                     -- statutory deadline (e.g. one month)
  completed_at  timestamptz
);

CREATE TABLE import_jobs (                        -- bulk CSV import of users or resources
  id            uuid PRIMARY KEY DEFAULT uuidv7(),
  kind          import_kind NOT NULL,
  status        import_status NOT NULL DEFAULT 'pending',
  source_asset_id uuid REFERENCES assets(id) ON DELETE SET NULL,  -- the uploaded CSV
  total_rows    integer,
  processed_rows integer NOT NULL DEFAULT 0,
  error_rows    integer NOT NULL DEFAULT 0,
  errors        jsonb NOT NULL DEFAULT '[]'::jsonb,   -- per-row failures
  created_by    uuid REFERENCES users(id) ON DELETE SET NULL,
  created_at    timestamptz NOT NULL DEFAULT now(),
  completed_at  timestamptz
);

CREATE TABLE favorite_resources (                -- a user's saved resources (Gingco parity)
  user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  resource_id uuid NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
  created_at  timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, resource_id)
);
```

**Append-only audit log.** Every privileged action is recorded, including the delegate case (actor and on-behalf-of), and the log is immutable: a trigger forbids update and delete, the runtime database role is not granted those rights, and an optional hash chain makes tampering detectable.
```sql
CREATE TABLE audit_log (
  id               uuid PRIMARY KEY DEFAULT uuidv7(),
  occurred_at      timestamptz NOT NULL DEFAULT now(),
  actor_kind       actor_kind NOT NULL,
  actor_user_id    uuid REFERENCES users(id) ON DELETE RESTRICT,
  on_behalf_of_user_id uuid REFERENCES users(id) ON DELETE RESTRICT,   -- delegate actions
  api_key_id       uuid REFERENCES api_keys(id) ON DELETE SET NULL,
  action           text NOT NULL,
  outcome          text NOT NULL DEFAULT 'success' CHECK (outcome IN ('success','failure','denied')),
  target_type      text,                          -- intentionally not a foreign key (see audit)
  target_id        uuid,
  ip               inet,
  user_agent       text,
  request_id       uuid,
  metadata         jsonb NOT NULL DEFAULT '{}'::jsonb,
  prev_hash        bytea,                         -- optional tamper-evidence chain
  entry_hash       bytea
);
CREATE INDEX audit_log_time_brin  ON audit_log USING brin (occurred_at);  -- append-only, time-ordered
CREATE INDEX audit_log_actor_idx  ON audit_log (actor_user_id);
CREATE INDEX audit_log_target_idx ON audit_log (target_type, target_id);

CREATE FUNCTION audit_log_immutable() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'audit_log is append-only';
END $$;
CREATE TRIGGER audit_log_no_change
  BEFORE UPDATE OR DELETE ON audit_log
  FOR EACH ROW EXECUTE FUNCTION audit_log_immutable();
-- and at the role level: REVOKE UPDATE, DELETE ON audit_log FROM openworkspace_app;
```

### Security and integrity audit
The schema was reviewed against current best practice; the decisions and the issues caught are below.

**Primary keys are non-enumerable and index-friendly.** Every key is a PostgreSQL 18 `uuidv7()`, which is time-ordered (so it behaves like a sequential key in B-tree indexes, unlike random UUIDv4) yet not guessable, so IDs can appear in `/api/v1` URLs without exposing sequential-ID enumeration. The one trade-off is that a v7 UUID encodes its approximate creation time; this is acceptable for internal records, authorization never depends on an ID being secret, and where creation time is sensitive the `uuidv7()` interval-shift or a separate public identifier can mask it.

**Secrets are hashed, encrypted, or not stored, deliberately per kind.** Passwords use Argon2id at the OWASP profile (m=19456 KiB, t=2, p=1, 16-byte salt, 32-byte output) stored as a self-describing PHC string, with an optional pepper supplied from configuration and never stored beside the hash. Recovery codes and API keys are stored as hashes only; API keys are high-entropy random tokens (256 bits) so a fast SHA-256 is appropriate and is looked up by a unique index, with a short non-secret prefix for display and the full key shown once. TOTP secrets and OIDC client secrets are reversible secrets, so they are encrypted at rest through the `crypto` facade rather than hashed. Passkey private keys are never transmitted or stored; only the credential id, public key and counter are kept, exactly as `webauthn-rs` persists a `Passkey`. One-time tokens (invitations, password resets and email verification) and recovery codes are stored only as hashes and expire; the SMTP password and OIDC client secrets are reversible and so are encrypted through the `crypto` facade.

**WebAuthn correctness.** The credential id is unique across all accounts, not just per user, which `webauthn-rs` requires; the signature counter is persisted and compared on each authentication to detect cloned authenticators; the user handle is a random opaque value, never the email, so authenticators carry no personal data; and the registration and authentication ceremony state is held server-side in the session, which is mandatory to prevent replay.

**OIDC providers are configurable but hardened by default.** The provider table exposes the settings real customer IdPs actually differ on (discovery or manual endpoints, client authentication including `private_key_jwt`, scope and claim mapping, group-to-role mapping in `oidc_role_mappings`, allowed email domains, RP-initiated logout, and login-button presentation), while the security-critical parts are fixed rather than offered as switches. Every flow is Authorization Code with PKCE S256, `state`, and `nonce`, which OAuth 2.1 and RFC 9700 require for all clients, so there is deliberately no field to weaken them; the implicit and resource-owner-password grants are not implemented; redirect URIs are matched exactly; and the `iss` response parameter guards against mix-up when several providers are configured. Identity is keyed on the immutable issuer and `sub`, never the email, and a federated login auto-links to an existing local account only when the email is verified, which closes the nOAuth class of takeover where an IdP asserts an unverified or mutable email; `allowed_email_domains` further gates who may provision at all. Client secrets are encrypted through the `crypto` facade, and a provider using `private_key_jwt` references a signing key in `crypto_keys` instead of holding a secret. No IdP access or refresh tokens are persisted in the first release: OIDC establishes the user's identity and OpenWorkspace then runs its own session, keeping the blast radius small; that changes only if the deferred calendar sync is built.

**Local accounts have a real lifecycle.** Invitations, password resets and email verification use single-use tokens in `user_tokens`, stored as hashes and expiring, with the raw token emailed once; `email_verified_at` records confirmation, and self-registration is off by default, so accounts arrive only by admin invitation, OIDC just-in-time, or CSV import.

**Brute force and session lifetime are bounded.** Repeated failed logins increment `failed_login_count` and trip `locked_until` per the configurable `lockout_threshold` and `lockout_minutes`; sessions expire on both an idle and an absolute timeout; MFA can be required instance-wide; and the minimum password length favours length over forced composition and is checked against a breach list, following current NIST guidance.

**Outbound email is configured, not hard-coded.** The SMTP host, port, credentials, STARTTLS and sender live in `instance_settings` with the password held as an encrypted envelope, so a deployment never has to be patched to send mail; each queued message can carry a Message-ID for iMIP threading, and the outbox idempotency key still guarantees a retry never double-sends.

**Referential integrity has a deliberate deletion policy.** Owned children cascade (occurrences with their booking, role permissions with their role, teams with their organization). References that must not silently destroy history restrict: a resource or a location with bookings or children cannot be deleted, it is archived. User references restrict because users are never hard-deleted; erasure anonymises them in place (next point), which keeps every foreign key valid while removing the personal data. Optional back-references that may outlive their target are set null (created_by, audit api_key_id).

**Polymorphism is modelled without losing foreign keys.** Where a row points at one of several entity kinds (a grant subject, a blackout scope) the schema uses two nullable foreign keys with a `num_nonnulls(...) = 1` check, so the database still enforces the reference. The single genuine exception is `audit_log.target_type` and `target_id`, which are free-form on purpose: an audit row must survive the deletion or archival of whatever it described, so it cannot hold an enforced foreign key.

**Subtype fields cannot land on the wrong node.** The campus map image and the building marker live on the location as the document requires, but check constraints ensure the map only exists on a campus, the marker only on a building, and the marker fractions stay within 0 to 1.

**The no-double-booking guarantee is structural, and so is assignment overlap.** `booking_occurrences` carries the Section 6.3 exclusion constraint over `(resource_id WITH =, period WITH &&)` filtered to live statuses, using half-open `[start, end)` ranges so a booking ending at the moment another begins does not conflict; a clashing insert raises `23P01`, mapped to HTTP 409, so two requests for the last slot can never both win. Cancellation, no-show and check-out move the row to a non-blocking status, and an early check-out also truncates the period so the freed time becomes bookable. The same GiST mechanism prevents overlapping permanent assignments on a resource. Recurrence and conflict prevention share one path: `rrule` expands a series into occurrence rows, and `chrono-tz` keeps the expansion DST-correct.

**The audit log is tamper-resistant in depth.** Updates and deletes are blocked by a trigger and, independently, the runtime database role is not granted update or delete on the table, so even a SQL-injection foothold cannot rewrite history; an optional `prev_hash`/`entry_hash` chain makes any out-of-band tampering detectable; a BRIN index suits the append-only, time-ordered shape; and retention is enforced by the worker per `instance_settings`.

**Least privilege at the database.** Two roles are used: an owner role that runs migrations and owns the schema, and a runtime application role with only `SELECT`, `INSERT`, `UPDATE`, `DELETE` on the operational tables, no DDL, and no `UPDATE`/`DELETE` on `audit_log`. For defence in depth on segmentation, the document's optional PostgreSQL row-level security attaches naturally here, keyed on the resolved organization and team.

**Uploaded files cannot carry scripts.** Asset content types are constrained to raster image formats for every image kind (no SVG), matching the decision to allow raster-only uploads and avoid an SVG sanitiser; images are re-encoded on ingest to strip metadata, and access is via short-lived presigned URLs rather than public paths.

**GDPR is built in.** Erasure (Article 17) anonymises the `users` row in place (clearing email and display name, setting `anonymized_at`) and deletes the credential rows (password, passkeys, TOTP, recovery codes, federated links, API keys), while booking occurrences remain for utilisation aggregates, now linked to an anonymised principal rather than a named person. Export (Article 15) produces a JSON plus an ICS, stored as an `export` asset and referenced from `data_subject_requests`, which records every request and its outcome.

**Time is unambiguous.** Every timestamp is `timestamptz` stored in UTC, and each location carries an IANA `timezone` that inherits down to its resources, so a resource's local day is fixed by where it physically sits rather than by who is viewing it. Wall-clock and recurrence are computed with `chrono` and `chrono-tz`, and the `.ics` carries the matching `VTIMEZONE` (Appendix F), so a booking reads correctly in every attendee's zone and across daylight-saving boundaries.

**Indexing matches the access patterns.** The exclusion constraint doubles as the availability index; partial indexes serve the hot sweeps (pending outbox rows, live occurrences for auto-release and reminders); a trigram GIN index backs resource type-ahead; the materialised `path` indexes subtree authorization; and the audit log uses BRIN for cheap time-range scans.

**Presentation order is not migration order.** The tables above are grouped by domain for reading, and a few foreign keys form natural cycles (users to locations, locations to assets, assets back to users; users to resource categories; identities to providers). The migration creates tables in dependency order and then adds the cycle-closing foreign keys with `ALTER TABLE ... ADD CONSTRAINT`, so the schema applies cleanly even though it does not read strictly top to bottom.

**Open points flagged, not hidden.** Three are worth a decision at build time. First, PostgreSQL enums (used for the few truly fixed sets) can only be appended to, not reordered or shrunk, so anything expected to churn (the permission catalogue, resource attributes) is kept as text or JSONB instead. Second, the editable `mail_templates` overlay the compile-time `askama` defaults, which means the override path needs a runtime renderer for the stored strings; if that is unwanted, the alternative is to keep templates compile-time only and make the editor manage just subject lines and branding tokens. Third, the location hierarchy uses a trigger-maintained `path` to avoid a new extension; if richer subtree querying is wanted later, `ltree` is the natural upgrade and was deliberately left out of the required extension set for now.
