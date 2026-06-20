# Technical debt

Single source of truth for technical debt in this repo. Debt that takes
more than a few minutes to clear should land here (or in a GitHub issue
labelled `tech-debt`) so it does not get forgotten.

Inspired by the workflow we run on a sibling project, but lighter:
there is no scanner, no baseline JSON, no automation. Updating this
file is part of finishing a piece of work.

## How to use this file

- **Add an entry** when you ship code that you know is incomplete,
  taking a shortcut, or putting off a refactor.
- **Always write the resolution condition.** A debt entry without a
  trigger to clear it is just a complaint.
- **Mark the source.** Commit SHA or PR number, so future readers can
  see where the debt came from.
- **Move resolved items** from Open to Resolved (don't delete) so the
  history of what we paid down stays visible.
- **GitHub issue vs. debt.md**: use the `tech-debt` issue label for
  debt that needs design discussion or that affects more than one area.
  Use `docs/debt.md` for debt that is mostly self-contained and just
  needs to be remembered.

## Baseline metrics

A snapshot of the repo at the time the debt-tracking process was put in
place. CI re-computes these on every push and reports the delta. The
numbers here are not a target — they are a starting point. Going up is
not automatically bad; **going up unexplained is**.

Baseline date: 2026-06-21
Baseline commit: `9f325ff` (M0.7)

| Metric                            | Baseline | Source                                                          |
| --------------------------------- | -------: | --------------------------------------------------------------- |
| Rust source lines (`crates/**/*.rs`) | 193   | `find crates/ -name '*.rs' -exec wc -l {} +`                    |
| External cargo dependencies       |        0 | `grep -E '^[a-z0-9_-]+ = ' crates/*/Cargo.toml \| grep -v 'path =' \| grep -v 'workspace'` |
| Workspace internal `path =` deps  |        4 | same as above, inverted                                         |
| `clippy --workspace --all-targets` warnings (default level) | 0 | `cargo clippy --workspace --all-targets`                        |
| `unsafe` blocks                   |        0 | forbidden via `workspace.lints.rust.unsafe_code = "forbid"`     |
| `TODO` / `FIXME` / `XXX` markers  |        0 | `grep -rnE 'TODO\|FIXME\|XXX' crates/ docs/`                   |
| `#[allow(dead_code)]` annotations |        0 | `grep -rn 'allow(dead_code)' crates/`                          |
| Tests passing                     |        0 | `cargo test --workspace`                                       |
| Tests `#[ignore]`d                |        5 | `cargo test --workspace` summary                               |
| Tests failing                     |        0 | `cargo test --workspace`                                       |
| Test coverage                     |   not yet | M2 will introduce `cargo-llvm-cov`                             |

The five ignored tests are deliberate: they are the Markdown round-trip
contract from `docs/testing-strategy.md`, scaffolded but not yet
implementable until M2.

## Open debt

No entries.

The repo is intentionally near-empty right now (M0 .. M0.7 set up the
project, not the product). The first real entries will appear as M1
(gpui integration) lands.

## Resolved debt

No entries.

## Trends (computed by CI)

CI updates this section on every push to `main`. Do not edit by hand —
overrides are reset by the next CI run.

<!-- TRENDS:BEGIN -->
(populated by `.github/workflows/debt-metrics.yml` after first run)
<!-- TRENDS:END -->
