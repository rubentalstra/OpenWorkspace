# shadcn/ui source archive — port ground truth

This directory is the **canonical reference** for the `crates/ui` rewrite. Every Rust/Leptos component
is transcribed 1:1 from the matching source here. **Reference only — never compiled.** It is outside
any crate `src/`, so Cargo never builds it and the Tailwind v4 `@source` globs never scan it.

> **Rule: do not guess.** When porting a component, open its `.tsx` here and transcribe the base classes,
> every CVA variant string, every `data-*`/`aria-*` attribute, the element structure, and the
> controlled/uncontrolled behaviour exactly. If something is unclear, re-read the source — never infer.

## Provenance

- Source: official shadcn/ui registry — <https://ui.shadcn.com>
- Style: **new-york-v4** (Tailwind v4, oklch tokens, `data-slot` attributes) — matches our pinned stack.
- Endpoint: `https://ui.shadcn.com/r/styles/new-york-v4/<name>.json` (per-item, carries `files[].content`).
- Index: `https://ui.shadcn.com/r/index.json` → `registry/_index.json`.
- Fetched: **2026-06-28** (UTC).

We deliberately mirror **new-york-v4**, not the legacy `/r/styles/new-york/` source. The legacy source is
Tailwind v3 (`focus-visible:ring-1`, `shadow`); v4 is what our app's `tailwind.css` and component set use
(`focus-visible:ring-[3px]`, oklch, `data-slot`).

## Layout

| Path | What |
| --- | --- |
| `ui/*.tsx` | 56 component primitives (the port targets). |
| `blocks/<name>/**` | 26 blocks: `sidebar-01..16`, `login-01..05`, `signup-01..05`. **Named deliverables: `sidebar-07`, `login-03`, `signup-03`.** |
| `lib/utils.ts` | shadcn `cn()` = `twMerge(clsx(...))` → our `cn!` over `tw_merge`. |
| `hooks/use-mobile.ts` | `useIsMobile()` (768px breakpoint) → our `use_is_mobile`. |
| `theme-neutral.json` | The neutral base color tokens (light/dark), incl. the full `--sidebar-*` and `--chart-*` groups → source for `apps/web/style/tailwind.css`. |
| `registry/<name>.json` | Raw registry item per component/block: `dependencies`, `registryDependencies` (composition graph), `files`. Use this to learn which components a block/component composes. |
| `registry/_index.json` | Full v4 registry index (names, types, radix/base doc links). |

## Port conventions

- **CVA → Rust.** shadcn `cva(base, { variants, defaultVariants })` → the `variants!` macro: a `#[default]`
  enum per variant axis with a `class()` method, plus a reactive `#[component]`. shadcn `cn(...)` → `cn!`.
- **Icons.** lucide-react named imports (e.g. `ChevronRight`) → `icondata::Lu<Name>` rendered via
  `leptos_icons::Icon` with `attr:class`. The Lucide glyph set matches 1:1; the rename is mechanical.
- **`asChild` / Radix `Slot`.** Rust has no `Slot`; reproduce the intended element/behaviour directly
  (e.g. a `href` prop that renders `<a>` vs `<button>`), preserving classes and `data-*`.
- **`data-slot`.** Every v4 component sets `data-slot="..."`; reproduce it (styling hooks depend on it).
- **Interactivity.** Pure Leptos only — `RwSignal`/`Memo`/`Effect`/`Show`/`For`/context/`window_event_listener`.
  No JS, no inline `<script>`.

## Not in new-york-v4 (as of fetch date)

`attachment`, `bubble`, `marker`, `message`, `message-scroller` return 404 under new-york-v4 — they ship
only in a newer component base (the radix/base split). They are not used by our crate or the named
deliverables; revisit if they land in the v4 style.

## Refreshing

Re-run the mirror (see the rewrite plan) against the same endpoint to update. Keep this README's
**Fetched** date in sync, and re-diff existing ports against any changed source.
