# ADR-0006: Defer runtime title-bar and Dock icon on macOS and Windows until gpui exposes an icon surface

- **Status**: Proposed
- **Date**: 2026-06-25
- **Deciders**: sat0-hir0

---

## Context

[ADR-0005](0005-app-icon-runtime-and-embed.md) decided to register
the application icon at runtime on every platform, and embed it into
`limn-ui.exe` as a Windows executable resource. The runtime portion
was conditional on gpui exposing a usable icon surface on the pinned
revision; the ADR explicitly delegated that finding to a follow-up
record "to be opened only in that case."

After ADR-0005 was proposed, an audit of the pinned gpui revision
(`1d217ee39d381ac101b7cf49d3d22451ac1093fe`) confirmed that:

- `gpui::WindowOptions` exposes one icon field,
  `pub icon: Option<Arc<image::RgbaImage>>`, documented as
  *"Icon image (X11 only)"*
  (`crates/gpui/src/platform.rs`).
- The matching `WindowParams::icon` is annotated as *"An image to set
  as the window icon (x11 only)"*.
- The macOS, Windows, and Wayland window backends do not read
  `WindowOptions::icon`. On those targets the field is silently
  ignored.
- There is no public gpui API for setting the macOS Dock icon
  (`NSApplication.applicationIconImage`) or the Windows runtime
  title-bar icon (`WM_SETICON`).

The workspace lint policy (`Cargo.toml`,
`[workspace.lints.rust] unsafe_code = "forbid"`) blocks raw Win32 or
AppKit FFI inside `crates/limn-ui/`, so the runtime gap cannot be
closed in the application itself without either weakening the lint
or routing the call through a different crate.

The Windows executable embed considered by ADR-0005 is independent
of gpui and is not affected by this finding: a `build.rs` using
`winresource` produces a valid `.ico` resource in `limn-ui.exe`
regardless of what gpui exposes at runtime.

This ADR does not resolve any `ARCHITECTURE.md` Open Question.

---

## Decision

**We defer the macOS Dock icon and the Windows runtime title-bar
icon until gpui exposes a cross-platform icon surface, and ship Wave
1 with only the two paths that work today: the Windows `.exe`
resource embed, and the X11 runtime icon on Linux / FreeBSD.**

Concretely, Wave 1 wires:

1. `crates/limn-ui/build.rs` calling `winresource` to embed
   `assets/appicons/windows/limn.ico` into `limn-ui.exe`.
2. `WindowOptions::icon` populated from
   `assets/appicons/linux/limn-256.png` inside a
   `cfg(any(target_os = "linux", target_os = "freebsd"))` guard, so
   X11 sessions show the Limn mark in the window decorations and the
   task list.

macOS Dock icon registration and the Windows runtime title-bar icon
remain unimplemented. They are recovered when either:

- gpui upstream adds a cross-platform icon API that the pinned
  revision (or a future one) can call; or
- M5 ships an `.app` bundle on macOS (Finder and the Dock both read
  `Contents/Resources/*.icns` from the bundle, independent of any
  gpui call) and a Windows installer or shortcut that sets the icon
  via shell metadata.

We make this decision because:

- The two paths that work today already deliver the majority of the
  ADR-0005 user-visible win at zero additional cost: Explorer, the
  Windows task bar, Alt+Tab, and X11 window managers all show the
  Limn mark once Wave 1 lands.
- The two paths that do not work today (macOS Dock at runtime,
  Windows runtime title-bar) cannot be implemented without either
  forking gpui or violating the workspace `unsafe_code` lint, both
  of which are disproportionate for one cosmetic surface.
- M5 already owns macOS bundling and Windows installer scripting.
  Folding the deferred icons into that milestone matches the
  packaging boundary ADR-0005 itself draws.

---

## Consequences

### Positive

- Wave 1 lands without forking gpui, without weakening the
  workspace `unsafe_code` lint, and without introducing a new FFI
  shim crate.
- The two surfaces that ship — Windows `.exe` resource and X11
  runtime icon — cover the largest share of "is this a real app?"
  first impressions for the platforms most contributors use today.
- The upstream contribution path is well-defined: gpui needs a
  platform-agnostic `set_window_icon` (or extended
  `WindowOptions::icon` semantics) on the macOS and Windows
  backends. When that lands, this ADR is superseded and the
  `cfg(any(linux, freebsd))` guard in `crates/limn-ui/src/main.rs`
  becomes unconditional.

### Negative / Trade-offs

- macOS users who run `limn-ui` directly out of `target/release/`
  (no `.app` bundle) continue to see the generic terminal-style
  icon in the Dock. The release notes for v0.1.x must call this
  out explicitly.
- Windows users see the Limn mark in Explorer and the task bar
  (from the resource embed) but the in-window title-bar icon shown
  by some shells (e.g. when the window is minimised and previewed
  in Alt+Tab on certain Windows themes) may still fall back to the
  resource embed instead of a richer runtime path. This is a minor
  cosmetic delta only visible in specific themes.
- The asymmetry between Linux / FreeBSD (runtime icon works) and
  macOS / Windows (runtime icon deferred) must be documented in the
  `assets/README.md` "How to use" matrix so future contributors do
  not assume the runtime path is uniform.

### Neutral

- The decision does not touch `limn-core` or `limn-service`, so
  ADR-0002's layering is preserved.
- The decision does not change the licence story:
  `winresource` (BSD-2-Clause) is still build-time only and the
  runtime artifact picks up no new dependency on macOS / Windows.
- ADR-0001's choice of gpui is unchanged; this ADR records a
  surface-level gap in the pinned revision rather than a flaw in
  the framework choice.

---

## Considered Alternatives

### Alternative A: Set the macOS Dock and Windows title-bar icon via direct FFI

- **Summary**: Add an `unsafe` FFI shim (either inside `limn-ui` by
  weakening the workspace lint, or inside a sibling
  `limn-platform-shim` crate that opts out) that calls
  `NSApplication.applicationIconImage` on macOS and `WM_SETICON` on
  Windows from the running process.
- **Reason for rejection**: Violates `unsafe_code = "forbid"` in the
  runtime layer that ADR-0005 itself relied on as a constraint, and
  risks racing with gpui's own event loop on both platforms.
  Adding a new shim crate purely to opt out of the workspace lint
  is structurally identical to weakening the lint and trades a
  one-pixel cosmetic win for an architectural exception that other
  ADRs explicitly forbid.

### Alternative B: Upgrade the gpui pin to a revision that exposes a cross-platform icon API

- **Summary**: Bump `gpui` and `gpui_platform` in the workspace to a
  newer revision that may have closed the gap upstream.
- **Reason for rejection**: At the time of writing no such revision
  exists in `zed-industries/zed` — the icon field has been X11-only
  for the lifetime of the public gpui crate. Bumping speculatively
  also conflates a runtime-icon decision with a gpui-update
  decision, each of which carries its own ADR-shaped impact
  (gpui-component compatibility, breaking changes elsewhere). The
  right place to record a gpui pin bump is a separate ADR scoped to
  that bump.

### Alternative C: Defer the entire Wave 1 until M5

- **Summary**: Withdraw ADR-0005 in its entirety and wait for the
  M5 packaging milestone to wire up all icon paths together.
- **Reason for rejection**: The Windows `.exe` embed and the X11
  runtime icon both work today with no packaging-shaped
  dependencies, so deferring them costs user-visible polish for no
  engineering payoff. ADR-0005 made this trade-off explicitly;
  reopening it would be churn.

---

## Links

- [ADR-0005](0005-app-icon-runtime-and-embed.md) — the parent
  decision this ADR completes; ADR-0005 §Decision delegates the
  runtime-API finding to "ADR-0006, to be opened only in that
  case."
- [ADR-0001](0001-adopt-gpui.md) — defines the runtime surface that
  this ADR observes as gap-bearing.
- [ADR-0002](0002-three-crate-layered-architecture.md) — constrains
  the wiring to `limn-ui` and forbids a shim crate that would
  re-layer the project.
- `crates/gpui/src/platform.rs` (gpui pin
  `1d217ee39d381ac101b7cf49d3d22451ac1093fe`) — the
  `WindowOptions::icon` and `WindowParams::icon` declarations whose
  X11-only annotation triggered this ADR.
- `crates/limn-ui/src/main.rs` — the call site guarded by
  `cfg(any(target_os = "linux", target_os = "freebsd"))` that
  records this decision in code.
