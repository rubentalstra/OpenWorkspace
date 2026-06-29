# Web architecture (`apps/web`)

How the web tier is split, and the one rule that keeps it from drifting back into
two parallel implementations of the same thing.

## The rule

> A UI-facing typed request/response is a Leptos **`#[server]` function** in the
> `app` crate, with its logic in a crate **facade** (`auth`, `db`, `storage`, …).
> Reads use `#[server(input = GetUrl)]` (CSRF-exempt by method); mutations use
> `#[server(client = CsrfClient)]` (the token is read from `<meta name="csrf-token">`
> and sent as `X-CSRF-Token`). Raw Axum routes in the `server` crate exist **only**
> for the handful of things a server function cannot express. **No operation is
> implemented twice.**

## The two crates

- **`apps/web/app`** — the full-stack Leptos app. It compiles **twice**: with the
  `ssr` feature it is linked into the server binary to render HTML and run server-fn
  bodies; with `hydrate` it compiles to WebAssembly to make that HTML interactive.
  Server-only dependencies (`db`, `auth`, `secrecy`, `uuid`, `serde_json`,
  `leptos_axum`) are gated behind `ssr`, so the wasm bundle never contains them; a
  `#[server]` fn's body lives in a `#[cfg(feature = "ssr")] mod backend`, and the
  browser build sees only a typed stub that performs the HTTP call.
- **`apps/web/server`** — a thin host: process startup (config, migrations, seeds,
  services, `AppState`), the layer stack (session → CSRF → trace), the Leptos mount,
  and the few raw routes below. It holds **no business logic**.

## Server function vs. raw route

| Concern | Where | Why |
| --- | --- | --- |
| login, logout, MFA verify | `app::auth` (server fns) | typed request/response |
| change password, TOTP, passkeys, recovery | `app::account` (server fns) | typed request/response |
| SSO provider list | `app::auth::list_oidc_providers` | typed read |
| floor builder / campus | `app::build` (server fns) | typed request/response |
| **OIDC `/auth/{slug}/start` + `/callback`** | `server::oidc` (raw) | 302 redirects to/from the IdP |
| **asset upload `/api/assets`** | `server::upload` (raw) | `multipart/form-data` |
| **asset serve `/api/assets/{id}`** | `server::upload` (raw) | 302 to a presigned URL |
| **`/health` `/ready` `/metrics`** | `server::main` (raw) | ops probes, outside the auth stack |

If you reach for a new raw `/api/*` route, stop: it almost certainly wants to be a
server function instead.

## Per-request context

`leptos_routes_with_context` runs `server::context::provider` for every SSR render
**and** every server-fn call. It provides the services a server fn may
`expect_context`: `Db`, `AuthzBackend`, `WebauthnService`, `TotpService`,
`ProviderRegistry`, `app::PublicBaseUrl`, plus the CSRF token. Request-scoped state
(`AuthSession`, `MfaSession`, `OidcSession`) is pulled with `leptos_axum::extract`.

## Single-implementation facade fns (`crates/auth`)

The security-sensitive sequences live once in `auth::session::layer`:
`password_first_factor` (the password sign-in flow), `complete_second_factor` (the
MFA-completion flow), `sign_out`, `rebind_after_password_change`. Both the server-fn
adapters and the in-crate integration tests call these — never re-assemble the steps.

## WebAuthn

The passkey browser ceremony is `app::webauthn` (hydrate-gated, pure `web-sys` +
`js-sys`, no JS file): it feeds the server's challenge JSON through
`PublicKeyCredential.parseCreationOptionsFromJSON()` / `parseRequestOptionsFromJSON()`,
calls `navigator.credentials.create()` / `.get()`, and serializes the result with
`.toJSON()` for the matching server fn. The server-fn boundary carries this as opaque
JSON strings so it stays wasm-safe (no `webauthn-rs` types cross it).

## Dev sign-in

`config/app.toml` `[dev.auth]` seeds a bootstrap admin
(`admin@openworkspace.test` / `devadminpassword`) and the local Keycloak provider
(`dev_seed_keycloak = true`). At `/login`: password sign-in, or "Continue with
Keycloak" (realm users `alice` / `alicepw`, `bob` / `bobpw`). Never enable these in
production — real deployments configure providers per environment and supply the
bootstrap admin via `APP_AUTH__*`.
