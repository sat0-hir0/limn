# ADR-0002: Adopt a Three-Crate Layered Architecture

- **Status**: Accepted
- **Date**: 2026-06-21
- **Deciders**: sat0-hir0

---

## Context

Limn has the following characteristics:

- **Heavy business logic** — substantial pure logic including block tree
  manipulation, Markdown round-trip, completion engine, and link analysis
- **Scattered I/O** — numerous side-effectful operations such as file
  read/write, AI API calls, and link index construction
- **Pre-1.0 GUI framework** — gpui changes its API frequently with breaking
  changes; we want to minimise the cost of UI churn
- **Testability matters** — we want to validate Markdown round-trip correctness
  and completion logic accuracy quickly through unit tests

In a single-crate setup, business logic, I/O, and UI would be intermingled,
requiring gpui and tokio initialisation just to run tests. Breaking changes in
gpui would also ripple across the entire codebase.

---

## Decision

**We adopt a three-crate layered architecture.**

```
limn-ui ─→ limn-service ─→ limn-core
limn-ui ────────────────→ limn-core
```

Dependencies flow in one direction only. Reverse dependencies are forbidden.

### limn-core (Functional Core)

- `std` only. No `tokio`, `gpui`, or I/O crates.
- Owns the block tree, Markdown conversion, completion engine, and link parser.
- All modules are pure functions — no side effects — so unit tests run extremely fast.

### limn-service (Imperative Shell)

- Depends on `limn-core` plus async / I/O crates only. No `gpui`.
- Owns file I/O, AI API integration, the link index, and graph data.
- Side effects are confined here.

### limn-ui (View layer)

- The only crate that may depend on all other crates.
- Contains only gpui bindings and UI logic. Business logic does not belong here.

This structure enforces Gary Bernhardt's "Functional Core, Imperative Shell"
pattern as a physical boundary via the Cargo workspace.

---

## Consequences

### Positive

- **Dependency direction is enforced mechanically by Cargo** — accidentally
  importing `gpui` in `limn-core` produces a compile error; no human review
  required
- **Tests for limn-core run extremely fast** — no gpui or tokio initialisation
  needed; `cargo test -p limn-core` completes in tens of milliseconds
- **Impact of gpui breaking changes is confined to limn-ui** — tests for
  limn-core and limn-service are not broken by gpui changes
- **limn-core is reusable in other projects** — it can be extracted in the
  future as a CLI tool or Language Server

### Negative / Trade-offs

- **Cargo workspace management overhead** — there are four `Cargo.toml` files
  (three crates plus the workspace root); dependency version alignment requires
  care
- **Cross-crate changes cost more** — when a business logic change affects both
  limn-core and limn-service, the PR touches more files
- **Risk of over-engineering** — in an early, small codebase three crates can
  feel redundant

### Neutral

- If `limn-service` remains thin, it could be merged into `limn-core` in the
  future (though this would sacrifice the dependency-direction benefits)

---

## Considered Alternatives

### Alternative A: Single crate

- **Summary**: Put everything in one crate — the simplest possible structure.
- **Reason for rejection**: Testing business logic requires initialising gpui.
  gpui breaking changes propagate across the entire codebase. As the codebase
  grows, dependencies become entangled and refactoring becomes difficult.

### Alternative B: Hexagonal Architecture (Port & Adapter)

- **Summary**: The domain layer defines Ports (interfaces), and the
  Infrastructure layer implements them as Adapters. The domain layer does not
  depend on concrete I/O implementations.
- **Reason for rejection**: In Rust, Ports are expressed as traits, but
  handling lifetimes and type parameters tends to become complex. This level
  of abstraction is excessive for Limn's scale. The three-crate structure
  achieves the equivalent effect via physical boundaries.

### Alternative C: DDD-style four layers (Presentation / Application / Domain / Infrastructure)

- **Summary**: The standard four-layer structure from Domain-Driven Design,
  suited for large-scale enterprise applications.
- **Reason for rejection**: Over-engineered for Limn's scale. Maintaining the
  four-layer boundaries adds boilerplate. Cargo crates are heavier units than
  modules, and four crates carry significant management overhead.

---

## Links

- [Gary Bernhardt: Functional Core, Imperative Shell](https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell)
- `ARCHITECTURE.md` — Code Map section
- `docs/design/testing-strategy.md` — testing strategy (connected to this structure)
- `CONTRIBUTING.md` — dependency direction rules for the three layers (contributor summary)
- ADR-0001: Adopt gpui as the GUI Framework
