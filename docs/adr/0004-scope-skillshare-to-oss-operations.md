# ADR-0004: Scope `.skillshare/` skills to OSS operating procedures

- **Status**: Accepted
- **Date**: 2026-06-23
- **Deciders**: sat0-hir0

---

## Context

Limn is a public OSS project with a single maintainer today. The
repository ships a small set of AI skills under `.skillshare/` to
make the maintainer's expectations legible to anyone (human or AI
tool) working on the project.

Two distinct kinds of skill could live under that directory:

1. **OSS operating procedures** — ADR governance, release flow,
   Conventional Commits, English-only docs, `CHANGELOG.md`
   immutability. These are *contributor-facing*: they describe how the
   project itself is run, regardless of who is contributing.
2. **Personal verifiers and orchestrators** — the maintainer's
   preferred workflow scripts (task routing, scope governance, AI
   output audits, layered-dependency probes, panic-site checks, etc.).
   These are *maintainer-personal*: they encode one person's process,
   not a project rule.

A survey of comparable Rust / native-tooling OSS projects (biome, zed,
deno, ruff) shows that committed skill directories contain only the
first category. Personal verifiers live in each maintainer's own
dotfiles, not in the repo.

Including personal verifiers in `.skillshare/` has a real cost:
contributors reasonably read anything under a published skill
directory as something they are expected to run, which turns a
maintainer's process preference into an implicit requirement.

## Decision

We restrict `.skillshare/` to **OSS operating procedures only**.
Personal verifiers and orchestrators are removed from the repository
and kept in the maintainer's own dotfiles instead.

Concretely, the committed skills cover exactly these areas:

- ADR governance (`adr-proposal`, `adr-acceptance`,
  `adr-required-check`, `adr-consistency-check`)
- Release flow (`release-preparation`, `release-confirmation`,
  `post-release-followup`)
- Conventional Commits (`conventional-commits-check`)
- `CHANGELOG.md` immutability (`changelog-untouched-check`)
- English-only committed Markdown (`doc-language-check`)

The skills remain **optional**: contributors who do not use AI tools
are unaffected. The hard quality gates continue to run through
`lefthook` (pre-push) and CI; they do not depend on these skills.

## Consequences

### Positive

- `.skillshare/` matches the convention seen in comparable OSS Rust
  projects — committed skills describe project rules, not personal
  workflow.
- Contributors are not implicitly asked to adopt the maintainer's
  preferred AI workflow.
- The skill surface stays small enough that drift between published
  triggers and shipped skills is easy to keep in check.

### Negative / Trade-offs

- The maintainer's personal verifiers are no longer discoverable from
  the repository; they live in dotfiles and must be installed
  separately to be used.
- Future contributors using AI tools do not inherit the maintainer's
  process — they bring (or build) their own.

## Considered Alternatives

### Alternative A: Keep the full skill set in the repository

- Summary: leave all personal verifiers and orchestrators under
  `.skillshare/` so anyone cloning the repo gets the full toolkit.
- Reason for rejection: a published skill set reads as a contributor
  requirement, and committing one person's workflow into a public OSS
  repo is out of step with the convention seen in similar projects.

### Alternative B: Split into two directories

- Summary: keep OSS-operating skills under `.skillshare/` and add a
  second directory (e.g. `.skillshare-personal/`) for the maintainer's
  workflow, gitignored or not.
- Reason for rejection: the maintainer's personal skills are already
  managed through dotfiles. Mirroring them inside the repo adds a
  sync surface for no contributor benefit.

---

## Links

- Related ADR: [ADR-0003](0003-temporarily-accept-gpl-contamination.md)
  — also uses `.skillshare/trace/` for maintenance follow-up.
