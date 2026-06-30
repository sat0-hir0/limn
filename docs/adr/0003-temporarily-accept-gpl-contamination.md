# ADR-0003: Temporarily Accept GPL Contamination

- **Status**: Accepted
- **Date**: 2026-06-21
- **Deciders**: sat0-hir0

---

## Context

Limn is published as open-source software under the Apache-2.0 licence.
As a consequence of adopting gpui (→ ADR-0001), the following transitive
dependency chain exists:

```
gpui            (Apache-2.0)
  └── sum_tree  (Apache-2.0)
        └── ztracing       ← GPL-3.0-or-later
              └── ztracing_macro  ← GPL-3.0-or-later
              └── zlog            ← GPL-3.0-or-later
```

`sum_tree` itself is `Apache-2.0`; the GPL surface starts at
`ztracing` (and its sibling `ztracing_macro`) and continues into
`zlog`. The cargo-deny configuration (`deny.toml`) covers exactly
these three GPL crates and no others.
GPL-3.0 and Apache-2.0 are compatible, but with a directional constraint:

- Apache-2.0 code may be incorporated into a GPL-3.0 project.
- **When GPL-3.0 code is incorporated into an Apache-2.0 project, binary
  distribution triggers GPL-3.0 conditions (source disclosure, copyleft, etc.).**

Limn intends to signal through Apache-2.0 that users may use, modify, and
redistribute the project commercially. Under the current setup, any binary
distribution must satisfy GPL-3.0 conditions. This is the state we call
"GPL contamination."

What `zlog` actually does is close to a no-op log decorator. The functional
impact is negligible; this is purely a licence compatibility issue.

The gpui upstream maintainers are aware of this and working on a fix, tracked by
[zed-industries/zed#55470](https://github.com/zed-industries/zed/issues/55470).

---

## Decision

**We temporarily accept the GPL contamination until the upstream issue is resolved.**

Concretely:

- We disclose in the "Known limitations" section of `README.md` that `zlog`
  (GPL-3.0-or-later) is included in binary distributions.
- We open a tracking issue on GitHub and link it to the upstream issue.
- We revisit this decision when any of the triggers in the
  "Reassessment trigger" section below fire. Until then we take no
  alternative action.

Rationale:

- `zlog` is functionally close to a no-op, so the cost of replacing it does
  not justify the effort.
- The upstream maintainers are aware of and actively addressing the issue;
  waiting for the upstream fix is the rational course.
- Limn v0.1.0 is in a pre-binary-distribution OSS publication phase. Taking a
  temporary measure while downloads and impact are still limited is acceptable.

---

## Consequences

### Positive

- The problem is deferred at zero implementation cost.
- When the upstream fix lands, updating `Cargo.lock` is all that is needed.
- The upstream maintainers share the same problem, giving them strong motivation to fix it.

### Negative / Trade-offs

- **Binary distribution is inconsistent** — distributing binaries requires
  satisfying GPL-3.0 copyleft conditions (source disclosure, licence notices,
  etc.)
- **Upstream fix timeline is uncertain** — it is unknown when
  zed-industries/zed#55470 will be resolved, and it may remain open
  for an extended period
- **Risk of user confusion** — the repository states "Apache-2.0" but GPL
  conditions effectively apply; users knowledgeable about licences may find
  this confusing

### Neutral

- Publicly disclosing and tracking the issue provides transparency to users
  and contributors

---

## Considered Alternatives

### Alternative A: Fork gpui and remove the `zlog` dependency

- **Summary**: Maintain a Limn-specific fork of gpui that strips `ztracing` /
  `zlog` from `sum_tree`, or write a custom `sum_tree` implementation.
- **Reason for rejection**: Implementation cost is very high. Because gpui is
  pre-1.0 and changes frequently, the cost of maintaining a fork is ongoing.
  A prior investment of this scale is not justified during the period when
  waiting for the upstream fix is still rational.

### Alternative B: Switch to a different GUI framework

- **Summary**: Replace gpui with `egui`, `iced`, Tauri, or another framework
  that does not include `zlog`.
- **Reason for rejection**: All alternatives were evaluated in ADR-0001; each
  involves trade-offs that undermine Limn's core values (zero-latency, GPU
  rendering, Rust native). Switching frameworks for a single licence issue is
  not justifiable at this point.

### Alternative C: Implement a custom `sum_tree` now to avoid `zlog`

- **Summary**: Re-implement `sum_tree` inside Limn without depending on
  `ztracing` / `zlog`. This corresponds to the four-stage path (b-1 through
  b-4) considered in `gpui-gpl-fix-path-b.md`.
- **Reason for rejection**: `sum_tree` is a core data structure of gpui, making
  implementation and maintenance costs high. With an upstream fix anticipated,
  the priority for implementing this in advance is low. If the triggers in
  the "Reassessment trigger" section fire without an upstream fix in place,
  this option will be reconsidered.

---

## Reassessment trigger

This decision is reassessed when any of the following conditions hold,
not on a fixed schedule:

1. [zed-industries/zed#55470](https://github.com/zed-industries/zed/issues/55470)
   is resolved upstream. Action: update `Cargo.lock` to eliminate the
   contamination.
2. Demand for binary distribution of Limn emerges while the upstream
   issue is still open (e.g. release artifacts, installers, packaged
   builds for end users). Action: re-evaluate the cost of alternative
   implementations against the cost of complying with GPL-3.0
   conditions on those binaries.
3. The `sum_tree` / `ztracing` / `zlog` dependency surface is
   restructured upstream (for example, `sum_tree` no longer depends on
   `ztracing`, or `zlog` is split out) such that the contamination can
   be avoided with a `Cargo.toml` change rather than a fork. Action:
   adopt the new dependency shape.
4. A licence audit, downstream redistribution request, or security
   review surfaces a concrete user impact from the contamination.
   Action: prioritise mitigation in proportion to the impact.

---

## Links

- Upstream issue: [zed-industries/zed#55470](https://github.com/zed-industries/zed/issues/55470)
- ADR-0001: Adopt gpui as the GUI Framework
- `README.md` — Known limitations section
- `CHANGELOG.md` — Known limitations (v0.1.0 entry)
- `docs/maintainer-runbook/release-public.md` — procedure for disclosing Known limitations at release
