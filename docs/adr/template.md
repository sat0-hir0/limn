<!--
ADR template for Limn. See docs/adr/README.md for how to author a
new ADR; the canonical orchestrator is .skillshare/skills/design/
adr-proposal/SKILL.md.

Three things this template enforces by being read top-down:

1. ONE DECISION PER ADR. If you find yourself writing "and we also
   …" in the Decision section, split into sibling ADRs. (Canon:
   adr.github.io, joelparkerhenderson, AWS Prescriptive Guidance.)

2. NO ROLLOUT / IMPLEMENTATION PLANS. An ADR records the *reason*
   for a decision, not how it will be executed. Multi-phase
   execution plans, stage lists, running task lists, and skill
   inventories belong under docs/development/. (Canon: AWS
   Prescriptive Guidance "ADR focuses on the reason for the
   decision rather than how the team implemented it"; Microsoft
   Well-Architected Framework "Break one decision into multiple
   if … multiple phases. Log each phase as its own decision
   record.")

3. IMMUTABLE ONCE ACCEPTED. Do not rewrite an Accepted ADR. If a
   decision changes, write a new ADR and set
   `Status: Superseded by ADR-NNNN` on the predecessor. Only the
   Status / Date metadata lines may be touched on an Accepted
   record. (Canon: AWS, Martin Fowler bliki, Microsoft WAF.)
-->

# ADR-NNNN: <decision title>

<!--
Status must be exactly one of:
  Proposed
  Accepted
  Rejected
  Deprecated
  Superseded by ADR-NNNN

Deciders lists every person involved in making the decision. It is
NOT a permanent attribution to the maintainer; replace with the
actual list when contributors arrive. Keep `sat0-hir0` only when
the decision was genuinely solo.
-->

- **Status**: Proposed
- **Date**: YYYY-MM-DD
- **Deciders**: <list-everyone-involved>

---

## Context

<!--
Record WHY the decision was needed. Constraints, prior ADRs that
constrain the choice, an Open Question this resolves, the event
that triggered the proposal. Do NOT put the rollout plan here —
that goes into docs/development/.
-->

Describe the background, constraints, and problem that made this
decision necessary. Record "why this decision was needed."

---

## Decision

<!--
ONE decision per ADR. If you write "and we also …", split into a
sibling ADR.
-->

The decision that was made and the rationale for it.
Write in the form: "We adopt X because Y."

---

## Consequences

The impact of this decision. Cover both positive and negative
aspects.

### Positive

- ...

### Negative / Trade-offs

- ...

### Neutral

- ...

---

## Considered Alternatives

Alternatives that were not adopted and the reasons they were
rejected.

### Alternative A: `<name>`

- Summary: ...
- Reason for rejection: ...

### Alternative B: `<name>`

- Summary: ...
- Reason for rejection: ...

---

## Links

- Related ADR: [ADR-NNNN](NNNN-xxx.md)
- Reference: ...
