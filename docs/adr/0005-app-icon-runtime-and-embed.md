# ADR-0005: Register the app icon at runtime and embed it in the Windows `.exe`, defer packaging to M5

- **Status**: Proposed
- **Date**: 2026-06-25
- **Deciders**: sat0-hir0

---

## Context

The repository already ships finished icon artwork for all three target
platforms under `assets/appicons/{macos,windows,linux}/`, together with
master SVGs and a documented regeneration workflow
([`assets/README.md`](../../assets/README.md)).

However, none of those assets are wired into the build or the running
application:

- `crates/limn-ui/Cargo.toml` has no `build.rs` and no resource-embedding
  build dependency.
- [`crates/limn-ui/src/main.rs`](../../crates/limn-ui/src/main.rs) constructs
  `WindowOptions { window_bounds, ..Default::default() }` and never sets an
  icon field.
- A repository-wide grep for `set_icon` / `app_icon` / `HICON` returns
  only the descriptive text inside `assets/README.md`.

As a result, on every supported OS the binary today shows the host
toolkit's generic placeholder in Explorer, the Dock, the task bar,
Alt+Tab, and the GNOME / KDE application list. For a v0.1.x release this
is a visible regression against the project's "quiet UI" axis
(ARCHITECTURE.md §Design Axes).

`assets/README.md` already states that wiring "is planned for M5" — the
packaging milestone. The decision recorded here is whether to honour
that schedule strictly or to bring forward the subset of work that does
not depend on a packaging pipeline.

The decision is constrained by:

- **ADR-0001** — gpui is the GUI framework; any runtime icon wiring must
  go through whatever icon surface gpui exposes on the pinned revision.
- **ADR-0002** — the three-crate layering forbids `limn-core` and
  `limn-service` from depending on UI / packaging concerns. Icon wiring
  must remain inside `limn-ui`.
- **ADR-0003** — the GPL allow-list is unaffected; the only new build
  dependency considered (`winresource`, BSD-2-Clause) is permissively
  licensed and does not reach the runtime artifact.
- **Workspace lint** — `unsafe_code = "forbid"` (workspace `Cargo.toml`)
  blocks raw Win32 / AppKit FFI in runtime code. `build.rs` scripts are
  outside the workspace lint scope, so a build-time resource embed is
  permitted.

This ADR does not resolve any `ARCHITECTURE.md` Open Question; the six
listed questions cover AI integration, `/` scope, completion breathing,
AI model selection, graph layout, and IME quality, none of which
overlap with icon registration.

---

## Decision

**We register the application icon at runtime now, and embed it as a
Windows executable resource at build time now. We defer all packaging
artefacts (installers, `.app` bundles, `.deb` / `.rpm`, code signing,
notarisation) to the M5 packaging milestone, unchanged.**

Concretely, the runtime / embed work scoped here covers:

1. A `crates/limn-ui/build.rs` script that, on Windows targets, compiles
   `assets/appicons/windows/` into a `.ico` resource and links it into
   `limn-ui.exe` via `winresource` (BSD-2-Clause, build-dependency
   only).
2. A `WindowOptions::window_icon` (or the equivalent surface exposed by
   the pinned gpui revision) populated from the bundled PNG bytes via
   `include_bytes!`, applied uniformly on Windows / macOS / Linux.

If the pinned gpui revision does not expose a runtime icon surface, the
runtime portion is deferred and recorded under a follow-up
ADR (ADR-0006, to be opened only in that case) describing the upstream
contribution path. The Windows resource embed proceeds regardless,
because it is independent of gpui.

We make this decision because:

- The artwork already exists in-tree, so the incremental cost is small
  (a handful of files in one crate).
- The visible quality gap between a running Limn window and an
  icon-bearing first-class native application is exactly the kind of
  surface "quiet UI" expects to be invisibly correct, not blank.
- The remaining packaging work (installer scripting, signing keys,
  notarisation flow) is what genuinely belongs to M5 — it requires
  product decisions (distribution channel, certificate ownership) that
  are not yet made.

The wiring stays inside `limn-ui` to preserve ADR-0002's layer
boundary. `limn-core` and `limn-service` continue to know nothing about
icons or packaging.

---

## Consequences

### Positive

- The running window, the `.exe` file in Explorer, and the Windows task
  bar all show the Limn mark instead of a placeholder — closing a
  visible "is this a real app?" gap at v0.1.x cost.
- On macOS and Linux, the running window shows the Limn mark in Dock /
  Alt-Tab / Activities once gpui's icon surface is wired, without
  waiting for `.app` or `.desktop` packaging.
- The `assets/appicons/` material starts paying for itself immediately
  rather than waiting one milestone.
- The packaging decisions deferred to M5 (signing, notarisation,
  installer choice) remain free to be made in their own context, with
  the artwork pipeline already proven.

### Negative / Trade-offs

- One new build dependency (`winresource`) enters the graph. It is
  BSD-2-Clause, build-time only, and does not appear in the shipped
  binary, but the workspace `Cargo.lock` grows.
- macOS users still see a generic icon in Finder until M5 ships the
  `.app` bundle, because Finder reads `.icns` from a bundle, not from
  the binary. The runtime Dock icon will be correct; the Finder file
  icon will not. This asymmetry must be documented in release notes.
- Linux distributions that install the binary without a `.desktop` file
  will likewise see no application-launcher entry. The runtime window
  icon will be correct; the launcher entry will not.
- If gpui's pinned revision lacks an icon surface, the runtime portion
  is silently deferred and only the Windows embed lands — a partial
  delivery that the changelog must call out clearly.

### Neutral

- The decision does not touch `limn-core` or `limn-service`, so the
  layered architecture is preserved.
- No change to the licence story: artwork is Apache-2.0 (see
  `assets/README.md`), `winresource` is BSD-2-Clause, both permissive.
- The M5 packaging milestone shrinks slightly in scope (the artwork
  pipeline is already validated) but does not change in nature.

---

## Considered Alternatives

### Alternative A: Do nothing until M5

- **Summary**: Honour `assets/README.md` verbatim and wire icons up
  only when the packaging milestone arrives.
- **Reason for rejection**: Leaves a visible quality gap during the
  entire v0.1.x window for no engineering payoff. The runtime / embed
  work does not depend on any M5-shaped decision (signing,
  distribution channel), so deferring it just defers user-visible
  polish that is already paid for in artwork.

### Alternative B: Bring forward the entire packaging pipeline

- **Summary**: Implement runtime icons, `.exe` embed, `.app` bundle,
  `.icns`, `.desktop` + hicolor placement, Windows installer (NSIS or
  MSIX), code signing and notarisation in one branch.
- **Reason for rejection**: Bundles three different categories of
  decision — UI wiring, OS shell integration, and distribution policy
  — into a single change. The distribution policy questions
  (certificate ownership, notarisation account, installer format) are
  not yet decided and would force them prematurely. It also violates
  the project's branch-lifetime guideline of one-to-three days
  ([`docs/development/git-strategy.md`](../development/git-strategy.md)).

### Alternative C: Embed in the Windows `.exe` only, skip runtime API

- **Summary**: Ship the Windows resource embed, leave macOS and Linux
  with the placeholder runtime icon until M5.
- **Reason for rejection**: The runtime icon path through gpui costs
  roughly the same as the Windows embed once the artwork is in place,
  and it benefits all three platforms. Skipping it leaves the macOS
  Dock and Linux Activities visibly blank for no proportional saving.
  This option is only adopted as a fallback if gpui's pinned revision
  does not expose an icon surface.

---

## Links

- [`assets/README.md`](../../assets/README.md) — artwork inventory and
  packaging plan note (the "planned for M5" line this ADR partially
  rolls back).
- [`crates/limn-ui/src/main.rs`](../../crates/limn-ui/src/main.rs) — the
  `WindowOptions` site that the runtime portion will touch.
- ADR-0001: Adopt gpui as the GUI Framework — defines the runtime
  surface this ADR relies on.
- ADR-0002: Adopt a Three-Crate Layered Architecture — constrains the
  wiring to `limn-ui`.
- [`docs/development/git-strategy.md`](../development/git-strategy.md) —
  the branch-lifetime rule that ruled out Alternative B.
