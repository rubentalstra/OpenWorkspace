#!/usr/bin/env bash
# SessionStart hook. Surfaces the non-negotiable workflow rules and the quick
# gate as additional context, so the conventions are active from the first turn
# without re-reading CLAUDE.md in full.
set -uo pipefail

read -r -d '' context <<'EOF' || true
OpenWorkspace non-negotiables (see CLAUDE.md):
- DOCS-FIRST: verify any API against the pinned version's docs (context7) before using it.
- Commits authored by the human only — no assistant/co-author/"generated with" trailers anywhere.
- Conventional Commits: type(scope): subject (≤50 chars, imperative, no trailing period).
- Branch off main; never commit or push to main directly. Open PRs with gh.
- Minimal comments: explain why, never what. Only // SAFETY: and /// on public items.
- Every third-party crate sits behind a first-party facade.
Quick gate before pushing:
  cargo fmt --all --check && leptosfmt --check apps crates && cargo clippy --workspace --all-targets -- -D warnings && cargo nextest run --workspace && cargo deny check && cargo audit && cargo leptos build
EOF

if command -v jq >/dev/null 2>&1; then
    jq -n --arg c "$context" \
        '{hookSpecificOutput: {hookEventName: "SessionStart", additionalContext: $c}}'
else
    printf '%s\n' "$context"
fi
