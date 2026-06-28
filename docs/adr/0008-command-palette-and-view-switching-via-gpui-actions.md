# ADR-0008: Command palette and view-switching via gpui actions

- **Status**: Proposed
- **Date**: 2026-06-29
- **Deciders**: sat0-hir0

---

## Context

ADR-0005 made the editor writable by adopting `gpui-component`'s
`InputState` behind the `LIMN_FEAT_EDIT` flag. The editable shell now
needs keyboard-driven commands — a command palette to invoke actions,
and (later) switching between views such as the editor and a settings
pane. Limn is positioned as a keyboard-first editor (`AGENTS.md`), so
the command/keybinding layer is foundational rather than incidental.

gpui already ships the mechanism for this: typed *actions* dispatched
through the focused view's *dispatch tree*, with *keybindings* mapping
keystrokes to action types, optionally scoped by a *context* string.
This raises three design questions that need to be settled before
handlers and overlays are built on top, because getting them wrong
forces churn across every later command:

1. **Where do keybindings and action types live?** Scattering
   `actions!` declarations and `bind_keys` calls across view modules
   makes the keymap impossible to audit and invites duplicate or
   conflicting bindings.

2. **What does an action carry — intent, or behaviour?** If actions
   encode *how* a command is executed, the action vocabulary churns
   whenever the implementation changes (e.g. when Wave 5 gives the
   palette a real overlay).

3. **Which context names do we use, and does the gpui-component `Root`
   context interfere?** gpui-component wraps the window's first view in
   a `Root` that declares its own `"Root"` key context. limn's actions
   must dispatch to limn views, not be entangled with the component's
   context.

A concrete constraint also surfaced during implementation: gpui
dispatches an action to the handlers found on the **focused view's
dispatch (focus) tree**. `EditorView` focuses the `InputState` so text
input reaches the component's `EntityInputHandler` (cursor, selection,
IME). Unless `EditorView` is itself on that focus chain, a
globally-bound action has no limn handler to reach. No real device was
available this session to confirm dispatch behaviour empirically, so
the decision favours the arrangement that is correct by construction.

This ADR records the foundation only (Wave 4). The command palette
overlay and additional views are later work; they are named here to
justify the shape of the foundation, not implemented by it.

---

## Decision

We adopt gpui's action/keybinding system as Limn's command layer, with
four constraints:

1. **One keybinding registry.** All action *types* are declared in
   `crates/limn-ui/src/actions.rs` (namespace `limn`), and all global
   keybindings are registered in a single `bind_keys(cx)` called once
   from the app run closure. limn-ui owns the vocabulary of intents.

2. **Actions are intent, handlers are behaviour.** An action type (e.g.
   `TogglePalette`) names *what the user wants*, not *how it happens*.
   The behaviour lives in a view's `on_action` handler. This lets Wave 5
   replace a handler body (placeholder log → real palette overlay)
   without changing the action type, so the action vocabulary stays
   stable as implementations evolve.

3. **Context name = view name.** The key context a view declares is named
   after the view (`"Editor"`, later `"Settings"`), declared on the
   view's render root via `key_context`. We do **not** use
   gpui-component's `Root` `"Root"` context for limn actions; limn
   contexts are owned by limn views.

4. **Handling views sit on the focus chain.** A view that handles
   actions tracks its own focus handle on its render root
   (`track_focus`) and registers handlers with `on_action`. The view
   keeps the actual keyboard focus on its inner input (so text/IME still
   works) while remaining an *ancestor* of the focused element on the
   dispatch tree — which is what makes action dispatch reach the handler
   deterministically.

5. **Cross-platform modifiers use gpui's `secondary`.** Keybindings that
   should mean "Cmd on macOS, Ctrl elsewhere" use gpui's `secondary`
   keystroke modifier, which resolves to Cmd on macOS and Ctrl on
   Windows/Linux from a single binding. A plain `cmd-` binding would
   collapse to the platform modifier (Win+… on Windows, which the shell
   reserves and never delivers to the app); gpui has no OS-level cmd→ctrl
   normalization for `cmd` itself, so `secondary` is the correct primitive.

We choose this because it keeps the keymap auditable in one place,
decouples the stable intent vocabulary from churning implementations,
and makes action dispatch correct by construction rather than dependent
on unverified focus behaviour.

---

## Consequences

### Positive

- The full keymap is auditable in one file; conflicts and duplicates are
  visible at a glance.
- Action types are stable contracts: Wave 5 (palette overlay) and later
  view-switching swap handler bodies, not the action vocabulary or
  keybindings.
- Dispatch is correct by construction — the handling view is always on
  the focus chain — so the feature does not hinge on focus behaviour
  that could not be verified on a real device this session.
- Context naming after views keeps limn's contexts independent of the
  gpui-component `Root` context.

### Negative / Trade-offs

- A central registry is a coordination point: every new command touches
  `actions.rs`, and views must agree on context names. This is the cost
  of auditability.
- Keeping focus on the inner input while the view sits above it on the
  dispatch tree is a deliberate split that future contributors must
  understand (documented in `editor.rs`); a naive "focus the view"
  change would break text input.
- Couples the command layer to gpui's action/dispatch model and to the
  pinned gpui rev, consistent with the existing pin constraint
  (ADR-0001, ADR-0005).

### Neutral

- Wave 4 lands exactly one action (`TogglePalette`) with a placeholder
  handler; the palette overlay and view-switching are deferred.
- No public CLI/argv change; the feature stays behind `LIMN_FEAT_EDIT`
  (ADR-0005).
- Wave 5's palette uses gpui-component's Dialog overlay machinery
  (`Root::render_dialog_layer`, the `"Dialog"` key context). This is
  independent of constraint 3 (not binding limn actions to the `"Root"`
  context): the Dialog's Esc/Enter are handled by the component's
  `"Dialog"` and `"List"` contexts, not by any limn-owned context, so the
  command layer stays decoupled from the component's `Root` context.

---

## Considered Alternatives

### Alternative A: Per-view keybinding registration

- Summary: Let each view declare its own `actions!` and call its own
  `bind_keys`, with no central registry.
- Reason for rejection: The keymap becomes impossible to audit as views
  multiply; duplicate or conflicting bindings are likely and hard to
  detect. The coordination cost of a central registry buys a property
  (one auditable keymap) we want from the start.

### Alternative B: Actions carry behaviour (fat actions)

- Summary: Encode execution details in the action itself (e.g. an action
  that directly opens a specific overlay), rather than treating the
  action as intent dispatched to a view handler.
- Reason for rejection: The action vocabulary would churn every time an
  implementation changes — exactly when Wave 5 turns the placeholder
  into a real overlay. Separating intent (action type) from behaviour
  (handler) keeps the contract stable across that change.

### Alternative C: Reuse the gpui-component `Root` context

- Summary: Hang limn's key contexts/actions off the `"Root"` context
  that gpui-component's `Root` already declares.
- Reason for rejection: Entangles limn's command layer with a
  third-party component's context, making limn's dispatch dependent on
  the component's internal context naming. Owning limn contexts on limn
  views keeps the boundary clean.

---

## Links

- Related ADR: [ADR-0005](0005-adopt-gpui-component-input-and-autosave-raw-text.md)
  (parent — editable shell this command layer builds on),
  [ADR-0001](0001-adopt-gpui.md) (gpui as GUI framework, source of the
  action/dispatch model),
  [ADR-0002](0002-three-crate-layered-architecture.md) (limn-ui owns UI
  bindings; the registry lives there)
- Trigger: `crates/limn-ui/src/actions.rs` (registry),
  `crates/limn-ui/src/editor.rs` (`Editor` context, focus-chain handler)
- Reference: `AGENTS.md` (keyboard-first, command palette as a stated
  limn-ui concern)
