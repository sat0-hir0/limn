# ADR-0001: Adopt gpui as the GUI Framework

- **Status**: Accepted
- **Date**: 2026-06-21
- **Deciders**: sat0-hir0

---

## Context

Limn is a keyboard-first, AI-integrated native editor.
Building it as a native application — not a web-based one — was established
from the start. The reasons are:

- **Zero-latency writing feel** — Electron / Tauri + Web stacks add rendering
  pipeline layers that accumulate latency between a keystroke and the screen update
- **GPU rendering** — we want to sustain 60 fps even with large block trees and
  long documents
- **Rust native** — confidence in memory safety, performance, and the ecosystem

We evaluated the available options for building a GUI in Rust natively.

---

## Decision

**We adopt gpui.**

gpui is a Rust-native UI library centred on GPU rendering. It has a
production track record powering an editor with world-class text-rendering
performance, which is direct evidence that the framework can meet our
zero-latency writing target.

The "zero-latency writing feel" that Limn targets aligns directly with gpui's
design philosophy. In addition, `gpui-component` (by longbridge) provides 60+
components, so we do not need to reinvent the foundations for an editor,
Markdown rendering, or a virtualised list.

`crates/limn-ui` is the sole crate that depends on gpui; all other crates
import nothing from gpui (→ ADR-0002). This confines the blast radius of
breaking changes in gpui itself to the UI layer.

---

## Consequences

### Positive

- Fast, zero-latency rendering through GPU acceleration
- We inherit a battle-tested rendering foundation rather than building one
- `gpui-component` lets us avoid reinventing foundational components
- Tree-sitter and Rope are already bundled and reusable for Markdown parsing
  and syntax highlighting
- IME support inherits from gpui's established pipeline

### Negative / Trade-offs

- **GPU required** — there is no software-rendering fallback. We cannot
  guarantee operation on older GPUs or in virtual environments
- **GPL contamination** — a transitive dependency (`sum_tree` → `ztracing` →
  `zlog`) is `GPL-3.0-or-later`, which conflicts with Limn's Apache-2.0 licence
  (→ ADR-0003 records the mitigation plan)
- **Unstable pre-1.0 API** — breaking changes are frequent. The UI layer must
  remain thin and gpui dependencies must stay localised
- **Small community** — very few gpui users outside its upstream maintainers.
  Documentation, examples, and community Q&A are limited
- **Not published on crates.io** — we must use the git main version. Version
  pinning is SHA-based, which carries a risk of unintended changes

### Neutral

- Confining gpui to `limn-ui` localises the cost of a future GUI framework
  replacement (serves as insurance)
- Following gpui's upstream development makes it easier to benefit from new features

---

## Considered Alternatives

### Alternative A: `egui`

- **Summary**: An immediate-mode GUI framework written in Rust. Simple API,
  rich documentation, active community, and WebAssembly support.
- **Reason for rejection**: The immediate-mode (non-retained-mode) design is a
  poor fit for the complex state management of a text editor. IME support is
  still incomplete — critical for Japanese input. Few precedents exist for
  building editor-scale UIs with egui.

### Alternative B: `iced`

- **Summary**: Elm-architecture-inspired Rust GUI with Vulkan backend support.
  Research into editor-oriented features (Rope integration, etc.) is advancing.
- **Reason for rejection**: Like gpui, it is pre-1.0 with an unstable API, but
  has fewer editor-proven track records than gpui. There is no equivalent to
  `gpui-component` — ready-made components aimed at editor use.

### Alternative C: `Tauri` (Rust + Web frontend)

- **Summary**: Rust backend combined with an HTML/CSS/JS frontend. Leverages the
  rich web technology ecosystem, including mature editor libraries such as
  CodeMirror and ProseMirror.
- **Reason for rejection**: WebView-based rendering has higher latency than
  gpui's direct GPU rendering, which contradicts the core value of
  "zero-latency writing feel." It also requires developing in two languages
  (Rust and JS), increasing context switching.

### Alternative D: `Tauri + SolidJS`

- **Summary**: Combining Tauri's Rust backend with SolidJS's fine-grained
  reactivity to minimise re-rendering cost on the frontend.
- **Reason for rejection**: Even with SolidJS optimisations, rendering still
  passes through the WebView pipeline and cannot match native GPU rendering.
  The two-language problem remains.

### Alternative E: Web app (Electron)

- **Summary**: Proven by widely-used Markdown-focused note apps. The richest
  web technology ecosystem.
- **Reason for rejection**: Electron has high memory consumption and slow
  startup. This is the opposite of Limn's core value of a "lightweight,
  zero-latency native editor."

---

## Links

- [gpui upstream](https://github.com/zed-industries/zed/tree/main/crates/gpui)
- [gpui-component (longbridge)](https://github.com/longbridge/gpui-component)
- ADR-0002: Adopt a Three-Crate Layered Architecture
- ADR-0003: Temporarily Accept GPL Contamination
