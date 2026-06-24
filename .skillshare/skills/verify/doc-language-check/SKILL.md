---
name: doc-language-check
description: Enforce that all committed docs are written in English (Limn's official documentation language per AGENTS.md).
---

# Verify doc language

## Use when

- Any change under `docs/**`, `*.md` in the repo, or `README.md`.

## Contract

- All committed Markdown is written in English.
- Personal notes that should not be public live under
  `.skillshare/maintainer-notes/` (gitignored) or `CLAUDE.local.md`
  (gitignored).

## Helper

```
scripts/verify/doc-language-check.sh
```

- exit 0: clean
- exit 1: non-English content detected (heuristic: CJK character
  density in committed `.md`)

## Stop condition

- Helper exit code.

## Boundary

- **Never** commit Japanese (or any non-English) content to public
  docs.
- **Must** keep personal-language notes in gitignored paths.

## Final Report

```yaml
doc-language-check:
  status: clean | failed
  offending_files:
    - path: <path>
      sample: <first non-English line>
```
