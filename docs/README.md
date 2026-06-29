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
| `docs/design/visual-language.md` | Visual language ( paper & ink, line blue, hairlines ). Maps to `crates/limn-ui/src/theme.rs`. |

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
| ADR-0005 | Adopt gpui-component `InputState` for editing and autosave raw text | Proposed |
| ADR-0006 | Reserved (number skipped, no decision recorded) | Rejected |
| ADR-0007 | User configuration via a TOML file at `~/.config/limn/config.toml` | Proposed |
| ADR-0008 | Command palette and view-switching via gpui actions | Proposed |
| ADR-0009 | Fuzzy open-file in the palette via vault listing and InputState buffer swap | Proposed |
| ADR-0010 | Settings view as a separate screen and 3-route command convergence | Proposed |
| ADR-0011 | Adopt Limn visual language ( paper & ink ) as the source of truth for color | Proposed |
