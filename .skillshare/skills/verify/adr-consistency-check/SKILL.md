---
name: adr-consistency-check
description: Detect contradictions and gaps across the ADR set (status, numbering, cross-refs, superseded chains).
---

# Verify ADR consistency

## Use when

- `docs/adr/**` changed.
- `$adr-proposal` Phase 2 dispatch.

## Contract

- Every ADR file has a well-formed `Status`, `Title`, `Context`,
  `Decision`, `Consequences`, `Considered Alternatives`.
- ADR numbering is contiguous starting at 0001.
- No two Accepted ADRs contradict each other on the same topic.
- `Superseded by` chains are bidirectional.

## Helper

```
scripts/verify/adr-consistency-check.sh
```

- exit 0: clean
- exit 1: list of inconsistencies (missing section, broken link,
  numbering gap, contradictory pair)

## Stop condition

- Helper exit code.

## Boundary

- **Never** allow two contradictory Accepted ADRs to coexist —
  one must be superseded.
- **Must** keep `docs/adr/README.md` and `docs/README.md` in sync
  with the actual ADR set.

## Final Report

```yaml
adr-consistency-check:
  status: clean | failed
  issues:
    - kind: contradiction | missing-section | numbering-gap | broken-supersede
      adr: <id>
      detail: <text>
```
