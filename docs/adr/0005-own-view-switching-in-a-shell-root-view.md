# ADR-0005: Own View Switching in a Shell Root View

- **Status**: Proposed
- **Date**: 2026-08-22
- **Deciders**: sat0-hir0

---

## Context

`crates/limn-ui/src/main.rs` currently opens its window with
`DocumentView` as the root view. `DocumentView` is read-only and holds
no notion of "which view is currently shown" — there is exactly one
view, and the window has no root above it to hold anything else.

Keyboard-first interaction is a core requirement (`ARCHITECTURE.md`,
`docs/design/basic-features.md`: "Keyboard shortcut system — Foundation
of the keyboard-first experience"). Both ADR-0001 (adopt gpui) and
ADR-0002 (three-crate layering, `limn-ui` as the sole gpui-dependent
crate) constrain the choice: the mechanism has to work inside gpui and
stay inside `limn-ui`.

gpui only dispatches an action along the currently focused element's
path (`crates/gpui/examples/testing.rs` in the pinned checkout shows
the pattern: `key_context` + `on_action` + `track_focus` on the element
that owns the shortcuts). A shortcut bound on one view's element tree
is invisible to every other view, because only one element tree is
focused at a time. Adding a second top-level view (a command palette, a
settings screen) needs one owner above both that can receive the
switch action regardless of which view currently has focus, and that
can render whichever view is current.

## Decision

View switching is owned by the window's root view, `Shell`. `Shell`
holds the current `ShellView` (`Editor`, `Palette`, or `Settings`),
renders the one that is current, and owns the `FocusHandle` the window
focuses on open. The key bindings that switch views are scoped to
`Shell`'s own key context (`"Shell"`), not to any individual view's
context, because `Shell` is the element that is always on the focus
path regardless of which child view is showing. The keystroke for each
view and its destination are declared together in one constant table,
`SHORTCUTS`, in `crates/limn-ui/src/shell.rs` — one row per shortcut,
so the full shortcut inventory is readable in one place instead of
scattered across each view's own bindings.

## Consequences

### Positive

- One source of truth for which view is on screen: `Shell::view`.
  Nothing else tracks it independently.
- The shortcut inventory is one table (`SHORTCUTS`) instead of being
  spread across each view's own key bindings.
- `DocumentView` is unchanged: it stays a plain `Render` type with no
  knowledge of `Shell` or of the other views, so it keeps its existing
  test coverage (`tests/e2e_render.rs`) untouched.
- There is exactly one focus path for keyboard shortcuts: through
  `Shell`. A future view does not need to duplicate focus-handling
  logic to participate in view switching.

### Negative / Trade-offs

- `Shell` owns the key context that every switch shortcut is bound to,
  so a view that needs to bind the same keystroke for its own purpose
  (for example, an editor-local `escape`) has to split contexts rather
  than binding directly on itself.
- `main.rs`'s boot path gains a level: it now builds `Shell` and
  focuses it, rather than constructing `DocumentView` directly as the
  root.
- `escape` always returns to the editor. This means the editor view's
  own `escape` handling, if it ever needs one, is consumed by `Shell`
  before it reaches the editor.
- The shortcut table binds `ctrl-` literally, per the keystrokes
  specified for this table. gpui also offers a `secondary-` modifier
  that maps to `cmd` on macOS and `ctrl` elsewhere; `SHORTCUTS` does
  not use it, so these three bindings are `ctrl` on every platform.

### Neutral

- `Shell` renders two views, `Palette` and `Settings`, that hold no
  state of their own today — `shell::placeholder` stands in for both.

## Considered Alternatives

### Alternative A: `App::on_action` global listener

- Summary: register the view-switching actions on `App` directly via
  `App::on_action`, so they fire regardless of what has focus, without
  needing a `Shell` root view at all.
- Reason for rejection: this moves "which view is current" out of any
  view and into a global closure over `App`, which has no natural home
  for that state. It also does not solve rendering — something still
  has to own the child-view swap in the element tree, which puts a
  `Shell`-shaped type back into the design anyway.

### Alternative B: One window per view

- Summary: open a separate OS window for the palette and for settings,
  and switch between windows instead of between views inside one
  window.
- Reason for rejection: window management (positioning, focus transfer
  between OS windows, closing side windows when the main window closes)
  costs more than the problem — showing a palette or a settings pane —
  is worth solving for.

### Alternative C: A data-carrying `ShowView(ShellView)` action

- Summary: one action type carrying the destination view, instead of
  three unit actions (`ShowPalette`, `ShowSettings`, `ShowEditor`).
- Reason for rejection: a data-carrying gpui `Action` needs either
  `#[action(no_json)]`, which drops the registry's ability to resolve
  the action by name, or `serde::Deserialize` + `schemars::JsonSchema`
  derives on `ShellView`, which adds two direct dependencies to
  `limn-ui` for a three-value enum. Three unit actions avoid both costs
  and stay inside what `gpui::actions!` already provides.

---

## Links

- Related ADR: [ADR-0001](0001-adopt-gpui.md) — adopts gpui, whose
  focus-path action dispatch this decision works within.
- Related ADR: [ADR-0002](0002-three-crate-layered-architecture.md) —
  keeps `Shell` inside `limn-ui`, the sole gpui-dependent crate.
- Reference: `crates/gpui/examples/testing.rs` in the pinned gpui
  checkout — the `key_context` + `on_action` + `track_focus` pattern
  `Shell` follows.
