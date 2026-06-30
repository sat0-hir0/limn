# Feature flag inventory

A list of all flags currently active under the
[feature flag policy](feature-flags.md).
When a PR adds, promotes, or removes a flag, update this document in
the same PR.

| Flag | Env var | Stage | Added | Owner | Notes |
|---|---|---|---|---|---|
| `edit` | `LIMN_FEAT_EDIT` | 1: hidden | Issue #3 | sat0-hir0 | Editable editor backed by `gpui-component` `InputState` (ADR-0005). Covers input / cursor / selection / delete / copy-cut-paste / undo-redo / IME, debounced autosave, the `AppShell` + `SettingsView` view-switching (ADR-0010), and the `ColorThemeGlobal` reactive-theme path (ADR-0011). |
| `palette` | `LIMN_FEAT_PALETTE` | 1: hidden | Issue #3 | sat0-hir0 | Command palette (`Ctrl+Shift+P`) with `Open File…` fuzzy search (ADR-0009) and `Open Settings`. Registered via gpui actions on the editable shell's focus chain (ADR-0008). |

## Stage reference

See [feature-flags.md](feature-flags.md) for the full policy.

- **1: hidden** — env var only, not shown in UI
- **2: experimental** — env var + UI toggle (Experimental Features screen)
- **3: stable** — on by default, flag code removal phase
