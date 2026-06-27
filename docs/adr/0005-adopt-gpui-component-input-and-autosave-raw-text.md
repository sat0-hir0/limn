# ADR-0005: Adopt gpui-component `InputState` for editing and autosave raw text

- **Status**: Proposed
- **Date**: 2026-06-28
- **Deciders**: sat0-hir0

---

## Context

M1 ships a read-only `DocumentView` (`crates/limn-ui/src/lib.rs`) that
renders a parsed `Vec<Block>`. M2 (this work, Issue #3) must make the
editor writable: character input, Japanese IME composition, cursor
movement, range selection, delete, copy/cut/paste, undo/redo, and
autosave back to the opened `.md`.

Two constraints shape the decision:

1. **`limn_core::markdown::serialize` is unimplemented**
   (`crates/limn-core/src/markdown.rs` returns
   `unimplemented!("serialize: lands in M2 together with the real
   parser")`). A full block round-trip (`Vec<Block>` → Markdown) is a
   separate, larger piece of work — the M1 parser only recognises ATX
   headings and paragraphs, so a serialize built on it would be lossy
   for any real `.md`.

2. **The "IME Quality" Open Question** (`ARCHITECTURE.md` →
   Open Questions → IME Quality) calls for real-device verification of
   gpui's Japanese IME support early in M2. We need an input path that
   exercises gpui's `EntityInputHandler` IME surface rather than a
   hand-rolled keystroke buffer.

The `gpui-component` crate (already pinned in `workspace.dependencies`
but not yet a dependency of any crate) provides an `InputState` /
`Input` widget whose state machine already covers character input,
cursor, selection, undo/redo, and IME composition via gpui's
`EntityInputHandler`. `docs/design/basic-features.md` already names the
`gpui-component` editor example as "the right starting point" for the
★★★ baseline features.

---

## Decision

We adopt `gpui-component`'s `InputState` as the editor's text buffer,
and **autosave the buffer's raw text directly to the opened `.md`**,
bypassing the block tree for the write path.

Concretely:

- The editable view holds a `gpui-component` `InputState` seeded with
  the file's raw UTF-8 text (not the parsed `Vec<Block>`).
- Autosave writes `InputState`'s current text verbatim through a new
  `limn-service` write path. No `Vec<Block>` → Markdown serialization
  is involved.

We choose this because (a) it reuses a maintained, IME-aware editing
state machine instead of reinventing cursor/selection/undo, directly
addressing the IME Quality Open Question, and (b) treating the file as
raw text for save makes editing lossless today, without blocking on the
full block serializer that `limn-core` defers to a later milestone.

The block round-trip (`serialize`) remains future work; this ADR does
not implement or depend on it.

---

## Consequences

### Positive

- Lossless save from day one: whatever the user types is what lands on
  disk, with no parser/serializer fidelity gap.
- IME, cursor, selection, and undo/redo come from a maintained
  component, shrinking the surface limn must own and test.
- Exercises gpui's real IME path, giving the IME Quality Open Question
  a concrete artifact to verify against.

### Negative / Trade-offs

- The rendered live view and the editing buffer diverge in
  representation (blocks for display history vs. raw text for editing).
  Reconciling them — instant Markdown rendering as you type — is
  deferred and will need its own decision.
- Adds `gpui-component` as a real dependency, pulling its transitive
  tree into the build and `cargo-deny` license review.
- Couples the editor to a pre-1.0, git-pinned component API; upgrades
  must move the pinned rev in lockstep with gpui (already a documented
  constraint in the workspace `Cargo.toml`).

### Neutral

- `limn_core::markdown::serialize` stays `unimplemented!`; this ADR
  neither needs nor forbids its later implementation.
- The write path is added to `limn-service` (I/O ownership per
  ADR-0002); `limn-ui` still performs no direct `std::fs`.

---

## Considered Alternatives

### Alternative A: Hand-rolled text buffer in limn-core

- Summary: Implement a custom editable buffer (rope/gap buffer) plus
  cursor, selection, and undo model in `limn-core` (std-only), and
  wire gpui's `EntityInputHandler` to it directly.
- Reason for rejection: Re-implements an IME-correct editing state
  machine that `gpui-component` already provides and tests. High risk
  on the exact axis the IME Quality Open Question flags, with no
  near-term payoff over reuse.

### Alternative B: Save via block round-trip (`Vec<Block>` → Markdown)

- Summary: Parse edits into blocks and serialize blocks back to
  Markdown on save.
- Reason for rejection: Requires implementing
  `limn_core::markdown::serialize` first, and the M1 block model only
  covers headings/paragraphs — saving any richer `.md` would be lossy.
  Blocks the writable editor on a much larger parser/serializer effort.

---

## Links

- Related ADR: [ADR-0001](0001-adopt-gpui.md) (gpui as GUI framework),
  [ADR-0002](0002-three-crate-layered-architecture.md) (I/O confined to
  limn-service)
- Open Question: `ARCHITECTURE.md` → Open Questions → IME Quality
- Trigger: `crates/limn-core/src/markdown.rs` (`serialize` unimplemented),
  `crates/limn-ui/src/lib.rs` (read-only `DocumentView`)
- Reference: `docs/design/basic-features.md` (gpui-component editor
  example as the ★★★ baseline starting point)
