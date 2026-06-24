---
name: adr-proposal
description: Draft a new ADR under docs/adr/ as Status Proposed and route it through independent design review before promotion.
---

# Propose ADR

## Use when

- A design decision needs to be recorded that cannot be inferred
  from the code alone.
- A code change implicitly makes a design decision that has not been
  recorded yet.
- An Open Question in `ARCHITECTURE.md` needs a resolution.

Concrete Limn examples that warrant an ADR rather than a plain
commit:

- Choosing a property-test framework for the M2 round-trip corpus.
- Picking the LLM API surface for M5 AI integration.
- Decoupling sum_tree from the GPL-tainted zlog chain (supersedes
  ADR-0003).
- Reorganising a crate boundary not already covered by ADR-0002.

## Contract

- The proposed ADR does not contradict any Accepted ADR (verified
  by `$adr-consistency-check` in Phase 2).
- The proposed ADR is reviewed by an independent verifier before
  being promoted to Accepted.
- `ARCHITECTURE.md` Open Questions are checked and cross-linked in
  the new ADR's `## Context`.

## Phase 1: Gather context

### Step 1-1: Read existing ADRs

- **Read**: every file under `docs/adr/`.
- **Decide**: identify any prior decision that constrains or
  overlaps the new one (e.g. ADR-0001 fixes gpui as the GUI
  framework; ADR-0002 fixes the three-crate layout; ADR-0003
  bounds the GPL allow-list).
- **Output**: list of related ADR ids in the new ADR's
  `## Context` and (if any supersede) the precise wording for the
  `Status: Superseded by ADR-NNNN` line on the old ADR.
- **Next**: Step 1-2.

### Step 1-2: Check Open Questions

- **Read**: `ARCHITECTURE.md` `## Open Questions`.
- **Decide**: does this proposal resolve one of them (AI Integration
  Approach / Scope of `/` / Completion Breathing / AI Model
  Selection / Graph View Layout / IME Quality)?
- **Output**: Open Question reference (if any) in `## Context`.
- **Next**: Step 1-3.

### Step 1-3: Scan recent code

- **Read**: `git log --oneline -20` and recent diffs against main.
- **Decide**: has the decision already been implicitly made (i.e.
  a `feat:` commit landed without an ADR)?
- **Output**: commit/PR references in `## Context`. If the decision
  is already in code, the ADR records the rationale retroactively.
- **Next**: Phase 2.

## Phase 2: Detect conflicts

### Step 2-1: Consistency check

- **Invoke**: `$adr-consistency-check`
  (`scripts/verify/adr-consistency-check.sh`).
- **Stop on**: any contradiction with an Accepted ADR. Resolve by
  superseding the older ADR, not by ignoring it.

### Step 2-2: Trace Open Question linkage

- **Read**: `.skillshare/trace/open-questions.md`.
- **Decide**: which Open Question this proposal addresses (or
  whether it adds a new one).
- **Output**: mapping update for that trace file in the same commit
  as the ADR draft.

## Phase 3: Draft

### Step 3-1: Create the file

- **Read**: `docs/adr/template.md`.
- **Output**: `docs/adr/NNNN-<slug>.md` with the leading
  `- **Status**: Proposed` metadata line plus the four MADR
  sections (`## Context`, `## Decision`, `## Consequences`,
  `## Considered Alternatives`). Limn uses MADR-light: the status
  lives in a metadata line, not a section.
- **Confirm**: the draft addresses exactly one decision. If you
  catch yourself writing "and we also …" inside the Decision
  section, split now — open a sibling ADR for each concern.

### Step 3-2: Commit

- **Output**: `docs(adr): propose ADR-NNNN <title>` (Conventional
  Commits format; the prefix is `docs(adr)` so release-please
  classifies it correctly).

## Stop condition

- The Proposed ADR is committed in its own commit (the `docs(adr):
  propose ADR-NNNN <title>` commit from Step 3-2). **Stop here**.
- Promotion to Accepted is a separate turn handled by
  `$adr-acceptance`. Do not chain into it. An autonomous-mode
  blanket approval (e.g. "go ahead", "proceed", "ざっくり進めて") does
  **not** authorise promotion — see `$adr-acceptance` Boundary for
  the same-turn-promotion guard.

## Boundary

- **Never** promote `Status` from Proposed to Accepted without the
  maintainer's explicit confirmation.
- **Never** mix multiple decisions in one ADR. If a draft covers
  more than one concern (e.g. "use library X" plus "deploy via Y"
  plus "stage rollout Z"), split it into one ADR per concern. OSS
  canon agrees on this: adr.github.io "captures a single AD";
  joelparkerhenderson "Each ADR should be about one AD, not
  multiple ADs"; Microsoft Well-Architected Framework "Break one
  decision into multiple if … multiple phases".
- **Never** include implementation rollout plans, stage-by-stage
  execution sequences, running task lists, or skill inventories
  inside an ADR. An ADR records the **reason** for a decision,
  not how it will be executed. Rollout plans live under
  `docs/development/`.
- **Do not** rewrite an Accepted ADR. ADRs are immutable once
  accepted; supersede with a new ADR and link
  `Status: Superseded by ADR-NNNN` on the predecessor. Only the
  `Status` / `Date` metadata lines may be touched on an Accepted
  record.
- **Must** cite the implementation file/line that triggered the
  decision (if any) so a future reader can locate it.
- **Stop** if `$adr-consistency-check` finds a contradiction —
  escalate to the maintainer before drafting further.
- **Stop** if the proposal is forward-looking or exploratory
  rather than a recorded decision. ADRs are decision records, not
  RFCs; open a GitHub Discussion or design sketch first, return
  to `$adr-proposal` once a decision has actually been made.

## Helper

This skill is orchestration only; no script.

## Final Report

Write to `.skillshare/records/adrs/NNNN.md`:

```yaml
---
adr: NNNN
slug: <kebab>
status: Proposed
created: <iso8601>
---

## Context summary
…

## Related ADRs
- supersedes / superseded by / see also: …

## Open Question linkage
- ARCHITECTURE.md Open Question: …
```
