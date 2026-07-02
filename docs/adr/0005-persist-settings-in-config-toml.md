# ADR-0005: Persist settings in `config.toml`

- **Status**: Proposed
- **Date**: 2026-07-02
- **Deciders**: sat0-hir0

---

## Context

Limn has no configuration infrastructure. There is no place to record
user preferences, no format for them, and no code path that reads or
writes them. Issue #76 introduces a Settings view that lets the user
edit the colour theme, the editor font (family and size), and the vault
folder path — none of which can currently be stored between runs.

Two prior constraints shape where this belongs:

- **ADR-0002 (three-crate layering).** `limn-core` is the functional
  core: `std` only, pure functions, no I/O crates. Deciding *where* the
  config file lives on disk and reading/writing it are side effects.
  That work belongs in `limn-service`, the imperative shell. (Serde
  derives on a plain data struct would be tolerable in the core, but the
  file-location logic and file I/O are not — keeping the whole config
  concern in one crate avoids splitting a single concept across the
  layer boundary.)
- **Issue #76 framing.** The issue calls for Settings as a view the user
  switches to, not a simultaneous overlay on the editor.

A format and a storage location must be chosen, and the interaction
model (view vs. overlay) settled, before the Settings feature can read
or write anything.

---

## Decision

We persist user settings in `~/.config/limn/config.toml` — the platform
config directory resolved via the `dirs` crate, so on Windows this is
`%APPDATA%\limn\config.toml` — as TOML through `serde`, loaded at
startup and written on Save, and we open Settings as a separate
switchable view rather than a modal or overlay.

We choose TOML because it is human-editable and matches the surrounding
Cargo/Rust ecosystem, so a user can inspect or hand-fix the file without
tooling. We place the config concern in `limn-service` because the
config-directory resolution and file I/O are side effects that ADR-0002
confines to the imperative shell. We make Settings a switchable view
because the issue frames it as a view the user moves to and away from,
not a layer drawn on top of the editor.

---

## Consequences

### Positive

- Settings survive across runs, stored in a single well-known file.
- The file is human-readable and hand-editable, matching the Rust
  ecosystem's default configuration format.
- The config concern sits entirely in `limn-service`, so `limn-core`
  stays `std`-only and the layer boundary from ADR-0002 holds.
- A settings-as-view model keeps the editor and Settings as distinct
  full surfaces, which is simpler to reason about than an overlay that
  shares screen space and focus with the editor.

### Negative / Trade-offs

- TOML is less expressive than richer formats for deeply nested or
  dynamic data; the settings schema stays flat to fit it comfortably.
- Config I/O adds `serde`, `toml`, and `dirs` as dependencies of
  `limn-service`.
- A switchable view means the user cannot see the editor and Settings at
  the same time, unlike a side panel.

### Neutral

- The config file location follows platform conventions via `dirs`, so
  the exact path differs per operating system.
- The settings struct uses per-field serde defaults, so a config file
  missing fields still loads.

---

## Considered Alternatives

### Alternative A: `JSON`

- Summary: Persist settings as JSON instead of TOML.
- Reason for rejection: TOML is more human-editable for a flat settings
  file and matches the Cargo/Rust ecosystem the project already lives
  in; JSON's lack of comments and noisier punctuation make hand-editing
  worse for no offsetting benefit at this scale.

### Alternative B: `config in limn-core`

- Summary: Put the config struct and its load/save logic in `limn-core`.
- Reason for rejection: Resolving the on-disk config location and
  reading/writing the file are I/O side effects, which ADR-0002 confines
  to `limn-service`. Placing them in the functional core would breach the
  layering boundary.

### Alternative C: `modal overlay instead of separate view`

- Summary: Present Settings as a modal or overlay drawn on top of the
  editor.
- Reason for rejection: Issue #76 specifies view switching (not showing
  editor and Settings simultaneously), so an overlay contradicts the
  stated interaction model.

---

## Links

- Related ADR: [ADR-0002](0002-three-crate-layered-architecture.md)
- Issue: #76 (Settings view)
