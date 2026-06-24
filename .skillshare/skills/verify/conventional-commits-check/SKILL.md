---
name: conventional-commits-check
description: Validate branch name, commit messages, and PR title against the Conventional Commits convention used by release-please.
---

# Verify Conventional Commits

## Use when

- Any commit on a feature branch (lefthook commit-msg already
  catches this locally; this skill is the AI-side gate).
- PR title finalisation.

## Contract

- Each commit subject matches `^(feat|fix|chore|docs|refactor|test|style|perf|ci|build|revert)(\([^)]+\))?!?: .+`.
- Branch prefix matches one of the types in
  `docs/development/git-strategy.md`.
- PR title is itself a Conventional Commit (it becomes the squash
  commit subject).

## Helper

```
scripts/verify/conventional-commits-check.sh
```

- exit 0: clean
- exit 1: list of malformed subjects / branch / PR title

## Stop condition

- Helper exit code.

## Boundary

- **Never** disable the lefthook commit-msg check.
- **Must** correct a malformed subject by amending **or** by
  opening a new commit (per [history-rewrite
  policy](../../../docs/development/git-strategy.md)).

## Final Report

```yaml
conventional-commits-check:
  status: clean | failed
  malformed:
    - kind: commit | branch | pr-title
      value: <text>
```
