#!/usr/bin/env bash
# scripts/verify/changelog-untouched-check.sh — CHANGELOG.md is owned by
# release-please. Any human-authored commit on the current branch
# that modifies it is a violation.
#
# Heuristic for "release-please-authored": the commit's author email
# matches one of the known release-please bot patterns.
#
# Exit codes:
#   0 — clean
#   1 — human commit touched CHANGELOG.md
#   2 — script error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/lib.sh"
require_cmd git

cd "$(repo_root)"

MAIN="${LIMN_MAIN_BRANCH:-main}"
BOT_EMAILS_RE='release-please|github-actions\[bot\]|googleapis-bot|google-cla'

if ! git rev-parse --verify "$MAIN" >/dev/null 2>&1; then
    log_warn "main branch $MAIN not present; nothing to compare"
    log_ok "changelog-untouched clean (vacuously)"
    exit 0
fi

range="$MAIN..HEAD"
violations=0

# git log -- CHANGELOG.md restricts to commits that touched the file.
while IFS=$'\t' read -r sha email subject; do
    [[ -z "$sha" ]] && continue
    if [[ "$email" =~ $BOT_EMAILS_RE ]]; then
        continue
    fi
    json_finding kind=human-edit sha="$sha" author="$email" subject="$subject" \
        detail="CHANGELOG.md is managed by release-please; revert this hunk" >&2
    violations=$((violations + 1))
done < <(git log --format='%h%x09%ae%x09%s' "$range" -- CHANGELOG.md 2>/dev/null || true)

if [[ "$violations" -eq 0 ]]; then
    log_ok "changelog-untouched clean"
    exit 0
fi

log_error "changelog-untouched failed: $violations human-authored edit(s)"
exit 1
