# ADR-0006: Defer the macOS Dock icon until gpui exposes a runtime icon API

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
- There is **no public gpui API for setting the macOS Dock icon**
  (`NSApplication.applicationIconImage`) from a running process.

The Windows runtime title-bar icon initially appeared to be in the
same situation. A Wave 1 acceptance test on a real Windows desktop
revealed that it is not: gpui's Windows backend calls
`LoadImageW(hInstance, MAKEINTRESOURCE(1), IMAGE_ICON, …)` during
window creation, which picks up whichever icon resource the process
binary exposes at resource ID 1. `winresource` already writes Limn's
`.ico` at that exact ID, so the title-bar icon, the Alt+Tab thumbnail,
and the Windows task bar entry all end up showing the Limn mark
through the executable embed — without ever touching
`WindowOptions::icon`. The same path covers X11 once the
`cfg(any(linux, freebsd))` arm in `crates/limn-ui/src/main.rs` runs.

That leaves exactly one surface unreachable from Wave 1: the macOS
Dock icon when `limn-ui` is launched directly out of `target/release/`
without an `.app` bundle. Finder, Spotlight and Dock all read the
icon from `Contents/Resources/*.icns`, which is by definition a
packaging artefact rather than a runtime call.

The workspace lint policy (`Cargo.toml`,
`[workspace.lints.rust] unsafe_code = "forbid"`) blocks raw AppKit
FFI inside `crates/limn-ui/`, so the Dock gap cannot be closed in the
application itself without either weakening the lint or routing the
call through a different crate.

This ADR does not resolve any `ARCHITECTURE.md` Open Question.

---

## Decision

**We defer the macOS Dock icon until either gpui exposes a runtime
icon API or M5 ships the `.app` bundle. Wave 1 ships every other
runtime surface (Windows title bar, Windows task bar, Windows Alt+Tab,
Explorer, X11 window decorations, X11 task list) through the
combination of the executable resource embed and
`WindowOptions::icon`.**

Concretely, Wave 1 wires:

1. `crates/limn-ui/build.rs` calling `winresource` to embed
   `assets/appicons/windows/limn.ico` into `limn-ui.exe`. This single
   resource is consumed by both Explorer (file icon, Alt+Tab,
   task bar) and gpui's Windows backend (runtime title-bar icon via
   `LoadImageW`).
2. `WindowOptions::icon` populated from
   `assets/appicons/linux/limn-256.png` inside a
   `cfg(any(target_os = "linux", target_os = "freebsd"))` guard, so
   X11 sessions show the Limn mark in the window decorations and the
   task list.

The macOS Dock icon remains unimplemented at runtime. It is recovered
when either:

- gpui upstream adds a public `set_application_icon` (or equivalent)
  that the macOS backend bridges to
  `NSApplication.applicationIconImage`; or
- M5 ships the `.app` bundle, at which point Finder, Spotlight and
  Dock all read the icon from `Contents/Resources/Limn.icns` without
  any runtime call.

We make this decision because:

- Every surface except the macOS Dock turned out to be reachable at
  zero further cost — the resource embed and the X11 cfg arm
  together cover the runtime path on the two platforms with a
  workable surface.
- The macOS Dock cannot be reached without either forking gpui or
  violating the workspace `unsafe_code` lint, both of which are
  disproportionate for one cosmetic surface.
- M5 already owns macOS bundling. Folding the deferred Dock icon
  into that milestone matches the packaging boundary ADR-0005 itself
  draws.

---

## Consequences

### Positive

- Wave 1 lands without forking gpui, without weakening the
  workspace `unsafe_code` lint, and without introducing a new FFI
  shim crate.
- The Windows runtime title-bar icon is reached through the same
  resource embed that ADR-0005 wires for Explorer and the task bar
  — one artefact, four surfaces, no extra cost. This was the
  unexpected upside of the Wave 1 audit.
- The X11 runtime icon ships through the supported
  `WindowOptions::icon` path with no caveats.
- The upstream contribution path is narrower than ADR-0005 first
  feared: only macOS Dock registration is missing. When gpui
  exposes a runtime `set_application_icon`, this ADR is superseded
  and the macOS arm of the `cfg(any(linux, freebsd))` guard expands
  to include `target_os = "macos"`.

### Negative / Trade-offs

- macOS users who run `limn-ui` directly out of `target/release/`
  (no `.app` bundle) continue to see the generic terminal-style
  icon in the Dock. The release notes for v0.1.x must call this
  out explicitly.
- The asymmetry between Windows / Linux / FreeBSD (runtime icon
  works through their respective paths) and macOS (Dock deferred)
  must be documented in the `assets/README.md` "Inside the running
  app" matrix so future contributors do not assume every platform
  shares the same path.

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

### Alternative A: Set the macOS Dock icon via direct AppKit FFI

- **Summary**: Add an `unsafe` FFI shim (either inside `limn-ui` by
  weakening the workspace lint, or inside a sibling
  `limn-platform-shim` crate that opts out) that calls
  `NSApplication.applicationIconImage` from the running process.
- **Reason for rejection**: Violates `unsafe_code = "forbid"` in the
  runtime layer that ADR-0005 itself relied on as a constraint, and
  risks racing with gpui's own event loop on the macOS main thread.
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
- **Reason for rejection**: The Windows `.exe` embed (which the
  Wave 1 acceptance test showed reaches both Explorer and the
  runtime title bar) and the X11 runtime icon both work today with
  no packaging-shaped dependencies, so deferring them costs
  user-visible polish for no engineering payoff. ADR-0005 made
  this trade-off explicitly; reopening it would be churn.

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
- `crates/gpui/src/platform/windows/window.rs` (same gpui pin) —
  the `LoadImageW(hInstance, MAKEINTRESOURCE(1), …)` call that
  lifts the Windows runtime title-bar icon from the executable
  resource embed.
- `crates/limn-ui/src/main.rs` — the call site guarded by
  `cfg(any(target_os = "linux", target_os = "freebsd"))` that
  records this decision in code.
