# docs/ — Documentation Map

An index of documents covering the design, development, and operation of Limn.
Read this before diving into the code to understand where everything lives.

---

## Design

| Path | Contents |
|------|----------|
| `/ARCHITECTURE.md` | Architecture overview: crate structure, dependency direction, data flow, cross-cutting concerns, open questions. |
| `docs/adr/` | Architecture Decision Records. Design decisions and their rationale recorded as ADRs. |
| `docs/design/basic-features.md` | Inventory of basic editor features that must be present at a minimum (with priorities). |
| `docs/design/testing-strategy.md` | Testing approach. Hybrid of the testing trophy and Functional Core unit tests. |

To understand the "why" behind a design decision, read `/ARCHITECTURE.md` first, then `docs/adr/`.

---

## Development Process

| Path | Contents |
|------|----------|
| `docs/development/git-strategy.md` | Branch strategy, merge approach, and release flow. |
| `docs/development/feature-flags.md` | Three-stage flag model for merging incomplete features into trunk. |
| `docs/development/flag-inventory.md` | Current inventory of active feature flags. |

---

## Operations / Maintainer

| Path | Contents |
|------|----------|
| `docs/maintainer-runbook/release-public.md` | Step-by-step procedure for squash → tag → make-public immediately before the v0.1.0 release. |

---

## Technical Debt

| Path | Contents |
|------|----------|
| `docs/debt/` | Records of known technical debt and accepted trade-offs. |

---

## ADR List

See [`docs/adr/README.md`](adr/README.md) for details.

| Number | Title | Status |
|--------|-------|--------|
| ADR-0001 | Adopt gpui as the GUI framework | Accepted |
| ADR-0002 | Adopt a three-crate layered architecture | Accepted |
| ADR-0003 | Temporarily accept GPL contamination | Accepted |
| ADR-0004 | Scope `.skillshare/` skills to OSS operating procedures | Accepted |
| ADR-0005 | Register the app icon at runtime and embed it in the Windows `.exe`, defer packaging to M5 | Proposed |
| ADR-0006 | Defer the macOS Dock icon until gpui exposes a runtime icon API | Proposed |
