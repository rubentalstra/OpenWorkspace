---
name: new-migration
description: Scaffold and apply a sqlx database migration the OpenWorkspace way, then refresh the offline query cache. Use when adding or changing the PostgreSQL schema in crates/db.
argument-hint: "<migration_name>"
allowed-tools: "Bash(sqlx *) Bash(cargo sqlx *) Bash(git add *)"
---

Add a reversible sqlx migration under `crates/db/migrations` and keep the offline cache in sync. Requires `DATABASE_URL` set (it's in `.claude/settings.local.json`) and the dev database up (`podman compose -f deploy/dev/compose.yaml up -d`).

## Steps

1. Scaffold a reversible pair:
   `sqlx migrate add -r <name> --source crates/db/migrations`
2. Edit the generated `*.up.sql` and `*.down.sql`. Keep the down migration a true inverse. Follow the schema conventions already in `crates/db/migrations` and the data model in `docs/` — read the relevant phase spec before designing tables; do not infer the schema.
3. Apply: `sqlx migrate run --source crates/db/migrations` (revert with `sqlx migrate revert --source crates/db/migrations`).
4. If you changed any `query!`/`query_as!` call or the schema they touch, regenerate the offline cache and commit it:
   `cargo sqlx prepare --workspace`
5. Stage the migration files **and** the updated `.sqlx/` together (`git add crates/db/migrations .sqlx`). CI runs `cargo sqlx prepare --workspace --check` and fails if `.sqlx/` is stale.

Then build/test to confirm the new queries compile against the schema.
