---
name: post-release-followup
description: After a release lands, harvest learnings and queue follow-ups (skipped items, deferred decisions, regressions).
---

# Post-release follow-up

## Use when

- `$release-confirmation` reports green on a new tag.

## Contract

- Anything deferred during the release cycle is captured before
  context decays.
- `.skillshare/records/` has a release-level summary.

## Phase 1: Harvest

### Step 1-1: Read session records

- **Read**: `.skillshare/records/sessions/` since the previous tag.
- **Output**: list of "deferred / blocked / scope-break" items.

### Step 1-2: Read PR records

- **Read**: `.skillshare/records/prs/` for unresolved follow-ups.

## Phase 2: Queue

### Step 2-1: Open GitHub issues

- **Output**: one issue per actionable follow-up, labelled with
  the release tag.

### Step 2-2: Update trace files

- **Read**: `.skillshare/trace/open-questions.md`,
  `.skillshare/trace/upstream-gpui.md`,
  `.skillshare/trace/gpl-deps.md`.
- **Output**: refresh each trace file to reflect the released
  state (resolved Open Questions, gpui pinned commit, GPL
  allow-list snapshot).

## Phase 3: Memory write

### Step 3-1: Persist non-obvious learnings

- **Output**: append to project-level memory only the
  non-derivable items (incidents, judgement calls, surprises).

## Stop condition

- Issues opened, trace files updated, memory entry written.

## Boundary

- **Never** memorise items that can be re-derived from code or
  git log.
- **Must** open follow-up issues before they decay into
  "remember to do something" lore.

## Helper

```
scripts/release/post-release-followup.sh
```

(Optional helper; orchestration may run without it.)

## Final Report

```yaml
post-release-followup:
  tag: v0.X.Y
  issues_opened: [<number>...]
  trace_updated: [open-questions, upstream-gpui, gpl-deps]
  memory_appended: <name>
```
