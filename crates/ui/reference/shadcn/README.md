# shadcn/ui (Base UI) source archive — port ground truth

This directory is the **canonical reference** for the `crates/ui` rewrite. Every Rust/Leptos component
is transcribed 1:1 from the matching source here. **Reference only — never compiled.** It lives outside
any crate `src/`, so Cargo never builds it and the Tailwind v4 `@source` globs never scan it.

> **Rule: do not guess.** When porting a component, open its `.tsx` here and transcribe the element
> structure, the `data-slot`/`data-*`/`aria-*` attributes, the `cn-*` semantic classes + inline
> structural utilities, and the controlled/uncontrolled behaviour exactly. Re-read the source rather
> than infer.

## Flavour: Base UI (not Radix)

shadcn ships two primitive bases — **Radix** and **Base UI** (<https://base-ui.com>). We port the
**Base UI** flavour. Its components import from `@base-ui/react/*` and decompose / wire ARIA / emit
`data-*` differently from Radix. Since we reimplement everything in Leptos, "Base UI as reference"
means we reproduce **Base UI's** structure, `data-slot` names, data-attribute conventions, and
behaviour — not Radix's.

## Styling model: semantic `cn-*` classes + the `nova` style

Base-UI shadcn does **not** inline the themed Tailwind utilities in each component. Instead:

1. A component's `cva()` emits **semantic classes** — `cn-button`, `cn-button-variant-default`,
   `cn-button-size-default`, plus a few inline *structural* utilities (`inline-flex shrink-0 …`) and a
   `data-slot="button"`.
2. A **named style CSS** defines every `cn-*` class with `@apply <utilities>`, scoped under a
   `.style-<name>` wrapper. shadcn ships 8 styles (luma, lyra, maia, mira, nova, rhea, sera, vega).
3. **We use the `nova` style.** `styles/style-nova.css` (369 `cn-*` rules) is our authoritative source
   for what every component *looks like*.

This is a great fit for a Rust port: the Leptos components stay thin (emit `cn-*` + `data-slot` + a few
structural utilities), and the entire visual identity lives in one CSS file we ship in the app.

### Port mapping
- `cva(base, { variants })` → the `variants!` macro: each variant axis becomes a `#[default]` enum whose
  `class()` returns the **semantic** class (`cn-button-variant-default`), not utilities. `cn!` still
  merges (so callers can still override with utilities via `tw_merge`).
- **`data-slot`** is mandatory on every element — the nova CSS targets `[data-slot=…]` extensively.
- **`@base-ui/react` data-attrs** (`data-open`, `data-closed`, `data-checked`, `data-disabled`,
  `data-side`, …) must be reproduced; `styles/shadcn-tailwind.css` defines the `@custom-variant`s that
  make `data-open:animate-…` etc. work.
- **Icons:** Base UI source uses an internal `IconPlaceholder`; replace with the real Lucide glyph via
  `icondata::Lu<Name>` + `leptos_icons::Icon` (match the icon the component clearly intends).
- **`render` / `asChild`:** Base UI's polymorphism — reproduce the intended element directly in Leptos
  (e.g. `href` → `<a>` else `<button>`), preserving classes and `data-*`.

## Layout

| Path | What |
| --- | --- |
| `ui/*.tsx` | 60 component primitives (the port targets). Authoring imports use `@/registry/bases/base/…` aliases. |
| `blocks/<name>/**` | 30 blocks incl. **named deliverables `sidebar-07`, `login-03`, `signup-03`**, plus `dashboard-01`, `sidebar-01..16`, `login-01..05`, `signup-01..05`, previews. |
| `examples/*.tsx` | 65 usage examples — reference for how each component is composed/used. |
| `hooks/use-mobile.ts` | `useIsMobile()` (768px) → our `use_is_mobile`. |
| `lib/utils.ts` | `cn()` = `twMerge(clsx(...))` → our `cn!`. |
| `components/`, `internal/` | shadcn's own site tooling (placeholder/sink) — minor reference only. |
| `styles/style-nova.css` | **The nova look** — 369 `cn-*` `@apply` rules. Source for the app stylesheet. |
| `styles/shadcn-tailwind.css` | Infra from the `shadcn` npm package: `@keyframes` (accordion, scroll-fade), `@custom-variant data-open/closed/checked/…`, scroll/no-scrollbar `@utility`. Required for the data-attr variants + animations. |
| `styles/globals.css` | App-level setup: `:root`/`.dark` oklch tokens, `@theme inline` mappings, `@custom-variant dark` + `style-*`, `@layer base`, custom `@utility`s. |

## Provenance

- Source: shadcn/ui v4 registry, **Base UI base** — `github.com/shadcn-ui/ui`, path
  `apps/v4/registry/bases/base/` (mirrored via `raw.githubusercontent.com`, branch `main`).
- Style CSS + infra: `apps/v4/registry/styles/style-nova.css`, `apps/v4/app/globals.css`, and the
  `shadcn` npm package's `tailwind.css`.
- Fetched: **2026-06-28** (UTC).

### Docs-nav entries that are NOT standalone components
**Typography** (styling guide), **Data Table** (Table + TanStack guide), **Date Picker** (Popover +
Calendar guide), **Toast** (deprecated → **Sonner**). Build these by composition; there is no single
`ui/` file for them.

## Refreshing
Re-mirror from the same paths on `main`. Keep the **Fetched** date current and re-diff existing ports
against any changed source.
