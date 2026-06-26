# Feature flag inventory

A list of all flags currently active under the
[feature flag policy](feature-flags.md).
When a PR adds, promotes, or removes a flag, update this document in
the same PR.

| Flag | Env var | Stage | Added | Owner | Notes |
|---|---|---|---|---|---|
| (none) | — | — | — | — | No flags at v0.1.0. |

## Stage reference

See [feature-flags.md](feature-flags.md) for the full policy.

- **1: hidden** — env var only, not shown in UI
- **2: experimental** — env var + UI toggle (Experimental Features screen)
- **3: stable** — on by default, flag code removal phase
