---
name: release-preparation
description: Inspect the release-please PR before merging — verify CHANGELOG accuracy, version bump correctness, and runbook compliance.
---

# Prepare release

## Use when

- release-please opens or updates the `chore: release v0.X.Y` PR.
- A milestone version is being cut.

## Contract

- Every Conventional Commit since the previous tag is reflected in
  the CHANGELOG draft.
- Version bump matches `git-strategy.md` rules (minor for
  feat/breaking while 0.x, patch for fix-only).
- `docs/maintainer-runbook/release-public.md` steps are honoured.

## Phase 1: Verify CHANGELOG

### Step 1-1: Diff commits vs CHANGELOG

- **Read**: `git log <last-tag>..HEAD` and the release-please PR
  CHANGELOG diff.
- **Decide**: every user-visible commit listed?

## Phase 2: Verify version

### Step 2-1: Check version bump

- **Decide**: bump class (patch / minor / major) matches the
  highest commit type.

## Phase 3: Runbook

### Step 3-1: Walk the runbook

- **Read**: `docs/maintainer-runbook/release-public.md`
- **Decide**: any step still outstanding?

## Stop condition

- CHANGELOG accurate, version correct, runbook satisfied.

## Boundary

- **Never** edit `CHANGELOG.md` by hand to fix a discrepancy;
  add a missing commit or amend metadata instead and let
  release-please regenerate.
- **Must** call `$release-confirmation` after the tag lands.

## Helper

This skill is orchestration only; no script.

## Final Report

```yaml
release-preparation:
  status: approved | needs-fix
  version: v0.X.Y
  commits_reflected: <n>
  commits_missing: [<sha>...]
  runbook_open: [<step>...]
```
