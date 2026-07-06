# ADR-0005: Provide read-only and editing as separate views gated by a feature flag

- **Status**: Proposed
- **Date**: 2026-07-07
- **Deciders**: sat0-hir0

---

## Context

`limn-ui` originally rendered documents through `DocumentView`, a
read-only `Render` of the parsed block tree. Making the app editable
introduced `EditorView`, which wraps `gpui-component`'s `InputState`
and gives the user text entry, IME, cursor movement, selection,
clipboard, and undo/redo.

`ARCHITECTURE.md`'s Open Questions records that the quality of gpui's
Japanese IME support is not yet established; real-device verification
is needed. Per [feature-flags.md](../development/feature-flags.md),
Limn merges incomplete or unsettled features into `main` behind a
Stage 1 (hidden) flag rather than holding them on a long-lived branch.
Editing an existing document is also destructive in a way read-only
viewing is not — an IME or `InputState` regression on an unverified
platform can corrupt a user's file. This combination means the app
needs a way to keep the previously working read-only path available
while the editable path is gated.

---

## Decision

We keep `DocumentView` (read-only) and `EditorView` (editable) as two
separate `gpui` view types, and switch between them at startup with
the `edit` field of the `FeatureFlags` struct (`LIMN_FEAT_EDIT`,
Stage 1 — see [flag-inventory.md](../development/flag-inventory.md)).
When the flag is off, the window renders `DocumentView` from
`Document.blocks`; when it is on, the window renders `EditorView`
seeded from `Document.text`. Both are wrapped in `gpui_component`'s
`Root` so the same initialization path (`gpui_component::init`) covers
either branch.

---

## Consequences

### Positive

- The read-only path keeps working unconditionally; an editing defect
  cannot regress users who have not opted in
- `main` stays releasable at every commit — editing work lands
  incrementally without a long-lived branch
- The flag is a single boolean read once at startup, so the branch in
  `main.rs` is a plain `if`/`else` with no ongoing bookkeeping cost

### Negative / Trade-offs

- Two view types must both stay correct against `Document` — a schema
  change to `Block` or the raw text can require updating both
  `DocumentView` and `EditorView`
- Contributors who want to review editing behavior must remember to
  set `LIMN_FEAT_EDIT=1`

### Neutral

- `Document` already carries both `blocks` and `text`, so neither view
  needs a conversion step to read its input

---

## Considered Alternatives

### Alternative A: Replace `DocumentView` with `InputState` in a read-only mode

- Summary: Retire `DocumentView` and give `InputState` a disabled or
  read-only mode, so a single view type serves both cases.
- Reason for rejection: `InputState`'s read-only rendering behavior is
  unverified against Limn's block-based layout, and collapsing to one
  view type would remove the lightweight, non-`InputState` rendering
  path that `DocumentView` provides today.

### Alternative B: Replace `DocumentView` with `EditorView` unconditionally

- Summary: Drop the flag and always render `EditorView`.
- Reason for rejection: gpui's IME quality is not yet established
  (`ARCHITECTURE.md` Open Questions), so this would ship an unverified,
  potentially destructive input path to every user rather than keeping
  `main` releasable at every commit.

---

## Links

- Related ADR: [ADR-0001](0001-adopt-gpui.md) (delegating input handling
  to `gpui-component`'s `InputState` is decided there, not here)
- Reference: [feature-flags.md](../development/feature-flags.md)
- Reference: [flag-inventory.md](../development/flag-inventory.md)
- Reference: `ARCHITECTURE.md` Open Questions — IME Quality
