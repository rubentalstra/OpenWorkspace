#!/usr/bin/env bash
# PreToolUse(Bash) guard. Inspects the command an agent is about to run and
# blocks workflow violations before they happen: direct commits/pushes to main,
# attribution trailers, and non-Conventional-Commit messages. Exits 2 to block
# (the stderr text is shown back to the agent); exits 0 to allow.
#
# This is the in-session layer. The committed .githooks/ layer is the backstop
# that also covers editor commits and any non-agent tooling.
set -uo pipefail

repo_root="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
validator="$repo_root/.githooks/lib/validate-commit-msg.sh"

# Fail open: never wedge unrelated work if a dependency is missing.
command -v jq >/dev/null 2>&1 || exit 0

cmd="$(jq -r '.tool_input.command // empty' 2>/dev/null)"
[[ -z "$cmd" ]] && exit 0

# Only git/gh commands are interesting.
case "$cmd" in
    *git*|*gh*) ;;
    *) exit 0 ;;
esac

block() {
    printf '%s\n' "$1" >&2
    exit 2
}

branch="$(git -C "$repo_root" rev-parse --abbrev-ref HEAD 2>/dev/null || echo '')"

# Extract every -m / --message payload (double- or single-quoted).
extract_messages() {
    perl -ne 'while (/(?:-m|--message)[ =]+"([^"]*)"|(?:-m|--message)[ =]+'\''([^'\'']*)'\''/g) { print((defined $1 ? $1 : $2), "\n") }' <<<"$cmd"
}

validate_text() {
    [[ -x "$validator" ]] || return 0
    local out
    if ! out="$("$validator" <<<"$1" 2>&1)"; then
        block "$out"
    fi
    [[ -n "$out" ]] && printf '%s\n' "$out" >&2
}

# git commit ---------------------------------------------------------------
if [[ "$cmd" =~ (^|[\;\&\|[:space:]])git([[:space:]]+-[^[:space:]]+)*[[:space:]]+commit ]]; then
    if [[ "$branch" == "main" || "$branch" == "master" ]]; then
        block "blocked: direct commits to $branch are not allowed. Branch off main first (git checkout -b <type>/<topic>)."
    fi
    # Reconstruct the message git will build: the first -m is the subject and
    # each subsequent -m is a body paragraph. Validate the whole thing once so
    # body paragraphs are not mistaken for malformed headers.
    msgs=()
    while IFS= read -r m; do msgs+=("$m"); done < <(extract_messages)
    if (( ${#msgs[@]} )); then
        full="${msgs[0]}"
        for ((i = 1; i < ${#msgs[@]}; i++)); do full+=$'\n\n'"${msgs[i]}"; done
        validate_text "$full"
    fi
fi

# git push -----------------------------------------------------------------
if [[ "$cmd" =~ (^|[\;\&\|[:space:]])git([[:space:]]+-[^[:space:]]+)*[[:space:]]+push ]]; then
    if [[ "$cmd" =~ (^|[[:space:]])(origin[[:space:]]+)?(HEAD:)?(refs/heads/)?(main|master)([[:space:]]|$) ]]; then
        block "blocked: pushing to main/master is not allowed. Open a PR from a feature branch instead."
    fi
    if [[ ! "$cmd" =~ push[[:space:]]+[^[:space:]] ]] && [[ "$branch" == "main" || "$branch" == "master" ]]; then
        block "blocked: current branch is $branch and a bare 'git push' would target it. Open a PR from a feature branch instead."
    fi
fi

# gh pr create -------------------------------------------------------------
if [[ "$cmd" =~ gh[[:space:]]+pr[[:space:]]+create ]]; then
    title="$(perl -ne 'print "$1\n" if /(?:-t|--title)[ =]+"([^"]*)"/ or /(?:-t|--title)[ =]+'\''([^'\'']*)'\''/' <<<"$cmd")"
    body="$(perl -ne 'print "$1\n" if /(?:-b|--body)[ =]+"([^"]*)"/ or /(?:-b|--body)[ =]+'\''([^'\'']*)'\''/' <<<"$cmd")"
    [[ -n "$title$body" ]] && validate_text "$title

$body"
fi

exit 0
