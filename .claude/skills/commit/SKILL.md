---
name: commit
description: Stage and commit the current changes as a single Conventional Commit that passes this repo's commit-msg rules. Use when the user asks to commit work in OpenWorkspace.
argument-hint: "[optional subject hint]"
allowed-tools: "Bash(git status *) Bash(git diff *) Bash(git add *) Bash(git commit *) Bash(git log *)"
---

Create one Conventional Commit for the current change set in OpenWorkspace.

## Steps

1. Run `git status` and `git diff --staged` (and `git diff` for unstaged) to see exactly what will be committed. If nothing is staged, stage the intended files with `git add` — confirm the set is coherent and unrelated changes are left out.
2. Confirm you are **not** on `main`/`master`. If you are, stop and tell the user to branch first (`git checkout -b <type>/<topic>`) — the hooks will reject a commit on main anyway.
3. Compose the message:
   - `type(scope): subject` — type ∈ `feat fix docs style refactor perf test build ci chore`.
   - scope = the crate or app touched (`domain db auth floorplan crypto i18n notify jobs ui config observability web worker app frontend server`) or a cross-cutting scope (`ci dev deps release`).
   - subject: imperative mood, ≤50 chars, **no trailing period**.
   - Add a body only when the *why* isn't obvious from the subject. Use a `BREAKING CHANGE:` footer or `!` after the scope for breaking changes (pre-V1: a breaking change bumps MINOR).
   - **Never** add an assistant/AI mention or any co-author/"generated with" trailer — commits are authored by the human only.
4. Commit with `git commit -m "..."` (and `-m` again for a body). The local `commit-msg` hook validates the result; if it rejects, read the message and fix it rather than bypassing.
5. Show the user the final `git log -1 --stat`.

Do not push or open a PR unless explicitly asked.
