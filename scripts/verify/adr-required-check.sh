#!/usr/bin/env bash
# scripts/verify/adr-required-check.sh — flag PRs that change public
# behaviour without recording the decision as an ADR.
#
# Triggers (any one is enough on its own):
#   - a new `pub mod` declaration under crates/*/src/
#   - a new top-level `[[bin]]` entry in a Cargo.toml
#   - a change to argv handling in any `main.rs`
#   - addition of a new `Launch::*` or app-mode enum variant
#
# Passes if the same PR adds at least one new ADR (= a new file
# matching docs/adr/NNNN-*.md, where NNNN > the previous max).
#
# This is advisory: exit 1 means "consider whether you should be
# writing an ADR", not "the build is broken". The maintainer can
# override by adding `[skip-adr-required]` to any commit message
# on the branch — useful when the change is a refactor that only
# moves code without introducing a new decision.
#
# Exit codes:
#   0 — no public-behaviour change, or an ADR was added, or skip flag set
#   1 — public-behaviour change without an ADR
#   2 — script error
#
# Environment:
#   ADR_REQUIRED_BASE   base ref for the diff (default: origin/main, fallback main)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/lib.sh"
require_cmd git

cd "$(repo_root)"

BASE_REF="${ADR_REQUIRED_BASE:-origin/main}"
if ! git rev-parse --verify "$BASE_REF" >/dev/null 2>&1; then
    BASE_REF="main"
    if ! git rev-parse --verify "$BASE_REF" >/dev/null 2>&1; then
        log_info "no base ref available; assuming fresh repo"
        log_ok "adr-required clean (no diff to evaluate)"
        exit 0
    fi
fi

# Manual override via commit message tag.
if git log "$BASE_REF"..HEAD --pretty=%B 2>/dev/null | grep -qF '[skip-adr-required]'; then
    log_info "skip flag found in commit messages; not enforcing"
    log_ok "adr-required skipped"
    exit 0
fi

# Collect public-behaviour signals from the diff.
DIFF=$(git diff "$BASE_REF"...HEAD 2>/dev/null || git diff "$BASE_REF" 2>/dev/null || true)
if [[ -z "$DIFF" ]]; then
    log_ok "adr-required clean (empty diff)"
    exit 0
fi

signals=()

# Signal 1: new `pub mod` in crates/*/src/.
if grep -E '^\+\s*pub\s+mod\s+[a-z_]+' <<<"$DIFF" \
   | grep -vE '^\+\+\+' >/dev/null 2>&1; then
    while IFS= read -r line; do
        signals+=("new-pub-mod: ${line# +}")
    done < <(grep -E '^\+\s*pub\s+mod\s+[a-z_]+' <<<"$DIFF" \
             | grep -vE '^\+\+\+' \
             | head -5)
fi

# Signal 2: new [[bin]] in any Cargo.toml.
if grep -E '^\+\[\[bin\]\]' <<<"$DIFF" >/dev/null 2>&1; then
    signals+=("new-bin: a new [[bin]] target was added to Cargo.toml")
fi

# Signal 3: argv handling change in main.rs files. A loose grep on
# `std::env::args` or `clap::Parser` lines that are additions.
if grep -E '^\+.*(std::env::args|clap::Parser)' <<<"$DIFF" \
   | grep -vE '^\+\+\+' >/dev/null 2>&1; then
    signals+=("argv-change: main.rs argv handling changed")
fi

# Signal 4: a new `Launch::Foo` / `Mode::Foo` style enum variant in
# UI code — added inside an enum block that already names "Launch"
# or "Mode" in the same file.
if grep -E '^\+\s*(Launch|Mode|Workspace)::\w+' <<<"$DIFF" \
   | grep -vE '^\+\+\+' >/dev/null 2>&1; then
    signals+=("new-app-mode: a Launch/Mode variant was introduced")
fi

# Quick exit if no signals.
if [[ "${#signals[@]}" -eq 0 ]]; then
    log_ok "adr-required clean (no public-behaviour signals)"
    exit 0
fi

# Was a new ADR added in the same diff?
if git diff --name-only --diff-filter=A "$BASE_REF"...HEAD 2>/dev/null \
   | grep -E '^docs/adr/[0-9]{4}-' >/dev/null 2>&1; then
    log_ok "adr-required clean: public-behaviour signals present, but a new ADR was added"
    exit 0
fi

# Public-behaviour signals without an ADR.
for s in "${signals[@]}"; do
    json_finding kind=missing-adr signal="$s" \
        detail="public-behaviour change detected; consider \$adr-proposal or add [skip-adr-required] to the commit message" >&2
done
log_error "adr-required failed: ${#signals[@]} public-behaviour signal(s) without an ADR"
exit 1
