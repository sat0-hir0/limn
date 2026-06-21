# Testing Strategy (Recommended Summary)

> Testing strategy for the block editor. Written to align with the architecture (Functional Core / Imperative Shell + domain organisation).
> Conclusion: testing trophy as the base, with dense unit tests for the pure core (hybrid). Workflow: TDD-style for the core, test-after for exploratory areas.

---

## 1. Overall Approach

- **Shape**: Base the strategy on the testing trophy (integration tests are the primary layer, the thickest). However, keep unit tests dense for the pure logic in the `Functional Core` — a hybrid with the pyramid.
- **Workflow**: Where the spec is well-defined, work TDD-style (tests first). Where the spec is "figure it out as we go", work test-after (build first, then test).
- **Rust assumption**: static analysis (types, clippy, ownership checker) catches a large share of the bugs that other languages catch with tests, at compile time. The bottom layer of the trophy is already thick without writing a single test.

Rationale: this editor has two distinct characters. The `Functional Core` (block operations, Markdown conversion, completion rules) is pure logic — pyramid-like, so unit tests are effective. The integration boundary between UI, file I/O, and AI is trophy-like: things only gain meaning when wired together. Use each approach where it fits.

---

## 2. The Four Layers

### Static Analysis (bottom layer — free guardian)
- Type checker / `cargo clippy` / ownership checker
- Null-pointer dereferences and use-after-free are structurally impossible
- Already thick without writing tests. Enforce `clippy` in CI

### Unit Tests (foundation — large volume, dense)
- Target: pure functions under `core/`
- Examples:
  - `block`: do insert, move, and fold operations return the expected tree?
  - `markdown`: **does the md → tree → md round-trip lose any information?** (top priority)
  - `completion`: does `#` input produce heading candidates? (rule-based portion)
  - `link`: is `[[ ]]` parsing correct?
- Pure functions — only inspect inputs and outputs. No setup needed, extremely fast. Use Rust's standard `#[test]` adjacent to each module

### Integration Tests (primary layer — thickest)
- Target: user-scenario-level tests combining `core` + `shell`
- Examples:
  - Open a file → insert a block → move it → verify the autosaved `.md` is correct
  - Write `[[link]]` → verify the backlink index is updated
  - Insert a table via `/` command → verify correct reflection in Markdown
- Validates behaviour, not implementation details — resilient to refactoring

### End-to-End Tests (apex — few, lifeline only)
- Target: launch gpui, simulate real key input, assert screen state
- Examples: launch → type `# ` → rendered as a heading / can edit and save
- Keep the count small (slow and brittle). Only critical paths
- **Note**: gpui is a native GUI; there is no Playwright equivalent. Use gpui's own test machinery (`TestAppContext` and the headless event-dispatch system). Requires investigation before starting

---

## 3. Workflow — When to Write Tests

| Target | Workflow | Reason |
|--------|----------|--------|
| `markdown` conversion, `block` operations | **TDD-style** (tests first) | Well-defined spec; inputs and outputs are clear |
| `link`, `vault` | TDD-style | Spec is relatively well-defined |
| Completion "feel", UI behaviour | **Test-after** | Exploratory territory; spec is still moving |
| Integration scenarios | BDD-style ("given X, when Y, then Z") | Good fit for the trophy integration layer |

---

## 4. Most Important Tests for This Project

- **Round-trip tests (md ⇄ tree) are the lifeline.** The design relies on "blocks internally, `.md` on disk"; information loss in conversion is fatal. Cover exhaustively with unit tests. Use diverse syntax as input: headings, lists, code blocks, tables, links, images, nested structures.
- **Undo/Redo consistency.** Apply an edit operation, then undo — does it restore the original state? Include undo for block moves.
- **Completion rule regression.** Every time a new rule is added, verify that existing candidates are not broken.

---

## 5. CI Execution Order (fastest first)

1. `cargo clippy` (static analysis) — instant
2. Unit tests (pure core) — fast
3. Integration tests (core + shell) — somewhat slow
4. End-to-end tests (gpui launch) — slow; run last

Run more frequently toward the bottom, less frequently toward the top. Locally, run steps 1–2 continuously; run steps 3–4 at push time.

---

## Summary (one line)

**Testing trophy as the base, with dense unit tests for the pure core. Make the round-trip test the lifeline. TDD-style where the spec is solid, test-after where it is exploratory. Leverage Rust's static analysis as the guardian at the bottom layer.**
