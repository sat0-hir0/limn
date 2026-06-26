# Feature inventory trace

Tracks the implementation status of each entry in
`docs/design/basic-features.md`. Maintained manually.

## Schema

| Field | Meaning |
|---|---|
| `FeatureID` | Stable id (matches `basic-features.md`) |
| `Status` | `not-started` / `in-progress` / `done` / `deferred` |
| `Impl` | File(s) under `crates/` implementing the feature |
| `Test` | File(s) under `crates/*/tests/` covering the feature |

## Entries

_(populated as features are tracked)_
