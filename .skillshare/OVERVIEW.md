# `.skillshare/` — Limn's AI operating skills

This directory holds Limn's vendor-neutral skill source-of-truth for
**OSS operating procedures only** — ADR governance, release flow,
Conventional Commits, English-only docs, and CHANGELOG immutability.
[skillshare](https://github.com/runkids/skillshare) syncs the skills
here into each AI tool's directory (`.claude/`, `.codex/`, `.agents/`,
etc.), which are all `.gitignore`'d.

Personal verifiers and orchestrators (`layer-check`, `pedantic-check`,
`task-completion`, etc.) are deliberately **not** committed here; they
live in each maintainer's own dotfiles. See `AGENTS.md`
"AI operating skills" for the published skill set.

**These skills are optional.** Human contributors who do not use AI
tools can ignore them entirely — the standard contribution workflow
described in [`CONTRIBUTING.md`](../CONTRIBUTING.md) does not depend
on anything here.

## Layout

```
.skillshare/
├── OVERVIEW.md               ← this file
├── config.yaml               ← skillshare sync targets
├── skills/                   ← skill source-of-truth (10 skills)
│   ├── design/               ── ADR proposal + acceptance
│   ├── verify/               ── adr-consistency / adr-required /
│   │                            changelog-untouched / conventional-commits /
│   │                            doc-language
│   └── release/              ── release-preparation / release-confirmation /
│                                post-release-followup
└── trace/                    ← lifecycle-tracking docs
```

Machine execution lives under the repository root at
`scripts/{verify,release,helpers}/`, separate from this directory so
that skills cite stable, vendor-neutral paths.

## Skill grammar

Every `SKILL.md` follows the same six-section shape:

```markdown
---
name: <skill-name>
description: <one-line summary>
---

# <Skill Title>

## Use when            ← trigger conditions
## Contract            ← what this skill guarantees
## Phase 1: …          ← time-ordered steps
  ### Step 1-1: …      (Read / Decide / Output / Next)
## Stop condition      ← success / failure / abort
## Boundary            ← Never / Do not / Must / Stop
## Helper              ← scripts/ to invoke
## Final Report        ← what gets recorded
```

Skills call each other by `$skill-name`. Skills never read `docs/`
directly — `docs/` is for human readers; these skills are for agents.
`scripts/verify/skills-self-check.sh` enforces this shape and resolves
every `$skill-name` cross-reference back to a real skill directory.

## Trigger map

The high-level trigger table is in [`AGENTS.md`](../AGENTS.md) under
"AI operating skills". Start there to find the right entry point for
your current task.

## Quickstart

Pick the line that matches what you are about to do.

| Goal | Command |
|---|---|
| Check ADR consistency | `bash scripts/verify/adr-consistency-check.sh` |
| Check whether a public-behaviour change needs an ADR | `bash scripts/verify/adr-required-check.sh` |
| Check CHANGELOG immutability | `bash scripts/verify/changelog-untouched-check.sh` |
| Check Conventional Commits | `bash scripts/verify/conventional-commits-check.sh` |
| Check English-only docs | `bash scripts/verify/doc-language-check.sh` |
| Inspect the release-please PR | `bash scripts/release/release-preparation.sh` |
| Confirm a tag + Pages build | `bash scripts/release/release-confirmation.sh [TAG]` |
| Run post-release follow-up | `bash scripts/release/post-release-followup.sh` |
| Self-test the skill set itself | `bash scripts/verify/skills-self-check.sh` |

All verifier scripts exit `0` on clean, `1` on a real finding, and
`2` on a script error (missing tool, bad metadata). JSON findings go
to stderr; high-level status lines use the `[info]/[ok]/[error]`
prefixes defined in `scripts/helpers/lib.sh`.

## Distribution

Skills are distributed across AI tools by skillshare:

```sh
skillshare sync
```

Targets are listed in [`config.yaml`](config.yaml). All per-vendor
directories created by `skillshare sync` (`.claude/`, `.codex/`,
`.agents/`, etc.) are `.gitignore`'d — the source of truth is here.

## Status

The architectural decision backing this directory is:

- [ADR-0004](../docs/adr/0004-scope-skillshare-to-oss-operations.md)
  — scope `.skillshare/` skills to OSS operating procedures.
