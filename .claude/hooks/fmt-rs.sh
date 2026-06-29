#!/usr/bin/env bash
# PostToolUse(Edit|Write) formatter. When a Rust source file under apps/ or
# crates/ is written, format it in place with leptosfmt then rustfmt so the CI
# fmt gate stays green. PostToolUse cannot block; this only normalises the file.
set -uo pipefail

command -v jq >/dev/null 2>&1 || exit 0

file="$(jq -r '.tool_input.file_path // empty' 2>/dev/null)"
[[ -z "$file" ]] && exit 0
[[ "$file" == *.rs ]] || exit 0
[[ -f "$file" ]] || exit 0

case "$file" in
    */apps/*|*/crates/*|apps/*|crates/*) ;;
    *) exit 0 ;;
esac

# leptosfmt rewrites view! macros; rustfmt handles the rest. Both read the
# repository's pinned config. Stay silent on success; report on failure only.
command -v leptosfmt >/dev/null 2>&1 && leptosfmt --quiet "$file" >/dev/null 2>&1 || true
command -v rustfmt   >/dev/null 2>&1 && rustfmt --edition 2024 "$file" >/dev/null 2>&1 || true

exit 0
