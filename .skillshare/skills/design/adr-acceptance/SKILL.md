---
name: adr-acceptance
description: Promote a Proposed ADR to Accepted and sync the ADR indices, Open Questions list, and trace files.
---

# Accept ADR

## Use when

- A Proposed ADR has been reviewed and the maintainer confirms
  acceptance.

## Contract

- ADR `Status` is updated to `Accepted` only with maintainer
  confirmation.
- All cross-references in `docs/` and `.skillshare/trace/` are updated
  in the same commit.

## Phase 1: Update status

### Step 1-1: Flip status

- **Output**: `docs/adr/NNNN-<slug>.md` `Status: Accepted` with
  the acceptance date.

## Phase 2: Sync indices

### Step 2-1: ADR index

- **Output**: add row to `docs/adr/README.md` index table.

### Step 2-2: Docs index

- **Output**: add row to `docs/README.md` ADR table.

### Step 2-3: Open Questions

- **Read**: `ARCHITECTURE.md` `## Open Questions`
- **Output**: remove the resolved item, link to the new ADR.

### Step 2-4: Trace file

- **Output**: update `.skillshare/trace/open-questions.md` to point
  at the new ADR id and slug.

## Phase 3: Commit

- **Output**: `docs(adr): accept ADR-NNNN <title>`

## Stop condition

- ADR status is Accepted, all four indices are in sync, the trace
  file reflects the change, and the commit is created.

## Boundary

- **Never** flip status without the maintainer's explicit
  confirmation. Confirmation must be specific to this ADR (cite the
  ADR number) and must arrive **after** the Proposed commit lands so
  the maintainer has had the chance to read the diff.
- **Never** promote in the same commit as `$adr-proposal`. The
  Proposed commit and the Accepted commit must be **separate commits
  with an actual review window between them** so reviewers can read
  the proposal before it becomes policy. A separate-commit gate is
  the minimal mechanical guard against same-session Proposed→
  Accepted bypass.
- **Never** leave an inconsistent state (e.g. ADR Accepted but
  index not updated).
- **Must** update Open Questions in the same commit.

## Helper

This skill is orchestration only; no script.

## Final Report

Update `.skillshare/records/adrs/NNNN.md`:

```yaml
status: Accepted
accepted_date: <iso8601>
indices_synced:
  - docs/adr/README.md
  - docs/README.md
  - ARCHITECTURE.md (Open Questions)
  - .skillshare/trace/open-questions.md
```
