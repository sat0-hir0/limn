# ADR-0010: Settings view as a separate screen and 3-route command convergence

- **Status**: Proposed
- **Date**: 2026-06-29
- **Deciders**: sat0-hir0

---

## Context

ADR-0007 introduced the user-configuration file
(`~/.config/limn/config.toml`, carrying `font` / `theme` / `vault_path`),
loaded at startup and held as a gpui global via [`AppConfig`]. Wave 7
applied only `vault_path`; reading the file is one half of the loop —
**writing** it from inside the running app is Wave 8.

ADR-0008 established the command palette and view-switching foundation:
action types as stable intent, handlers as behaviour, a single
keybinding registry, and `key_context = view name`. Wave 5 wired the
palette's "Open File..." command (ADR-0009); the second placeholder,
"Open Settings", landed as a stub that logged and closed the dialog.

Wave 8 turns "Open Settings" into a real settings view. Three sub-design
questions had to be answered together:

1. **Where does the settings view live in the view tree?** Adding a
   "settings flag" to [`EditorView`] would entangle two unrelated edit
   surfaces (text buffer vs config form) and force the editor's
   `key_context("Editor")` to host Esc semantics that belong to settings.

2. **How do the three routes that open settings (palette command,
   `secondary-,` keybinding, `Esc` to close from settings) converge on
   a single handler?** Three duplicated handler bodies would diverge as
   the feature evolved. ADR-0008's action/intent model points at one
   action per intent, one handler — but the handler now has to live
   *above* both screens, since the editor cannot meaningfully close
   itself and the settings cannot open from a screen it isn't on.

3. **Who owns the palette dialog overlay during a screen switch?**
   ADR-0008 left it on the editor (the only screen at the time), so
   `render_dialog_layer` ran inside [`EditorView::render`]. With two
   screens, the overlay must outlive a screen swap — opening settings
   from the palette must close the dialog *and* swap screens cleanly.

This ADR records the structural answer to all three at once because no
question stands alone: the screen split (1) is what forces the
converging handlers (2) up to a parent view, and the same parent is the
natural new home for the dialog overlay (3).

---

## Decision

We introduce **`AppShell`**, a top-level view that owns the active
screen and the dialog overlay, and we route all three settings-related
intents through actions dispatched into the shell. Concretely:

1. **`AppShell` is the window's first-level view inside `Root`.** The
   render tree becomes `Window → Root → AppShell → (EditorView |
   SettingsView)`. The shell holds an `enum Screen { Editor(...),
   Settings(...) }` plus cached entities for both screens, so toggling
   between them is a screen swap — the editor's autosave debounce
   timer and the settings draft both survive the transition.

2. **One action per intent, handled on `AppShell`.** `OpenSettings`
   switches `screen` to `Settings(...)`; `CloseSettings` switches it
   back to `Editor(...)`. `TogglePalette` moves from `EditorView` to
   `AppShell` (along with the dialog overlay) and is gated on
   `Screen::Editor` so the palette never opens over the settings UI.
   The palette's `OpenSettings` confirm dispatches the **same**
   `OpenSettings` action that the `secondary-,` keybinding does — the
   convergence is by construction.

3. **`Esc` is scoped to the `"Settings"` context.** `escape` is bound
   to `CloseSettings` only inside the `"Settings"` key context that
   [`SettingsView`] declares. This keeps Esc free in the editor (where
   nothing should consume it) and lets the palette's Dialog keep its
   own Esc-to-close — the bindings cannot collide because their
   contexts cannot overlap.

4. **`SettingsView` writes through `limn-service`.** Save serializes
   the draft via `LimnConfig::save_to` on the background executor
   (ADR-0007's atomic rename pattern) and, on success, copies the
   draft into the [`AppConfig`] global so the running session sees it.
   The view never calls `std::fs` directly — ADR-0002 is upheld.

We choose this shape because it makes the three routes correct by
construction (they dispatch the same action, which has one handler),
preserves each screen's identity and edit state across switches, and
keeps the boundary between `limn-ui` and `limn-service` clean for the
new persistence path.

[`AppConfig`]: ../../crates/limn-ui/src/config_global.rs
[`EditorView`]: ../../crates/limn-ui/src/editor.rs
[`SettingsView`]: ../../crates/limn-ui/src/settings.rs
[`EditorView::render`]: ../../crates/limn-ui/src/editor.rs

---

## Consequences

### Positive

- **Upholds ADR-0002.** `limn-ui` still never calls `std::fs` directly;
  Save goes through `LimnConfig::save_to` in `limn-service`.
- **Inherits ADR-0008's dispatch model.** Actions remain intent and
  handlers remain behaviour; the new `OpenSettings` / `CloseSettings`
  types are stable contracts across the palette, the keybinding, and
  the Esc-in-Settings routes.
- **Screen identity survives toggling.** Cached entities mean the
  editor's autosave timer and the settings draft persist across
  back-and-forth navigation — the user can flip into settings and back
  without losing in-flight edits.
- **Single source of truth for "which view is visible".** A future
  Wave 9+ feature (e.g. command history, multi-window) has one place
  to query the active screen, rather than reading focus state across
  views.

### Negative / Trade-offs

- **Dialog overlay owner moved.** The palette overlay used to live on
  `EditorView`; it now lives on `AppShell`. Anything previously
  reasoning about "the editor owns the dialog" no longer holds. The
  palette is also intentionally suppressed on the settings screen —
  Wave 9+ may need to lift this guard if settings-mode commands arrive.
- **Settings UI is intentionally minimal.** Save persists the draft to
  disk and updates the live `AppConfig` global, but applying `font` /
  `theme` to rendering is deferred (the change is observable on next
  launch). Toasts / error surfacing for a failed save are also deferred
  — Wave 8 logs and moves on. These are recorded shortcuts, not
  invisible debt.
- **`secondary-,` is registered with a `None` context** so it works
  from any focused screen. This is consistent with ADR-0008's
  `TogglePalette` binding but means we are committed to never binding
  `secondary-,` to a screen-specific intent in the future.

### Neutral

- The shell holds **both** screen entities for the life of the window
  (built eagerly in `AppShell::new`). The upfront cost is one
  `SettingsView` construction at startup; the alternative (lazy on
  first open) would have to materialize input states against a window
  the shell doesn't own at that point. The eager path is simpler and
  the cost is negligible.
- `EditorView::render` no longer calls `Root::render_dialog_layer` —
  that call moved to `AppShell::render`. Any future view that wants
  to open dialogs must do so through the shell or claim its own
  overlay responsibility explicitly.
- `run_read_only` is **untouched**. The shell, the settings view, and
  the action-routing changes all live behind `LIMN_FEAT_EDIT`. The M1
  read-only path keeps the same minimal entry shape it has had since
  Wave 1.

---

## Addendum (Wave 9, 2026-06-29)

The intentional shortcut recorded in the trade-offs above — *"applying
`font` / `theme` to rendering is deferred (the change is observable on
next launch)"* — is resolved in Wave 9. Persisting the draft and
re-rendering the running session are now the same action:

- **Startup** (`run_editable`, `crates/limn-ui/src/main.rs`) pipes the
  loaded `LimnConfig.theme` into `gpui_component::Theme::change(.., None,
  cx)` right after `cx.set_global(AppConfig(..))`, then `font_family` /
  `font_size` into the `Theme` global, and calls `cx.refresh_windows()`.
  Font fields are written *after* `Theme::change` because `change` runs
  `apply_config`, which resets them — the override has to win.
- **Save** (`SettingsView::save`, `crates/limn-ui/src/settings.rs`)
  applies the same theme/font update inside the write-success `cx.update`
  closure, passing `Some(window)` so `Theme::change` fires
  `window.refresh()` itself. The synchronous test path (`save_to_path`)
  mirrors this with `None` + an explicit `cx.refresh_windows()` to keep
  the two paths semantically equivalent.

`run_read_only` stays untouched — it never initialises gpui-component, so
the theme wiring lives entirely behind `LIMN_FEAT_EDIT`.

Render pixel-verification remains outside gpui headless test coverage;
the observable contract (`Theme::global(cx).mode == expected` after a
save) is asserted in
`crates/limn-ui/tests/e2e_render.rs::theme_global_reflects_config_after_toggle`.

This addendum does **not** change the structural decision above; it only
records that the deferred-rendering shortcut is now paid down. The ADR
stays `Proposed` until a maintainer promotes it.

---

## Considered Alternatives

### Alternative A: Add a "mode" flag to `EditorView`

- Summary: Keep one view; toggle between "edit text" and "edit
  settings" via a boolean (or enum) on `EditorView`.
- Reason for rejection: Forces one `key_context` to host two
  incompatible Esc semantics, mixes the editor's autosave logic with
  settings' draft/save lifecycle, and bloats the view's render path
  with branches. The screen split is cheaper to reason about and lets
  each surface declare its own context cleanly (ADR-0008 constraint 3).

### Alternative B: Open settings in a new window

- Summary: Spawn a separate gpui window for the settings UI.
- Reason for rejection: Multi-window machinery (focus handoff,
  per-window globals, close-window plumbing) is heavier than the
  problem warrants and would force a re-think of the `AppConfig`
  global lifecycle. A single-window screen swap matches the
  keyboard-first design (`AGENTS.md`) — the user never leaves the app
  to change a setting.

### Alternative C: Dispatch open/close from inside `EditorView` /
`SettingsView` instead of a parent shell

- Summary: Let each view handle its own open/close action and mutate
  a shared "active view" cell.
- Reason for rejection: Splits the convergence point across two views,
  re-introducing the duplicated handler bodies the shell was designed
  to avoid. It also leaves no obvious owner for the dialog overlay
  during a screen swap. ADR-0008's "one action, one handler" stance
  is upheld more naturally by lifting the handler to a parent view.

---

## Relationship to ADR-0008

ADR-0008 left two consequences open that this ADR resolves:

- *"Wave 4 lands exactly one action (`TogglePalette`) with a
  placeholder handler; the palette overlay and view-switching are
  deferred."* — view-switching is the topic of this ADR.
- *"the `EditorView` handler sitting on the focus chain... does not
  carry over to the Wave 5 palette's `List`"* — the same focus-chain
  reasoning now applies to `AppShell`. The shell calls
  `track_focus(&self.focus_handle)` on its render root, so it sits on
  the dispatch tree above whichever screen is focused, and the
  globally-bound `OpenSettings` / `TogglePalette` actions reach its
  handlers deterministically (ADR-0008 constraint 4).

The bindings registry from ADR-0008 grows by two action types
(`OpenSettings`, `CloseSettings`) and two keystrokes (`secondary-,`,
`escape@Settings`). No constraint is overturned; this ADR builds on
top.

---

## Links

- Related ADR: [ADR-0008](0008-command-palette-and-view-switching-via-gpui-actions.md)
  (parent — actions / dispatch / context model),
  [ADR-0007](0007-user-configuration-via-toml-file.md) (config file
  this view edits),
  [ADR-0005](0005-adopt-gpui-component-input-and-autosave-raw-text.md)
  (editable shell the settings view layers on top of),
  [ADR-0002](0002-three-crate-layered-architecture.md)
  (limn-ui never calls `std::fs`; settings save goes through
  limn-service)
- Trigger: `crates/limn-ui/src/shell.rs` (the new `AppShell`),
  `crates/limn-ui/src/settings.rs` (the new `SettingsView`),
  `crates/limn-ui/src/actions.rs` (the new `OpenSettings` /
  `CloseSettings` types and their keybindings),
  `crates/limn-ui/src/palette.rs` (palette `OpenSettings` confirm now
  dispatches the action),
  `crates/limn-ui/src/main.rs` (`run_editable` wraps the editor in
  `AppShell` before handing it to `Root`)
- Reference: `AGENTS.md` (keyboard-first), `docs/design/basic-features.md`
