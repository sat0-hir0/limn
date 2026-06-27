# Feature flag inventory

A list of all flags currently active under the
[feature flag policy](feature-flags.md).
When a PR adds, promotes, or removes a flag, update this document in
the same PR.

| Flag | Env var | Stage | Added | Owner | Notes |
|---|---|---|---|---|---|
| `edit` | `LIMN_FEAT_EDIT` | 1: hidden | M2 (Issue #3) | sat0-hir0 | Editable editor backed by `gpui-component` `InputState` (ADR-0005). Wave 1: input / cursor / selection / delete / copy-cut-paste / undo-redo / IME. Autosave is a later wave. |

## Stage reference

See [feature-flags.md](feature-flags.md) for the full policy.

- **1: hidden** — env var only, not shown in UI
- **2: experimental** — env var + UI toggle (Experimental Features screen)
- **3: stable** — on by default, flag code removal phase
