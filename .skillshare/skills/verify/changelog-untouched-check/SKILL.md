---
name: changelog-untouched-check
description: Fail if CHANGELOG.md was edited by a human commit; release-please owns the file.
---

# Verify CHANGELOG untouched

## Use when

- Any change (always).

## Contract

- `CHANGELOG.md` is modified only by release-please commits.
- Human commits never touch it.

## Helper

```
scripts/verify/changelog-untouched-check.sh
```

- exit 0: clean (the file was not modified, **or** the only
  modifier is release-please).
- exit 1: a human-authored commit modified the file.

## Stop condition

- Helper exit code.

## Boundary

- **Never** edit `CHANGELOG.md` by hand. Use Conventional Commits
  on regular commits; release-please will update the changelog
  via its release PR.

## Final Report

```yaml
changelog-untouched-check:
  status: clean | failed
  offending_commit: <sha>
```
