---
name: adr-required-check
description: Flag PRs that change public behaviour without recording the decision as an ADR (advisory).
---

# Verify ADR required

## Use when

- Any PR that touches `pub mod` declarations under `crates/*/src/`.
- Any PR that adds a `[[bin]]` entry or argv handling in `main.rs`.
- Any PR that introduces a new `Launch`/`Mode`/`Workspace` enum
  variant — these signal an app-level mode that did not exist
  before.

The check is **advisory**: the goal is not to block every refactor
that happens to touch a `pub mod` line, but to make it harder to
silently land a public-behaviour change without writing the ADR
that `$adr-proposal` requires.

## Contract

- A diff containing one or more public-behaviour signals **and**
  no new `docs/adr/NNNN-*.md` exits non-zero.
- A diff containing public-behaviour signals **and** a new ADR
  exits clean.
- A diff containing no signals exits clean (vacuous).
- A commit message on the branch carrying `[skip-adr-required]`
  overrides the check (= the maintainer explicitly judged that no
  ADR was needed — useful for pure refactors).

## Helper

```
scripts/verify/adr-required-check.sh
```

- exit 0: no signals, or an ADR was added, or skip flag set
- exit 1: public-behaviour signal(s) without an ADR
- exit 2: script error

`ADR_REQUIRED_BASE` overrides the diff base ref (default
`origin/main`, fallback `main`).

## Stop condition

- Helper exit code.

## Boundary

- **Never** treat a clean exit as proof an ADR was unnecessary;
  the check is heuristic-based and may miss subtle decisions.
- **Never** suppress the warning by amending the commit message
  with `[skip-adr-required]` to dodge writing the ADR — the flag
  is for refactors where no decision was made, not for sidestepping
  documentation work.
- **Must** be paired with `$adr-proposal` when the signal is real:
  the warning is the trigger, not the resolution.

## Final Report

```yaml
adr-required-check:
  status: clean | failed | skipped
  signals: [<signal-line>...]
  adr_added: <path-or-null>
```
