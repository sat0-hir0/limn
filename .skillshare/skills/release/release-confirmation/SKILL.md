---
name: release-confirmation
description: After a tag lands, confirm the GitHub Release, the artifacts, and the docs site reflect the new version.
---

# Confirm release

## Use when

- A `v0.X.Y` tag has just been created by release-please.

## Contract

- GitHub Release exists with the expected CHANGELOG diff as notes.
- Documentation site (GitHub Pages) reflects the tag.
- Any per-platform artifact (later milestones) is present.

## Phase 1: Tag and Release

### Step 1-1: Confirm tag

- **Invoke**: `gh release view v0.X.Y --json tagName,name,body`

## Phase 2: Docs site

### Step 2-1: Trigger / confirm Pages build

- **Invoke**: `gh workflow view docs.yml --json runs`
- **Decide**: latest run on `main` post-tag is green.

## Phase 3: Optional artifacts

### Step 3-1: For each platform

- **Decide**: skip until M5 packaging is in place.

## Stop condition

- Tag, Release, and Pages build all confirmed.

## Boundary

- **Must** call `$post-release-followup` after this skill exits.
- **Stop** if the docs site build is red; investigate before
  announcing.

## Helper

```
scripts/release/release-confirmation.sh [TAG]
```

- TAG defaults to the most-recent tag (`git describe --tags`).
- Confirms the tag exists locally, the GitHub Release is
  published, and the latest `docs.yml` run on main is green.
- exit 0 — tag, release, and docs build all confirmed.
- exit 1 — at least one artifact missing.
- exit 2 — script error (no tag, gh not installed).

## Final Report

```yaml
release-confirmation:
  status: green | failing
  tag: v0.X.Y
  release_url: <url>
  docs_run: <id>
```
