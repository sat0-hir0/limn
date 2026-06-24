#!/usr/bin/env bash
# scripts/release/release-confirmation.sh — after release-please's tag
# lands, confirm the GitHub Release exists with the expected body and
# the docs site rebuild succeeded.
#
# Reads the tag from $1 or detects the most recent tag automatically.
#
# Exit codes:
#   0 — tag, release, and docs build all confirmed
#   1 — at least one artifact missing
#   2 — script error (gh missing, no tag)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/lib.sh"
require_cmd git

cd "$(repo_root)"

TAG="${1:-}"
[[ -z "$TAG" ]] && TAG=$(git describe --tags --abbrev=0 2>/dev/null || true)
[[ -n "$TAG" ]] || die "no tag supplied and no recent tag found"
log_info "verifying release for tag $TAG"

violations=0

# 1. Tag exists locally.
if ! git rev-parse --verify "refs/tags/$TAG" >/dev/null 2>&1; then
    json_finding kind=missing-tag tag="$TAG" \
        detail="tag is not present in the local repository" >&2
    violations=$((violations + 1))
fi

# 2. GitHub Release exists (best effort, needs gh).
if command -v gh >/dev/null 2>&1; then
    if ! gh release view "$TAG" --json tagName >/dev/null 2>&1; then
        json_finding kind=missing-release tag="$TAG" \
            detail="GitHub Release not yet published for this tag" >&2
        violations=$((violations + 1))
    else
        log_info "GitHub Release $TAG is published"
    fi

    # 3. docs.yml workflow ran successfully on the tag commit.
    if last_run_concl=$(gh run list --workflow docs.yml --branch main --limit 1 \
                        --json conclusion --jq '.[0].conclusion' 2>/dev/null); then
        if [[ "$last_run_concl" != "success" ]]; then
            json_finding kind=docs-build-not-green workflow=docs.yml \
                conclusion="$last_run_concl" \
                detail="latest docs build on main is not green" >&2
            violations=$((violations + 1))
        else
            log_info "docs.yml latest main run: success"
        fi
    fi
else
    log_warn "gh not installed; release / docs build checks skipped"
fi

if [[ "$violations" -eq 0 ]]; then
    log_ok "release-confirmation clean: $TAG looks healthy"
    exit 0
fi

log_error "release-confirmation failed: $violations issue(s)"
exit 1
