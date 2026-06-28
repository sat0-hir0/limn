# ADR-0007: User configuration via a TOML file at `~/.config/limn/config.toml`

- **Status**: Proposed
- **Date**: 2026-06-29
- **Deciders**: sat0-hir0

---

## Context

Limn so far has no user-controllable configuration: the read-only vs
editable split is driven by hidden `LIMN_FEAT_*` env vars (feature
flags), and the vault root is implicit — the palette's "Open File"
(ADR-0009) lists `.md` files relative to the directory of the file
currently open, which itself comes from the CLI argument or the embedded
Welcome document. There is no way for a user to say "this is my notes
folder" once and have launches land there, nor any place to record
preferences such as editor font or theme.

Wave 7 introduces the **minimum viable configuration mechanism**: a small
on-disk file holding `font`, `theme`, and `vault_path`, loaded once at
startup. The wave deliberately stops at the mechanism plus applying
`vault_path`; rendering `font` / `theme` and a settings UI are Wave 8.

Several constraints from prior decisions bound the design, and they are
all facets of one decision — "how Limn persists and loads user
configuration" — so they are recorded together:

- **Layering (ADR-0002).** All `std::fs` and serialization live in
  `limn-service`; `limn-ui` never touches the filesystem directly.
  Config read/write therefore belongs in `limn-service`. But the loaded
  config must be reachable from the widget tree, and the idiomatic
  `gpui` mechanism for that is `cx.set_global` / `cx.global::<T>()`,
  which requires `impl gpui::Global for T`. `limn-service` is `gpui`-free
  and the orphan rule forbids implementing a `gpui` trait for a
  `limn-service` type in either crate cleanly.
- **GPL surface / dependency hygiene (ADR-0003).** New crates are
  scrutinized. A config feature needs a directory resolver and a
  serialization format; both should ideally add **no new crate** to the
  build graph.
- **Atomic write pattern (ADR-0005).** The editor's autosave already
  writes via temp-file-plus-rename for reader atomicity. Config saves
  should reuse that property rather than invent a second write story.
- **Vault root (ADR-0009).** The palette currently derives the vault
  root from the open file's parent directory as an interim measure. A
  configured `vault_path` should be able to take over that role.

A further hard requirement from the originating issue: **a broken or
missing config must never block startup.** The editor must always open.

---

## Decision

We adopt **a single TOML file at the literal path
`<home>/.config/limn/config.toml` on every OS, (de)serialized with
`serde` + `toml` in `limn-service`, loaded infallibly with default
fallback, and exposed to the UI through a `gpui::Global` newtype wrapper
in `limn-ui`** — because it satisfies every constraint above with zero
new dependencies and a fail-open load.

Concretely:

1. **Fixed literal path, not a platform config dir.** The path is
   `dirs::home_dir()` joined with the literal segments
   `.config/limn/config.toml` on all platforms (including Windows and
   macOS), per the originating issue. We use `dirs::home_dir()` only to
   locate `$HOME`/`%USERPROFILE%`, *not* `dirs`' platform-specific
   `config_dir()`. This keeps the path predictable and identical
   everywhere, which is what the issue asked for; a future ADR may
   migrate to XDG / platform dirs if cross-platform conventions become a
   priority.

2. **Crates already in the graph; no new surface.** `dirs` (6.x),
   `toml` (0.8.x), and `serde` (1.x) are already present in `Cargo.lock`
   via `shellexpand` / `rust-i18n` / `gpui`. Declaring them as direct
   dependencies of `limn-service` adds **no new crate** and requires
   **no `deny.toml` change** — all three are MIT OR Apache-2.0, already
   on the allow list. `toml` is pinned to **0.8** (not 1.x, which also
   exists transitively) to match the 0.8.x already resolved and avoid a
   `multiple-versions` advisory.

3. **Config type in `limn-service`; `gpui::Global` newtype in
   `limn-ui`.** `LimnConfig` (and `FontConfig`, `Theme`, `ConfigError`)
   live in `limn-service`, which stays `gpui`-free (ADR-0002). The UI
   wraps it in a newtype, `AppConfig(LimnConfig)`, and implements
   `gpui::Global` on the wrapper — the standard orphan-rule workaround —
   then registers it with `cx.set_global`, exactly mirroring how
   `FeatureFlags` is registered today.

4. **TOML + `serde(default)` for forward compatibility; no
   `deny_unknown_fields`.** Every config struct carries
   `#[serde(default)]` and deliberately omits `deny_unknown_fields`.
   This gives two-way forward compatibility: a *missing* field (older
   file) falls back to `Default`, and an *unknown* field (newer file)
   is ignored rather than rejected. A bad-but-parseable newer file thus
   never breaks an older build, and vice versa.

5. **Fail-open load; only save returns a `Result`.** `LimnConfig::load`
   returns `Self`, never an error. If the home dir can't be resolved it
   returns defaults; if the file is missing it writes the defaults out
   (best effort) and returns them, so a first run leaves a
   self-documenting file; if the file exists but fails to parse it
   returns defaults **and leaves the broken file untouched** (never
   overwriting what the user may want to hand-fix). Each fallback logs to
   `stderr` so the reason is observable without changing the return type.
   Only `save` / `save_to` return `Result<(), ConfigError>`, and the
   write reuses ADR-0005's temp-file-plus-rename atomicity (the
   per-write PID+counter temp naming is dropped, since config writes are
   rare and single-writer). Internally `load`/`save` are thin wrappers
   over `load_from(&Path)` / `save_to(&Path)` so the logic is testable
   without depending on the process-global home directory.

6. **`vault_path` partially replaces the interim vault root.** When the
   editable path is launched with no CLI argument and `vault_path` is
   `Some`, Limn opens the first `.md` in that vault. A CLI argument still
   wins (existing behaviour preserved), and `None` keeps the previous
   Welcome fallback. This is a *fallback-preserving* partial migration of
   ADR-0009's "open file's parent as vault root" interim measure, not a
   wholesale replacement — so ADR-0009 is **not** superseded, only
   related.

---

## Consequences

### Positive

- A user can set their notes folder once (`vault_path`) and launch into
  it with no argument — the user-facing value of Wave 7.
- Zero new crates and no `deny.toml` change: the dependency and license
  posture is unchanged (ADR-0003).
- Startup is robust: no config state — missing, partial, broken, or
  written by a newer build — can prevent the editor from opening.
- The atomic-save property is consistent with autosave (ADR-0005); a
  concurrent reader never sees a half-written config.
- Forward/back compatibility is structural (`serde(default)` without
  `deny_unknown_fields`), so adding fields later is non-breaking.

### Negative / Trade-offs

- The fixed `~/.config/limn` path ignores platform conventions
  (`%APPDATA%` on Windows, `~/Library/Application Support` on macOS).
  Accepted deliberately per the issue; revisitable in a later ADR.
- Unknown keys are dropped on the next save (not round-tripped), so a
  newer build's extra settings are lost if an older build saves over the
  file. Out of scope for Wave 7.
- `font` / `theme` are loaded and carried but not yet applied to
  rendering, so the file has fields with no visible effect until Wave 8.
- Silent fail-open means a typo in the config is easy to miss (only a
  `stderr` line signals it); there is no in-app surfacing yet.

### Neutral

- `LimnConfig::load` writing defaults on first run creates the file as a
  side effect of reading — intentional (self-documenting first run) but
  worth noting it is not a pure read.
- The `AppConfig` newtype adds one indirection (`AppConfig(LimnConfig)`)
  for global access, matching the established `FeatureFlags` pattern.

---

## Considered Alternatives

### Alternative A: `directories` / `ProjectDirs` (platform config dir)

- Summary: Use the `directories` crate's `ProjectDirs::config_dir()` to
  place the file at the OS-idiomatic location (`%APPDATA%\limn`,
  `~/Library/Application Support/limn`, `$XDG_CONFIG_HOME/limn`).
- Reason for rejection: `directories` is **not** in the current graph, so
  it would add a new crate (against ADR-0003 hygiene), and the
  originating issue explicitly asks for the fixed literal
  `~/.config/limn` path on all OSes. `dirs::home_dir()` covers the only
  resolution we need with no new crate.

### Alternative B: `etcetera`

- Summary: A lighter base-directory crate offering both "home" and
  "XDG"/"native" strategies.
- Reason for rejection: Same blocker as Alternative A — a new crate not
  in the graph — for a path policy (fixed literal) that needs only
  `home_dir()`.

### Alternative C: Fail-closed load (abort startup on bad config)

- Summary: Return `Result` from `load` and exit (or show a fatal error)
  when the config is missing or unparseable, forcing the user to fix it.
- Reason for rejection: Directly violates the issue's requirement that
  the editor always open. A single typo in a preferences file should
  never lock a user out of their notes; defaults + a `stderr` warning is
  the safer posture.

---

## Links

- Related ADR: [ADR-0002](0002-three-crate-layered-architecture.md) —
  layering rule that places config I/O in `limn-service` and the
  `gpui::Global` wrapper in `limn-ui`.
- Related ADR: [ADR-0003](0003-temporarily-accept-gpl-contamination.md) —
  dependency/license hygiene satisfied by reusing in-graph crates.
- Related ADR: [ADR-0005](0005-adopt-gpui-component-input-and-autosave-raw-text.md)
  — the temp-file-plus-rename atomic write pattern reused by config save.
- Related ADR: [ADR-0009](0009-fuzzy-open-file-and-buffer-swap.md) —
  the interim "open file's parent as vault root" that `vault_path`
  partially (fallback-preserving) replaces; not superseded.
