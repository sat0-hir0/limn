#!/usr/bin/env bash
# scripts/verify/doc-language-check.sh — committed Markdown must be in
# English (Limn's official documentation language).
#
# Heuristic: count CJK Unified Ideographs, Hiragana, Katakana,
# Hangul, and CJK punctuation. If a committed .md file contains
# more than a small tolerance worth of those code points, fail.
# Gitignored personal-note paths (.skillshare/maintainer-notes/,
# CLAUDE.local.md, AGENTS.local.md) are skipped.
#
# Exit codes:
#   0 — clean
#   1 — non-English content detected in committed files
#   2 — script error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/lib.sh"

cd "$(repo_root)"

# Use python for reliable Unicode codepoint counts in Git Bash on
# Windows. Try python3 first (the canonical name on Linux/macOS), then
# python. On Windows the unqualified `python` is sometimes an App
# Execution Alias that just prints a message — guard against that by
# probing with `--version`.
PY=""
for candidate in python3 python py; do
    if command -v "$candidate" >/dev/null 2>&1 \
       && "$candidate" --version >/dev/null 2>&1; then
        PY="$candidate"
        break
    fi
done
[[ -n "$PY" ]] || die "python3 / python required for Unicode-safe scanning"

# CJK character tolerance per file. Set above zero to allow a small
# number of code-block examples or quoted terms.
TOLERANCE="${DOC_LANG_TOLERANCE:-3}"

# Files this script scans. Personal-language scratch is excluded.
mapfile -t TARGETS < <(git ls-files '*.md' | grep -vE '(^|/)CLAUDE\.local\.md$|(^|/)AGENTS\.local\.md$|^\.skillshare/maintainer-notes/' || true)

violations=0

for file in "${TARGETS[@]}"; do
    count=$("$PY" - "$file" <<'PYEOF'
import sys
path = sys.argv[1]
total = 0
samples = []
with open(path, encoding="utf-8", errors="replace") as fh:
    for lineno, line in enumerate(fh, start=1):
        cjk = 0
        for ch in line:
            cp = ord(ch)
            if (
                0x3000 <= cp <= 0x303F      # CJK punctuation
                or 0x3040 <= cp <= 0x309F   # Hiragana
                or 0x30A0 <= cp <= 0x30FF   # Katakana
                or 0x3400 <= cp <= 0x4DBF   # CJK Ext A
                or 0x4E00 <= cp <= 0x9FFF   # CJK Unified Ideographs
                or 0xAC00 <= cp <= 0xD7AF   # Hangul Syllables
                or 0xFF00 <= cp <= 0xFFEF   # Halfwidth/fullwidth
            ):
                cjk += 1
        if cjk and len(samples) < 1:
            samples.append((lineno, line.strip()))
        total += cjk
print(total)
for lineno, line in samples:
    print(f"{lineno}\t{line}")
PYEOF
)
    cjk_count=$(head -n1 <<<"$count")
    sample=$(sed -n '2p' <<<"$count")
    if [[ -z "$cjk_count" ]] || ! [[ "$cjk_count" =~ ^[0-9]+$ ]]; then
        continue
    fi
    if [[ "$cjk_count" -gt "$TOLERANCE" ]]; then
        first_line=$(cut -f1 <<<"$sample")
        first_text=$(cut -f2- <<<"$sample")
        json_finding kind=non-english file="$file" \
            cjk_chars="$cjk_count" \
            first_line="${first_line:-?}" \
            sample="${first_text:0:80}" \
            detail="committed Markdown must be in English" >&2
        violations=$((violations + 1))
    fi
done

if [[ "$violations" -eq 0 ]]; then
    log_ok "doc-language clean: all committed Markdown is English-only"
    exit 0
fi

log_error "doc-language failed: $violations file(s) with non-English content"
exit 1
