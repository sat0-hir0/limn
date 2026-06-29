# ADR-0006: Reserved (number skipped)

- **Status**: Rejected
- **Date**: 2026-06-29
- **Deciders**: sat0-hir0

---

## Context

ADR numbering allocates 0006 chronologically between ADR-0005 (gpui-component
`InputState` adoption) and ADR-0007 (user configuration file). During the
M2 wave sequence the slot was tentatively held for a decision on runtime
icon loading on Windows and macOS, but the question was resolved without
a record: the existing gpui asset path was kept, and no architectural
choice needed to be captured.

By the time ADR-0007 was authored, downstream ADRs (0008 — command palette
and view switching, 0009 — fuzzy open-file and buffer swap, 0010 —
settings view) had already been planned and cross-referenced against the
0007+ numbers in commit messages, the SSOT progress log
(`~/.claude/state/slice-editable-editor-shell.md`), and the code itself.
Renumbering them to close the gap would have rewritten history that
external readers were already linking against.

The `adr-consistency-check` verifier requires ADR filenames to form a
contiguous sequence starting at 0001. To keep that invariant honest
without renumbering live ADRs, this record explicitly fills slot 0006 as
a non-decision.

## Decision

ADR-0006 is **reserved as a non-decision**. No technical position is
recorded under this number; the slot exists only to keep the sequence
contiguous so the verifier can run without a numbering exception.

Future ADRs continue from ADR-0011 onwards. Authors must not reuse
0006 — it is now permanently a "this slot is intentionally blank"
marker.

## Consequences

- The verifier's "contiguous numbering from 0001" check passes.
- Anyone scanning the ADR directory in numerical order will see a clear
  "no decision here" notice rather than wondering whether 0006 went
  missing or was lost.
- The cost of resolving the gap is one extra record. The alternative —
  renumbering 0007 through 0010 — would have required rewriting commit
  messages, code comments referencing those ADR numbers, and the SSOT
  log, all to remove a one-line note. The trade is unfavourable.
- `Rejected` is the closest match among the verifier's allowed status
  values for "no decision was made here". It is not a literal rejection
  of a proposal; the slot was never carried to a decision in the first
  place.

## Considered Alternatives

### Renumber ADR-0007 through ADR-0010

Rejected. Every commit message authoring those ADRs already names the
target number ("docs(adr): propose ADR-0007 …", "ADR-0008 Proposed
committed", etc.) and the in-code references (e.g.
`crates/limn-ui/src/actions.rs` referring to `ADR-0010`) were chosen to
match the existing filenames. Renumbering would touch ten or more files
and rewrite recent history to fix a one-record gap. Not worth it.

### Patch the verifier to tolerate intentional gaps

Rejected. The verifier's contiguous-numbering rule exists precisely so
"missing" ADRs are obvious. Adding an allowlist or a comment-driven
opt-out shifts the burden from the ADR set to the verifier and weakens
the rule everywhere. Filling the slot keeps the rule strict and visible.

### Leave the gap and skip the verifier locally

Rejected. The verifier runs in the `pre-push` hook and CI; it is the
mechanism that keeps the ADR set readable. Bypassing it would mean
either disabling the hook (loses the safety) or carrying a
session-by-session workaround (loses portability).

---

## See also

- [docs/adr/README.md](README.md) — ADR index. The note in the table
  there explains the same gap from the index side.
