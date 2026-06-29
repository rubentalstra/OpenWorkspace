# P10 — Scene model, renderer and catalog

> Source of truth: `OpenWorkspace-architecture-plan.md` (Appendix D "P10"; §6.4
> the structured scene; §3.2 the six UI states). Read those before changing this
> phase.

## Goal

The floor layer the booking views consume: a structured **scene model**, a
growable **SVG component catalog**, and a read-only **inline-SVG renderer** with
pan/zoom, reactive `data-state`, and a keyboard-navigable, accessible surface.

> **Done:** scene round-trips (serde and through Postgres jsonb) and renderer
> snapshots are stable.

## Architecture

```
crates/floorplan ──
  model/   pure serde (Scene, SceneNode, Geometry, Transform, Style, ViewBox,
           SceneNodeId, CatalogKind) — no Leptos, always built; schema versioning,
           migration chain, typed SceneError + validate(); samples::office().
  catalog/ Leptos (ssr/hydrate): a registry (static CatalogEntry table + entry!
           macro) mapping CatalogKind → an inline-SVG render fn + palette metadata.
  render/  Leptos (ssr/hydrate): <FloorPlan> — reactive viewBox pan/zoom, per-node
           data-state via Memo, click/Enter/Space selection.
        ▲                                  ▲
  db (dev-dep only) ─┐              apps/web/app ── /ui/floor showcase
  db::load_floor_plan ┘            (floorplan dep, ssr/hydrate forwarded)
```

`leptos`/`web-sys` are optional behind `ssr`/`hydrate`, so the model stays UI-free
and `db`/server code can (de)serialize `floor_plans.scene` without a UI framework.

## The status model — three distinct layers

These are **not** duplicates; they live at different layers and are bridged by one
tested projection (`domain::SpaceState::project`).

- **`domain::BookingStatus`** — the lifecycle of one reservation
  (`booked → checked_in → checked_out`, `released`, `no_show`, `cancelled`).
  `checked_in` belongs here.
- **`domain::OccurrenceKind`** — what occupies a resource's calendar row:
  `booking` / `permanent_assignment` / `blackout` (feeds the GiST no-double-booking
  constraint).
- **`domain::SpaceState`** — the six *display* states (plan §3.2): `free`,
  `partially_free`, `not_free`, `temporarily_blocked`, `permanent_user`,
  `cannot_be_booked`. A presentation read-model the renderer shows as `data-state`.
  Projected per viewed window from the facts above + capacity + bookability
  (`SpaceState::project`, implemented now; the DB query that supplies the
  occurrences lands in P13). A checked-in desk projects to `not_free`.

The enum lives in `domain` (not `floorplan`) so the P13 projection stays domain
logic without a `domain → floorplan` cycle; `floorplan` depends on `domain` and
re-exports `SpaceState`.

## Scope (this phase)

- **Scene model** (`crates/floorplan/src/model`): `Scene`/`SceneNode`/`Geometry`
  (point/line/polygon/path)/`Transform`/`Style`/`ViewBox`/`SceneNodeId`;
  `CatalogKind` is a closed serde enum with `#[serde(other)] Unknown` (a newer
  build's kind loads as a neutral placeholder, never a parse failure). Schema
  versioning (`CURRENT_SCENE_VERSION`) + a forward-migration chain (`load_scene`)
  so any past scene always loads; typed `SceneError` + `Scene::validate`.
- **Catalog registry** (`crates/floorplan/src/catalog`): one static `CATALOG`
  table (built with the `entry!` macro) is the single extension point — a new
  component is one render fn + one entry + its `CatalogKind` variant. A completeness
  test keeps the enum and the table in lockstep. Per-category render modules
  (structure / bookable / zoning / wayfinding / annotation). Bookable nodes render a
  focusable `role="button"` group with a reactive `data-state` and a Lucide state
  glyph; wayfinding nodes render their Lucide marker with an SVG `<title>`. Same
  registry feeds the P11 builder palette (`entries`/`by_category`).
- **Renderer** (`crates/floorplan/src/render`): `<FloorPlan scene states on_select>`
  — an `RwSignal<ViewBox>` drives pan (`pointerdown/move/up` + pointer capture) and
  zoom-to-cursor (`wheel`), mapped through the `NodeRef<Svg>` bounding rect. Nodes
  render once (static scene); each bookable node's `data-state` is a `Memo` so a
  single-node availability change (P15 SSE) repaints only that node. SSR and hydrate
  emit byte-identical markup (initial `viewBox` from the scene; handlers wire on
  hydrate).
- **Theming layer** (`apps/web/style/main.css`): the re-themeable `cn-floor-*`
  layer. A deployment restyles the whole floor by overriding the `--cn-floor-*`
  tokens (which reuse the Tailwind palette) or the rules — like `nova` themes the UI
  kit. Availability is shown by **icon and colour**, never colour alone (WCAG 1.4.1).
- **DB loader** (`crates/db/src/floorplan.rs`): `load_floor_plan(pool, LocationId)
  → Option<FloorPlanRow>` via compile-time `query_as!`; `scene` stays opaque
  `serde_json::Value`. The consumer feeds `(scene, scene_schema_version)` into
  `floorplan::load_scene` so the migration chain runs on load.
- **Showcase** (`apps/web/app/src/showcase/floor.rs`, route `/ui/floor`): the
  sample office rendered live with a state legend; the showcase is split into a
  shell (`mod.rs`) + one file per page.

## Pinned versions (DOCS-FIRST verified)

- **Leptos 0.8.20** — `view!` renders SVG natively and preserves camelCase
  attributes (`viewBox`, `preserveAspectRatio`); `NodeRef::<leptos::svg::Svg>`;
  `RenderHtml::to_html` for snapshot rendering under an `Owner`.
- **leptos_icons 0.7.1 / icondata 0.7** — Lucide glyphs; `Icon`'s `width`/`height`
  props (not `attr:`) replace the default `1em`.
- No new runtime crates; `insta` is the only new dev-dependency.

## Acceptance criteria (tests)

- **Model** (`floorplan`, no feature): serde round-trip (string + `Value`),
  `Unknown`-kind fallback, migration (`load_scene` empty/identity, future version
  rejected), validation (duplicate id / degenerate viewBox / bad geometry),
  `proptest` round-trip over integer-coordinate scenes.
- **Registry** (`floorplan`, ssr/hydrate): every non-`Unknown` `CatalogKind` is
  registered exactly once; `bookable` flag matches the kind; `by_category` groups.
- **Renderer snapshots** (`floorplan`, ssr): `<FloorPlan>` over the office sample
  → `to_html` → `insta`, default and mixed `SpaceState` maps.
- **Domain** (`domain`): `SpaceState::project` precedence + capacity, `as_str` keys.
- **DB** (`db`, `#[sqlx::test]`): a scene stored as jsonb loads back through
  `load_floor_plan` + `load_scene` equal to the original (the Scene↔Postgres-jsonb
  round-trip); a missing plan is `None`.
- **Playwright** (`end2end/floor.spec.ts`): `/ui/floor` server-renders and hydrates
  without console errors; wheel/drag change the `viewBox`; bookable desks are
  focusable; selection updates the readout; axe WCAG 2.1 AA passes.

## Out of scope (later phases)

- Floor **builder**/editor, `resource_positions` binding, campus map-image/marker
  editor (P11) — the catalog registry + theming + state model are complete; the
  remaining catalog *components* are added one entry at a time as P11/P13 need them.
- Progressive zoom, breadcrumb, exploded 3D floor stack (P12).
- The Map/List/Calendar booking views, object-detail popup, and loading real
  persisted floors into a page; the `SpaceState` **availability projection** from
  live DB occurrences (P13).
- **SSE** live single-node `data-state` mutation (P15). Visual polish (P20).

## Verification

Dev stack up. `cargo nextest run -p floorplan -p domain -p db` (model + projection
+ snapshot + jsonb round-trip; `DATABASE_URL` = owner). Then the full gate:

```
cargo fmt --all --check && leptosfmt --check apps crates \
  && cargo clippy --workspace --all-targets -- -D warnings \
  && cargo nextest run --workspace && cargo deny check && cargo audit \
  && cargo leptos build
```

In-browser: `cargo leptos watch` → `/ui/floor` (pan/zoom, focus a desk, cycle a
state); `npx playwright test floor` in `end2end/` for render/interactivity/a11y.
Regenerate `.sqlx` after touching the loader query (`cargo sqlx prepare
--workspace`); record renderer snapshots with `cargo insta accept`.
