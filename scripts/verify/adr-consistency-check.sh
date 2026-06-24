#!/usr/bin/env bash
# scripts/verify/adr-consistency-check.sh — ADR set sanity check.
#
# Checks:
#   1. Filenames under docs/adr/ match `NNNN-<slug>.md` and form a
#      contiguous sequence starting at 0001.
#   2. Every ADR has the required MADR sections.
#   3. Every ADR is listed in both docs/adr/README.md and
#      docs/README.md ADR tables.
#   4. `Superseded by ADR-NNNN` chains are bidirectional (the
#      successor ADR mentions the predecessor).
#
# Exit codes:
#   0 — clean
#   1 — inconsistencies (printed as JSON lines)
#   2 — script error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/lib.sh"

cd "$(repo_root)"

ADR_DIR="docs/adr"
[[ -d "$ADR_DIR" ]] || die "$ADR_DIR not found"

violations=0
mapfile -t ADR_FILES < <(find "$ADR_DIR" -maxdepth 1 -type f -name '[0-9]*-*.md' | sort)

# 1. Contiguous numbering.
expected=1
for file in "${ADR_FILES[@]}"; do
    basename=$(basename "$file")
    id="${basename%%-*}"
    if ! [[ "$id" =~ ^[0-9]{4}$ ]]; then
        json_finding kind=bad-id file="$file" detail="filename must start with NNNN" >&2
        violations=$((violations + 1))
        continue
    fi
    num=$((10#$id))
    if [[ "$num" -ne "$expected" ]]; then
        json_finding kind=numbering-gap file="$file" \
            expected="$(printf '%04d' "$expected")" actual="$id" \
            detail="ADR numbering must be contiguous from 0001" >&2
        violations=$((violations + 1))
    fi
    expected=$((expected + 1))
done

# 2. Required content per ADR. Limn uses MADR-light with the
# metadata in a leading `- **Field**:` block and full sections for
# the analytical parts.
REQUIRED_METADATA=("Status")   # leading `- **Status**:` line
REQUIRED_SECTIONS=(
    "## Context"
    "## Decision"
    "## Consequences"
    "## Considered Alternatives"
)
for file in "${ADR_FILES[@]}"; do
    for field in "${REQUIRED_METADATA[@]}"; do
        if ! grep -qE "^- \*\*${field}\*\*:" "$file"; then
            json_finding kind=missing-metadata file="$file" field="$field" \
                detail="MADR-light metadata line missing (leading - **${field}**:)" >&2
            violations=$((violations + 1))
        fi
    done
    for section in "${REQUIRED_SECTIONS[@]}"; do
        if ! grep -qxF "$section" "$file"; then
            json_finding kind=missing-section file="$file" section="$section" \
                detail="MADR section missing" >&2
            violations=$((violations + 1))
        fi
    done
done

# 3. Index presence in both tables.
ADR_INDEX="$ADR_DIR/README.md"
DOCS_INDEX="docs/README.md"
for file in "${ADR_FILES[@]}"; do
    id="$(basename "$file" | cut -d- -f1)"
    label="ADR-$id"
    # Match $label only when both ends are non-identifier chars so a
    # substring like `old-ADR-0001` cannot satisfy `ADR-0001`. POSIX
    # character classes keep this portable across GNU/BSD grep.
    boundary="(^|[^A-Za-z0-9_-])${label}([^A-Za-z0-9_-]|\$)"
    if [[ -f "$ADR_INDEX" ]] && ! grep -qE "$boundary" "$ADR_INDEX"; then
        json_finding kind=missing-from-index file="$file" index="$ADR_INDEX" \
            detail="add a row for $label" >&2
        violations=$((violations + 1))
    fi
    if [[ -f "$DOCS_INDEX" ]] && ! grep -qE "$boundary" "$DOCS_INDEX"; then
        json_finding kind=missing-from-index file="$file" index="$DOCS_INDEX" \
            detail="add a row for $label" >&2
        violations=$((violations + 1))
    fi
done

# 4. Superseded chain check. `grep` returning non-zero on non-match
# would otherwise trip `set -e`/`pipefail` from lib.sh; the
# `|| true` guard keeps the script flowing through ADR files that
# have no `Superseded by` line (the common case).
for file in "${ADR_FILES[@]}"; do
    successor=$(grep -oE 'Superseded by ADR-[0-9]{4}' "$file" | head -1 || true)
    [[ -z "$successor" ]] && continue
    succ_id=$(grep -oE '[0-9]{4}' <<<"$successor" || true)
    pred_id=$(basename "$file" | cut -d- -f1)
    succ_file=$(find "$ADR_DIR" -maxdepth 1 -name "${succ_id}-*.md" | head -1 || true)
    if [[ -z "$succ_file" ]]; then
        json_finding kind=broken-supersede file="$file" successor="ADR-$succ_id" \
            detail="successor ADR file not found" >&2
        violations=$((violations + 1))
        continue
    fi
    if ! grep -q "ADR-$pred_id" "$succ_file"; then
        json_finding kind=one-way-supersede predecessor="$file" \
            successor="$succ_file" \
            detail="successor must mention the predecessor" >&2
        violations=$((violations + 1))
    fi
done

if [[ "$violations" -eq 0 ]]; then
    log_ok "adr-consistency clean: ${#ADR_FILES[@]} ADR(s) verified"
    exit 0
fi

log_error "adr-consistency failed: $violations issue(s)"
exit 1
