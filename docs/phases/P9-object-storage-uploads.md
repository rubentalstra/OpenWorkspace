# P9 — Object storage and uploads

> Source of truth: `OpenWorkspace-architecture-plan.md` (Appendix D "P9"; §6.6
> storage; Appendix H "Uploaded files cannot carry scripts"). Read those before
> changing this phase.

## Goal

Provide the binary-asset layer the later phases consume — campus map images,
floor reference backgrounds, resource photos, the instance logo, GDPR exports and
CSV imports — with a **raster-only upload pipeline** and **presigned-URL access**.

> **Done:** upload, re-encode and format and size-rejection tests.

## Security posture

- **Raster only, no SVG, no sanitiser.** Uploads are accepted only as PNG/JPEG/WebP
  (detected by magic bytes, not the client's declared type/extension). AVIF is in
  the stored-`content_type` allow-list but not accepted for upload yet (its codecs
  need heavy native deps); SVG and every other format are rejected with `415`.
- **Re-encode strips metadata.** Every upload is decoded under strict
  dimension/allocation limits (decompression-bomb guard) and re-encoded, which
  drops EXIF/GPS and any ancillary segments. A WebP thumbnail variant is generated.
- **Presigned URLs, never public paths.** Assets are served via short-lived
  presigned GET URLs; the app authorizes the request, then 302-redirects.
- **Least privilege.** The runtime DB role already has DML on `assets` (P8); the S3
  identity is dev-only in the repo, prod via secrets.

## Scope (this phase)

Headless storage layer + real endpoints + tests; **no UI** (the consuming admin /
builder screens are P11/P17).

1. **`crates/storage`** — facade over `object_store` (S3/SeaweedFS, local,
   in-memory) and `image`. `Storage` (`put`/`get`/`delete`/`signed_get_url`,
   `from_config`); `process_upload` (validate → bounded decode → re-encode →
   thumbnail → SHA-256); `StorageError`, `ImageKind`, `ImageLimits`. No vendor type
   crosses the facade.
2. **`crates/db::assets`** — `AssetKindRow`, `AssetRow`, `NewAsset`, and
   insert/variant/load/delete queries against the existing `assets` table.
   `domain::AssetId` added. **No migration** (the `assets` table, `asset_kind`
   enum and FKs exist from P2).
3. **Server** — `POST /api/assets` (multipart, `DefaultBodyLimit`, CSRF via the
   surrounding layer, authorized **per kind** through the P8 `AuthzBackend`:
   `logo`→`InstanceConfigure`, `campus_map`→`HierarchyEdit`,
   `floor_background`/`reference_image`→`FloorBuild`, `object_photo`→`ResourceManage`;
   `export` rejected) and `GET /api/assets/{id}` (→ 302 presigned URL,
   `?variant=thumb` for the thumbnail). `Storage` wired into `AppState`.
4. **Dev/CI** — SeaweedFS gains an S3 identity (`deploy/dev/seaweedfs/s3.json`) so
   presigned sigv4 works; the `openworkspace` bucket is provisioned. CI starts
   SeaweedFS in the nextest job.

## Pinned versions (DOCS-FIRST verified)

- `object_store 0.14.0` (feature `aws` → `reqwest/rustls` + **aws-lc-rs**, no ring;
  `Signer::signed_url` for presigning; `AmazonS3Builder` endpoint/allow-http/
  path-style). `image 0.25.10` (pure-Rust `png`/`jpeg`/`webp`; re-encode via
  `DynamicImage` drops metadata). SeaweedFS 4.36 S3 (identity JSON; bucket via
  `weed shell`; sigv4 presigned URLs).

## Acceptance criteria (tests)

- `crates/storage` (in-memory + **real SeaweedFS**): accepts PNG/JPEG/WebP and
  thumbnails them; **rejects** SVG/GIF/non-raster (`415`) and oversized
  (`413`); re-encode **strips embedded metadata**; thumbnail downscales preserving
  aspect; `put → signed_get_url → fetch` returns identical bytes with the right
  `Content-Type`.
- `crates/db` (`#[sqlx::test]`): asset insert/load/variant round-trip; variant
  cascades on parent delete; the `content_type` CHECK rejects non-raster
  non-export types.
- `server`: the `kind → (action, target)` authorization mapping (and its
  rejections). The handler is thin glue over these integration-tested layers
  (matching the repo's existing handler-test boundary).

## Out of scope (later phases)

Upload/preview UI and the attach flows that set the consuming FKs (P11/P17); AVIF
codec support; SSE-S3 encryption-at-rest and CDN (P20); GDPR export *content*
(P19 — P9 only provides `export`-kind storage); worker-offloaded variant
generation (P16 — P9 re-encodes synchronously in `spawn_blocking`).

## Verification

Dev stack up (Postgres + SeaweedFS). `cargo nextest run -p storage -p db`
(`DATABASE_URL` = owner; SeaweedFS at `localhost:8333`), then the full gate.
`cargo sqlx prepare --workspace` for the new `db::assets` queries.
