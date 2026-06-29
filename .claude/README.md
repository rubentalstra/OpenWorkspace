# `.claude/` — local workflow tooling

Committed configuration that helps any editor agent follow this repo's conventions. Everything here is plain shell and markdown; nothing runs in CI.

## Layout

| Path                      | Purpose                                                                                                                                |
|---------------------------|----------------------------------------------------------------------------------------------------------------------------------------|
| `settings.json`           | Hooks + a permissions allowlist for the cargo/sqlx/gh/podman commands used in the gate. **Committed.**                                 |
| `settings.local.json`     | Machine-local env only (`DATABASE_URL`, `SQLX_OFFLINE`). **Git-ignored** — never commit.                                               |
| `hooks/guard-git.sh`      | Pre-command guard: blocks commits/pushes to `main`, attribution trailers, and non-Conventional-Commit messages before they run.        |
| `hooks/fmt-rs.sh`         | Formats edited `.rs` files (`leptosfmt` + `rustfmt`) so the CI fmt gate stays green.                                                   |
| `hooks/session-rules.sh`  | Surfaces the non-negotiables + quick gate at session start.                                                                            |
| `agents/rust-reviewer.md` | A reviewer that judges diffs against the house rules (lint policy, error split, facade rule, comments, server/sqlx/async constraints). |
| `skills/commit`           | Compose a compliant Conventional Commit.                                                                                               |
| `skills/pre-pr`           | Run the full quality gate and report pass/fail per check.                                                                              |
| `skills/new-migration`    | Scaffold + apply a sqlx migration and refresh `.sqlx/`.                                                                                |
| `skills/new-crate`        | Scaffold a workspace crate the house way.                                                                                              |

## Git-level enforcement (`.githooks/`)

The same commit rules are enforced for **every** commit (any tool, any person) via committed git hooks. Activate once per clone:

```sh
git config core.hooksPath .githooks
```

- `.githooks/commit-msg` — validates the message (Conventional Commits, no attribution, no trailing period).
- `.githooks/pre-push` — blocks pushes to `main`/`master` and runs the cheap fmt gates.
- `.githooks/lib/validate-commit-msg.sh` — the shared validator both the git hook and `hooks/guard-git.sh` call, so the two layers can't drift.

The rules themselves live in `CLAUDE.md` and `docs/rust-style.md` — those remain the source of truth.

## Vendored skills (`skills/rust-skills`)

A third-party Rust ruleset is vendored into the repo (the agent loads only the rules relevant to the code in front of it; invoke with `/rust-skills`).

- **Source:** `github.com/leonardomso/rust-skills`, version `1.5.1` (see the `metadata` block in `skills/rust-skills/SKILL.md`).
- **License:** MIT — `skills/rust-skills/LICENSE` is kept as required for redistribution.
- **Vendored:** `SKILL.md` (index) + all 265 rule files under `rules/`, across 26 categories.
- **Deliberately excluded:** the upstream `checks/` directory (Python/shell maintainer tooling — we don't run it), the README/CHANGELOG/CONTRIBUTING, and the `CLAUDE.md`/`AGENTS.md` symlinks (a nested `CLAUDE.md` would be loaded as project instructions). No nested `.git`.
- **It supplements, not overrides:** `CLAUDE.md` and `docs/rust-style.md` remain authoritative — they pin this repo's stack-specific rules (Leptos `#[server]`, sqlx `query!`, the facade rule, leptosfmt) that the generic ruleset does not cover. On any conflict, the repo docs win.

To update: re-clone upstream to a scratch dir, then copy `SKILL.md`, `LICENSE`, and `rules/` over `skills/rust-skills/` (never the `checks/` dir or the `.git`), and bump the version noted above.
