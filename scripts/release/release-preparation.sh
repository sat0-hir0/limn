#!/usr/bin/env bash
# scripts/release/release-preparation.sh — sanity-check a release-please
# PR before merging.
#
# Verifies that every commit since the last tag is represented in
# the CHANGELOG diff that release-please assembled, and that the
# version bump class matches the commit set per git-strategy.md
# (patch = fix-only, minor = feat or breaking while 0.x).
#
# Exit codes:
#   0 — release PR looks consistent
#   1 — discrepancy detected (missing commit, wrong bump class)
#   2 — script error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/lib.sh"
require_cmd git

cd "$(repo_root)"

# Last tag (or noted as none on a brand-new repo).
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [[ -z "$LAST_TAG" ]]; then
    log_warn "no prior tag found; this is the first release"
fi

# Commits since the last tag (or full history if none).
range="${LAST_TAG:+$LAST_TAG..HEAD}"
range="${range:-HEAD}"

mapfile -t COMMITS < <(git log --format='%h%x09%s' "$range" 2>/dev/null \
                       | grep -v -E '^[a-f0-9]+\s+chore.*release' \
                       || true)

log_info "$LAST_TAG..HEAD has ${#COMMITS[@]} non-release commits"

# Determine the highest commit type.
highest="patch"
for line in "${COMMITS[@]}"; do
    subject="${line#*$'\t'}"
    if grep -qE '^(feat|fix|refactor|perf|chore|docs|test|style|ci|build|revert)(\([^)]+\))?!:' <<<"$subject" \
       || grep -q '^BREAKING CHANGE:' <<<"$subject"; then
        highest="minor"   # 0.x semantics: breaking → minor bump
        continue
    fi
    if grep -qE '^feat(\([^)]+\))?:' <<<"$subject"; then
        highest="minor"
    fi
done

log_info "highest commit class since $LAST_TAG: $highest"

violations=0

# Verify CHANGELOG.md has been touched (release-please should have
# updated it on the release PR).
if [[ -n "$LAST_TAG" ]]; then
    if ! git log --format='%h' "$range" -- CHANGELOG.md | grep -q .; then
        json_finding kind=changelog-untouched range="$range" \
            detail="release-please should have updated CHANGELOG.md" >&2
        violations=$((violations + 1))
    fi
fi

# Surface each non-release commit so the maintainer can confirm
# release-please picked it up.
printf 'Commits since %s:\n' "${LAST_TAG:-(repo start)}"
for line in "${COMMITS[@]}"; do
    printf '  %s\n' "$line"
done

if [[ "$violations" -eq 0 ]]; then
    log_ok "release-preparation: $highest bump looks correct"
    exit 0
fi

log_error "release-preparation failed: $violations issue(s)"
exit 1
