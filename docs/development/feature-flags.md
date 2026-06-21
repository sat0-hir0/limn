# Feature flags

Limn avoids long-lived branches by **merging incomplete or experimental
features into main while hiding them behind a feature flag**. Flags
follow a three-stage maturity model; each stage uses a different toggle
mechanism.

## TL;DR

| Stage | Code state | Toggle | User view |
|---|---|---|---|
| **1: hidden** | Incomplete, merged to main | Env var only (`LIMN_FEAT_<NAME>`) | Not visible; users are unaware |
| **2: experimental** | Unit-tested, may still be buggy | Env var + Settings UI toggle | Can opt in at their own risk |
| **3: stable** | Stable, on by default | Flag code removed | Available as a normal feature |

Server-side rollout (Finch-equivalent) is **permanently out of scope**.
This is a deliberate privacy decision and a prerequisite for trust in an
OSS desktop application.

## Why feature flags

- **Merge incomplete features into main** — avoids long-lived branches,
  merge conflicts, and stale code ([git-strategy.md](git-strategy.md))
- **Contributors and early adopters can opt in** — enables early bug
  reports
- **Incomplete code ships disabled (or not at all)** — preserves release
  quality
- **Maintainers can explicitly track what is in main but not yet done**

## Three-stage model

### Stage 1: hidden

The feature is incomplete; its contract is not yet settled.

- Toggle via env var only (`LIMN_FEAT_<NAME>=1`)
- **Not shown** in the Settings UI
- Users are unaware the flag exists and cannot toggle it
- Purpose: maintainer and early contributors iterate on fixes

Example:

```sh
# Try the editing feature as a maintainer
LIMN_FEAT_EDIT=1 cargo run -p limn-ui
```

### Stage 2: experimental

Unit tests pass and the contract is settled, but the feature may still
be buggy.

- Toggle via env var **or** the Settings UI toggle
- Listed with a warning in the **Experimental Features** section of
  Settings
- Real users can opt in and try it at their own risk — real usage
  produces bug reports
- The env var takes priority over the config file (so maintainers can
  override config during debugging)

Settings UI mockup (inspired by `chrome://flags`):

```
Settings → Experimental Features
─────────────────────────────────────
⚠ Use at your own risk. These features may be
   incomplete or unstable.

[ ] Editing (Experimental)
    Edit and save Markdown files. Requires restart.
    May cause data loss in rare edge cases — backup
    important files.

[ ] Slash palette (Experimental)
    Insert blocks via the / shortcut.

──────
Reset all experimental flags
```

### Stage 3: stable

No bug reports for a sustained period of real-world usage, or the
maintainer explicitly declares stability.

- On by default; flag code is removed
- Removed from the Settings UI
- Available as a normal feature

## Promoting and demoting flags

| Transition | Condition |
|---|---|
| **1 → 2** | Unit tests complete (contract is settled) |
| **2 → 3** | 3 months or 1 milestone passes with no bug reports, or maintainer declares stable |
| **3 → 2** | Increasing bugs require pulling the feature back to experimental (emergency) |
| **2 → 1** | Design rework requires removing the GUI toggle (emergency) |
| **1 → remove** | Flag sits hidden for 1 month — file an issue asking "is this still alive?" |

## Implementation pattern (Pattern B: aggregated struct)

The Rust ecosystem has no de-facto feature flag crate. Most projects
use `std::env::var` directly. Limn adopts **Pattern B — a single
aggregated struct** — which covers the requirement in about 30 lines.

```rust
// crates/limn-ui/src/feature_flags.rs
#[derive(Debug, Clone, Default)]
pub struct FeatureFlags {
    pub edit: bool,
    pub palette: bool,
    pub ai: bool,
}

impl FeatureFlags {
    /// Read all `LIMN_FEAT_*` env vars at startup. Truthy values
    /// (case-insensitive) are `1`, `true`, `on`, `yes`. Anything else
    /// (or absent) is OFF.
    pub fn from_env() -> Self {
        Self {
            edit: env_truthy("LIMN_FEAT_EDIT"),
            palette: env_truthy("LIMN_FEAT_PALETTE"),
            ai: env_truthy("LIMN_FEAT_AI"),
        }
    }
}

fn env_truthy(name: &str) -> bool {
    matches!(
        std::env::var(name)
            .ok()
            .as_deref()
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("1" | "true" | "on" | "yes"),
    )
}
```

Flags are read once at startup and accessed throughout the widget tree
via `cx.global::<FeatureFlags>()`. At v0.1.0 there are no active flags
(the struct itself is introduced in M2).

### Evolving to Stage 2

Add `from_env_and_config` alongside `from_env`. The struct shape does
not change; only the source of values expands.

```rust
impl FeatureFlags {
    pub fn from_env_and_config(config: &Config) -> Self {
        Self {
            // Env var takes priority over config (for maintainer debugging)
            edit: env_truthy("LIMN_FEAT_EDIT") || config.flags.edit,
            ...
        }
    }
}
```

### Evolving to Stage 3 (removing a flag)

- Remove the field from the struct
- Remove the corresponding line from `from_env`
- In all call sites, replace `if flags.edit { ... } else { ... }` with
  only the `then` branch
- Remove the corresponding toggle from the Settings UI
- Update [`flag-inventory.md`](flag-inventory.md)

## Naming conventions

| Target | Convention |
|---|---|
| Env var | `LIMN_FEAT_<NAME>` (`<NAME>` is SCREAMING_SNAKE_CASE) |
| Struct field | `snake_case` (e.g. `pub edit: bool`) |
| Settings UI label | English feature name + " (Experimental)" |
| Flag name | **The feature itself** — do not use milestone names (`m2`, `m3`) |

Using milestone names for flags causes confusion: when the milestone is
complete but the flag lingers, it is no longer clear what the flag
controls.

## Truthy values for env vars

| Value (case-insensitive) | Result |
|---|---|
| `1` / `true` / `on` / `yes` | ON |
| Any other value (`0` / `false` / `off` / `no` / arbitrary string) | OFF |
| Env var absent | OFF |

Evaluation is **value-based, not presence-based** (`is_ok`), so
`LIMN_FEAT_EDIT=0` explicitly turns the flag off.

## What we deliberately omit

- **Server-side rollout** (Finch / LaunchDarkly style): the privacy
  policy prohibits telemetry delivery in an OSS desktop app; it would
  also undermine user trust
- **A/B testing**: the user base is too small in the early stages for
  meaningful statistics, and there is no telemetry infrastructure
- **Feature flag crate** (`features`, `featureflag`, etc.): no
  industry-standard crate exists; Pattern B covers the need in ~30 lines

## Flag inventory

All currently active flags are listed in
[`flag-inventory.md`](flag-inventory.md). When a PR adds, promotes, or
removes a flag, update that document in the same PR.

## Related docs

- [`git-strategy.md`](git-strategy.md) — branch / PR / squash-merge / release
- [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md) — contributor summary
