# P11 — Floor builder and campus editor

> Source of truth: `OpenWorkspace-architecture-plan.md` (Appendix D "P11"; §3.4;
> §6.4). Read those before changing this phase.

## Goal

The write side of the floor layer: an in-app SVG **floor builder** that produces the
scene and binds bookable components to resources, plus a **campus map-image +
draggable-marker editor**.

> **Done:** a builder end-to-end (place a desk, bind it), marker-position storage,
> and builder accessibility checks.

## Decisions (this phase)

- **Per-seat desks (Gingco-style).** A multi-person desk is N independently-bookable
  `Desk` seats over a non-bookable `DeskBlock` surface; you book a *specific* seat.
  Placing a "Desk pod (N)" drops the block + N seats. `PartiallyFree` is therefore an
  *aggregate* state (P12), not a per-seat one.
- **Equipment = a simple reusable catalog** (`equipment_items`: a name) assigned to a
  resource with a quantity (`resource_equipment`). Not bundles.
- **Vehicles removed entirely** (enum + DB type) — no half-built resource kinds.
- **Atomic save**: the whole document (scene + resources + rules + equipment +
  bindings + zones) commits in one optimistic-locked transaction
  (`floor_plans.version`); a conflict returns a typed "reload" error.
- **UI from the kit only**: every control is a `crates/ui` component (the builder
  toolbar, picker links, equipment select, campus markers are `Button`/`NativeSelect`);
  `floorplan` depends on `ui`. (Hard rule, see `CLAUDE.md`.)

## Architecture

```
crates/floorplan/builder ── pure ops (place/move/rotate/delete + undo/redo, unit-tested)
   + FloorBuilder editor (palette, click-place single/pods, select, drag-move,
     pan/zoom, reference underlay). Edits a shared RwSignal<Scene>, reports selection.
   No db/auth. Uses ui::Button.
apps/web/app/build ── wasm-safe DTOs + #[server] fns (authz-gated) + the pages
   (/build picker, /build/:floor_id editor + resource/equipment/copy panel,
   /build/campus/:campus_id). Maps DTOs ↔ db; the app owns persistence + authz.
apps/web/server ── provides Db + AuthzBackend into the server-fn context; CsrfClient
   gained a server-side Client impl so #[server(client=CsrfClient)] mutations compile.
crates/db ── save_floor_builder_doc (atomic, optimistic, reconciles
   positions/resources/rules/equipment/zones) + resource/equipment/floor/campus I/O.
```

## Scope (this phase)

- **DB** (`crates/db`): `equipment_items` + `resource_equipment` migration; the
  `vehicle` drop migration; `save_floor_builder_doc`, `list_resources`, equipment
  CRUD, `list_floors`, `load_campus_editor`, `update_building_marker`,
  `set_campus_map_image`. `#[sqlx::test]`s.
- **Builder** (`crates/floorplan/builder`): unit-tested `ops` + the `FloorBuilder`
  editor + a render snapshot.
- **Server-fn context** (`apps/web/server`): `Db`/`AuthzBackend` in context; the
  `CsrfClient` ssr `Client` impl (unblocks all future mutations).
- **App** (`apps/web/app/build`): authz-gated `#[server]` fns + the `/build` pages +
  the resource/equipment/copy-config panel + the campus editor.

## Verified APIs (DOCS-FIRST)

- `leptos_axum` `leptos_routes_with_context` provides its context closure to **server
  functions too** (not just SSR) — that's where `Db`/`AuthzBackend` are injected.
- A custom server-fn `client=` type must impl `server_fn::client::Client` on **every**
  build; on ssr `CsrfClient` delegates to `ReqwestClient` (never invoked).
- `ui::Button` forwards `attr:`/`on:` to its root `<button>`/`<a>` (the shadcn
  `{...props}` pattern) — `attr:disabled`, `href`, etc. work without a typed prop.

## Acceptance criteria (tests)

- **floorplan**: `ops` unit tests (snap, place point/pod, move/nudge/rotate/delete,
  undo/redo, `resource_kind`); the `FloorBuilder` render snapshot.
- **db**: save → bind → unbind-on-remove, stale-version conflict, equipment CRUD +
  case-insensitive dup, marker round-trip + CHECK rejection.
- **domain**: `SpaceState::project`, `ResourceStatus`.
- **Playwright** (`end2end/build.spec.ts`, run `--headed` locally to watch): `/build`
  gates unauthenticated access; the builder renders its toolbar + canvas; axe WCAG
  2.1 AA on `/build`. The full authenticated place→bind→save→reload + campus-marker
  flow is scripted and runs against a seeded floor.

## Out of scope (later)
CSV bulk-place, measure/rulers/guides/layers, catalog expansion (later builder
polish); aggregate marker state + progressive zoom (P12); the booking views + the live
`SpaceState` projection (P13); SSE (P15); the full admin shell + per-node grant-mgmt
UI (P17/P18); in-page campus-map *upload* UI (the db + server fn exist; the multipart
upload widget is a follow-up). Rotate-handle + multi-click wall/zone drawing in the
builder are noted follow-ups.

## Verification

`cargo nextest run -p floorplan -p db -p domain` (DATABASE_URL=owner); then the full
gate: `cargo fmt --all --check && leptosfmt --check apps crates && cargo clippy
--workspace --all-targets -- -D warnings && cargo nextest run --workspace && cargo
deny check && cargo audit && cargo leptos build`. In-browser: `cargo leptos watch`,
sign in, `/build` → open a floor → place a desk pod → bind a seat (+rules+equipment) →
Save → reload; copy-config → paste; `/build/campus/:id` drag a marker. `npx playwright
test build --headed` to watch the e2e.
