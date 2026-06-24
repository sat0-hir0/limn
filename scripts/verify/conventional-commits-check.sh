#!/usr/bin/env bash
# scripts/verify/conventional-commits-check.sh — validate branch name,
# commits on the current branch, and PR title (if available via gh)
# against the Conventional Commits convention that release-please
# consumes.
#
# Defaults:
#   - Range: <main-branch>..HEAD
#   - PR title check: only when gh is installed AND
#     CONVENTIONAL_COMMITS_PR is set (or `gh pr view` succeeds).
#
# Exit codes:
#   0 — clean
#   1 — at least one violation
#   2 — script error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/lib.sh"
require_cmd git

cd "$(repo_root)"

# Conventional Commits type set used by release-please for Limn.
CC_RE='^(feat|fix|chore|docs|refactor|test|style|perf|ci|build|revert)(\([^)]+\))?!?: .+'

# Branch prefix allow-list mirrored from docs/development/git-strategy.md.
BRANCH_RE='^(feat|fix|docs|chore|refactor|test|perf|ci|build|revert)/[a-z0-9][a-z0-9-]*$'

MAIN="${LIMN_MAIN_BRANCH:-main}"
violations=0

# 1. Branch name.
branch=$(git rev-parse --abbrev-ref HEAD)
if [[ "$branch" == "$MAIN" ]]; then
    log_info "on $MAIN; skipping branch-name check"
elif [[ ! "$branch" =~ $BRANCH_RE ]]; then
    json_finding kind=branch-name value="$branch" \
        detail="branch must be <type>/<kebab-topic>" >&2
    violations=$((violations + 1))
fi

# 2. Commits in the range <main>..HEAD.
range="$MAIN..HEAD"
if git rev-parse --verify "$MAIN" >/dev/null 2>&1; then
    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        sha="${line%% *}"
        subject="${line#* }"
        if [[ ! "$subject" =~ $CC_RE ]]; then
            json_finding kind=commit sha="$sha" subject="$subject" \
                detail="must match Conventional Commits" >&2
            violations=$((violations + 1))
        fi
    done < <(git log --format='%h %s' "$range" 2>/dev/null || true)
else
    log_warn "branch $MAIN not present locally; skipping commit-range check"
fi

# 3. PR title (best-effort; needs gh and a PR for the current branch).
if command -v gh >/dev/null 2>&1; then
    if pr_title=$(gh pr view --json title --jq '.title' 2>/dev/null); then
        if [[ -n "$pr_title" && ! "$pr_title" =~ $CC_RE ]]; then
            json_finding kind=pr-title value="$pr_title" \
                detail="PR title becomes the squash commit; must match Conventional Commits" >&2
            violations=$((violations + 1))
        fi
    fi
fi

if [[ "$violations" -eq 0 ]]; then
    log_ok "conventional-commits clean"
    exit 0
fi

log_error "conventional-commits failed: $violations violation(s)"
exit 1
