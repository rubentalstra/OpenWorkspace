---
name: pre-pr
description: Run the full OpenWorkspace quality gate and report pass/fail per check before opening or updating a PR. Use when the user wants to verify a branch is green, or is about to push or open a PR.
allowed-tools: "Bash(cargo fmt *) Bash(leptosfmt *) Bash(cargo clippy *) Bash(cargo nextest *) Bash(cargo deny *) Bash(cargo audit *) Bash(cargo leptos *) Bash(cargo sqlx *) Bash(git status *)"
---

Run the same gate CI runs, locally, and report a per-check summary. Dev services (PostgreSQL etc.) must be up for the test step: `podman compose -f deploy/dev/compose.yaml up -d`.

Run these in order and capture the result of each (don't stop at the first failure — collect all so the user sees the full picture):

1. `cargo fmt --all --check`
2. `leptosfmt --check apps crates`
3. `cargo clippy --workspace --all-targets -- -D warnings`  (no `--all-features` — ssr+hydrate conflict)
4. `cargo nextest run --workspace`  (needs the DB up; migrations applied)
5. `cargo sqlx prepare --workspace --check`  (fails if `.sqlx/` is stale — regenerate with `cargo sqlx prepare --workspace` and commit)
6. `cargo deny check`
7. `cargo audit`
8. `cargo leptos build`

Report a checklist (✓/✗ per gate). For any failure, show the relevant output and the one-line fix. If everything passes, say the branch is gate-clean and remind the user to use a Conventional-Commit PR title and to merge with `gh pr merge <n> --squash --delete-branch` once checks are green. Do not open the PR yourself unless asked.
