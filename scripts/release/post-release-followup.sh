#!/usr/bin/env bash
# scripts/release/post-release-followup.sh — after a release lands,
# harvest open follow-ups from this release cycle's records into a
# single Markdown file under .skillshare/records/ so they do not
# decay into "remember to do something" lore.
#
# Open follow-ups are anything written to
# .skillshare/records/{prs,sessions}/*.md with a `- follow-up:` line.
#
# Exit codes:
#   0 — followup summary written
#   1 — cannot read the records directory
#   2 — script error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/lib.sh"
require_cmd git

cd "$(repo_root)"

TAG="${1:-}"
[[ -z "$TAG" ]] && TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "next")
log_info "harvesting follow-ups for $TAG"

OUT=".skillshare/records/followups-${TAG}.md"
mkdir -p ".skillshare/records"

{
    printf '%s\n' "---"
    printf 'tag: %s\n' "$TAG"
    printf 'generated: %s\n' "$(git log -1 --format=%cs 2>/dev/null || echo "n/a")"
    printf '%s\n\n' "---"
    printf '# Follow-ups since previous tag\n\n'

    found_any=0
    for path in .skillshare/records/prs/*.md \
                .skillshare/records/sessions/*.md \
                .skillshare/records/adrs/*.md; do
        [[ -f "$path" ]] || continue
        mapfile -t lines < <(grep -nE '^[[:space:]]*-?[[:space:]]*follow-up:' "$path" 2>/dev/null || true)
        [[ "${#lines[@]}" -eq 0 ]] && continue
        found_any=1
        printf '## %s\n\n' "$path"
        for line in "${lines[@]}"; do
            printf '  %s\n' "$line"
        done
        printf '\n'
    done

    if [[ "$found_any" -eq 0 ]]; then
        printf '_No follow-ups recorded this cycle._\n'
    fi
} > "$OUT"

log_ok "post-release-followup: wrote $OUT"
exit 0
