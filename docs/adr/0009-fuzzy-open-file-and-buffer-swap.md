# ADR-0009: Fuzzy open-file in the palette via vault listing and InputState buffer swap

- **Status**: Proposed
- **Date**: 2026-06-29
- **Deciders**: sat0-hir0

---

## Context

ADR-0008 established the command palette shell (Wave 5): a modal list,
opened with Ctrl/Cmd+Shift+P, with two static commands — "Open File..."
and "Open Settings" — both stopping at a placeholder log. Wave 6 makes
"Open File..." real: the user picks it, fuzzy-searches the vault's `.md`
files, and the chosen file replaces what is being edited.

Three sub-problems had to be answered to ship that one capability, and
each carries a constraint from a prior ADR:

1. **Which fuzzy-matching library?** Limn keeps its GPL surface minimal
   (ADR-0003 accepts GPL contamination only *temporarily* and only where
   forced by the `gpui`/Zed dependency tree). Zed's own `fuzzy` crate is
   GPL-3.0; pulling it in would *add* GPL surface voluntarily, against
   that policy.

2. **How does the UI get the list of files to search?** `limn-ui` must
   never call `std::fs` directly (ADR-0002): all I/O lives in
   `limn-service`. The existing `Vault` exposes `open_first_md` /
   `open_path_raw` / `save_raw`, but nothing that *enumerates* the vault.

3. **How is the editor repointed to the newly chosen file?** The editor
   (ADR-0005) is a single `gpui-component` `InputState` seeded with raw
   text, autosaving on a debounced `InputEvent::Change`. Switching files
   must not autosave the *new* file's text back to the *old* path, and
   ARCHITECTURE.md's thread model says never block the main thread.

These three are facets of the single decision "how the palette opens a
file"; they are recorded together because none stands alone and each
constrains the others (e.g. the buffer-swap choice depends on the
autosave model, the listing API shape depends on the matcher's input).

---

## Decision

We implement palette open-file as **a `limn-service` vault listing fed
into the MPL-licensed `nucleo-matcher`, whose result switches the editor
by swapping the existing `InputState`'s buffer** — not by rebuilding the
window's view tree. Concretely:

1. **Fuzzy library = `nucleo-matcher` (MPL-2.0).** MPL-2.0 is already on
   the `deny.toml` allow list, so this adds no GPL surface and needs no
   policy change. We use its convenience path:
   `Pattern::parse(query, CaseMatching::Smart, Normalization::Smart)`
   then `match_list(items, &mut matcher)` over a `Matcher::new(Config::DEFAULT)`,
   which returns the surviving items ranked by descending score. The
   corpus is the directly-listed vault (small), so matching runs inline
   on the current thread, which `match_list`'s own documentation endorses
   for small lists; a single `Matcher` is reused across keystrokes to
   avoid reallocating its scratch buffers.

2. **Vault listing API = `Vault::list_md_files() -> Vec<VaultEntry>`.**
   It returns every `.md` file *directly under* the root, sorted
   alphabetically; subdirectories are not walked (mirroring
   `open_first_md`'s shallow scan — a recursive walk is deferred to M3).
   The vault root is the **directory holding the currently open file**.
   This keeps `std::fs` inside `limn-service` (ADR-0002) and reuses the
   same shallow scan/filter/sort that `open_first_md` already relied on.

3. **File switch = `InputState::set_value` buffer swap.** Rather than
   tearing down and rebuilding the window's view tree, `EditorView`
   gains `open_file(raw, window, cx)` that keeps the same `InputState`
   entity, focus handle, and change subscription, and only swaps the
   text. Ordering is load-bearing and is the heart of the decision:
   **(a)** flush the old file synchronously and drop the debounce timer,
   **(b)** repoint `path`/`title` to the new file, **(c)** call
   `set_value`, **(d)** re-focus. `set_value` sets `emit_events = false`
   around the text replace, so it does **not** emit `InputEvent::Change`
   — seeding the new buffer therefore schedules no save, closing the
   window in which new text could be written to the old path.

   The on-switch flush runs **synchronously on the main thread**, unlike
   the debounced autosave (which is dispatched to the background
   executor). This is a deliberate, narrowly-scoped exception to
   ARCHITECTURE.md's "never block the main thread" rule: a file switch is
   a discrete user action rather than a hot path, vault files are small
   Markdown notes, and an inline flush is what makes the
   flush-before-repoint ordering airtight without an async round-trip
   that could interleave with the repoint.

The palette stays a single `ListDelegate` with two modes (Commands,
Files); confirming "Open File..." transitions to Files mode in place and
flips the list to searchable. The editor is reached through a
`WeakEntity<EditorView>` held by the delegate, avoiding a reference cycle
(editor → dialog → palette → editor).

---

## Consequences

### Positive

- No voluntary GPL surface: `nucleo-matcher` is MPL-2.0, already
  allow-listed; ADR-0003's containment stance is upheld.
- The layering rule (ADR-0002) holds: the new enumeration I/O lives in
  `limn-service`, and the UI receives plain `VaultEntry` values.
- Switching files is cheap and preserves editor identity (focus,
  undo-history reset by `set_value`, subscription), with no window/view
  rebuild and no flicker from re-creating the root.
- The autosave race is closed by construction, not by timing: `set_value`
  emitting no `Change`, plus flush-before-repoint, means neither the old
  nor the new path can receive the wrong text.

### Negative / Trade-offs

- **Provisional vault root.** Deriving the root from the open file's
  parent directory is a stopgap; it is wrong for a multi-folder vault and
  yields an empty list when the editor has no path (the ephemeral
  Welcome). This is **explicit debt** to be repaid by Wave 7's configured
  vault root.
- **Shallow listing.** Only top-level `.md` files are searchable until
  M3 adds a recursive walk; nested notes are invisible to open-file.
- **Main-thread flush on switch.** A pathologically large file would
  block the UI briefly at switch time. Accepted given the small-file
  assumption; revisit if large files become a real workload.
- **Inline matching on the main thread.** Fine for a directly-listed
  vault, but does not scale to thousands of files; the high-level
  `nucleo` crate (background matching) would be the upgrade path if the
  corpus grows (e.g. once recursion lands).

### Neutral

- "Open Settings" remains a placeholder log; its view transition is
  Wave 8 and out of scope here.
- The palette gains a second mode but stays one `ListDelegate` and one
  dialog; no new modal machinery.

---

## Considered Alternatives

### Alternative A: Zed `fuzzy` crate

- Summary: Reuse the `fuzzy` crate already vendored via the Zed
  dependency tree for matching.
- Reason for rejection: It is GPL-3.0. Adopting it would *add* GPL
  surface by choice, contradicting ADR-0003, which accepts GPL only
  where the `gpui` dependency tree forces it. `nucleo-matcher` gives
  equivalent fuzzy ranking under MPL-2.0.

### Alternative B: Rebuild the window root on file switch

- Summary: On open-file, construct a fresh `EditorView` (and `Root`) for
  the new file, replacing the window's view, as the startup path does.
- Reason for rejection: Heavier (re-creates the view tree, loses editor
  identity and focus continuity), and it still has to solve the
  old-path autosave race separately. The `set_value` swap solves the
  race intrinsically and keeps the editor entity stable.

### Alternative C: Suppress autosave with a flag instead of relying on `set_value`

- Summary: Add an "ignore next Change" guard on `EditorView` and seed the
  new text through the normal mutation path.
- Reason for rejection: Adds stateful, error-prone coordination across
  the change handler. `InputState::set_value` already guarantees no
  `Change` is emitted (`emit_events = false`), so the simpler, verified
  primitive is preferred over a hand-rolled guard.

---

## Links

- Related ADR: [ADR-0008](0008-command-palette-and-view-switching-via-gpui-actions.md) — command palette foundation this builds on
- Related ADR: [ADR-0005](0005-adopt-gpui-component-input-and-autosave-raw-text.md) — `InputState` editor + raw-text autosave the buffer swap interacts with
- Related ADR: [ADR-0003](0003-temporarily-accept-gpl-contamination.md) — GPL minimisation policy driving the matcher choice
- Related ADR: [ADR-0002](0002-three-crate-layered-architecture.md) — layering rule placing the listing I/O in `limn-service`
- Reference: `nucleo-matcher` (MPL-2.0), `Pattern::match_list` / `Matcher` API
