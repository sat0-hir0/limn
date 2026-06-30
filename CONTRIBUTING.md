# Contributing

Everyone participating in this project is expected to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Development workflow

1. Open an issue (or pick up an existing one).
2. Create a branch (`feat/<topic>`, `fix/<topic>`, `chore/<topic>`, etc. — see [`docs/development/git-strategy.md`](docs/development/git-strategy.md) for the full naming convention).
3. Verify all of the following pass locally before opening a PR:
   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
4. Open a PR. The PR title must follow [Conventional Commits](https://www.conventionalcommits.org/) format — it becomes the commit message after squash-and-merge.
5. Merge only after all CI jobs are green.

For branch, merge, and release rules in full, see [`docs/development/git-strategy.md`](docs/development/git-strategy.md).

## Incomplete / experimental features (feature flags)

To avoid long-lived branches, incomplete or experimental features are merged into `main` hidden behind a feature flag. Three stages:

| Stage | How to enable | User-visible? |
|---|---|---|
| **1: hidden** | Environment variable only (`LIMN_FEAT_<NAME>=1`) | No — unreachable without the env var |
| **2: experimental** | Env var + settings UI | At-your-own-risk opt-in |
| **3: stable** | Flag removed | On by default |

When a PR adds, promotes, or removes a flag, update [`docs/development/flag-inventory.md`](docs/development/flag-inventory.md) in the same PR.

There is no server-side rollout (no equivalent of Chrome Finch). See [`docs/development/feature-flags.md`](docs/development/feature-flags.md) for details.

## Three-crate dependency rules

Only the following dependency directions are allowed. Reverse dependencies are blocking:

```
limn-ui ─→ limn-service ─→ limn-core
limn-ui ────────────────→ limn-core
```

- `limn-core`: `std` only. Do not add `tokio`, `gpui`, or I/O crates.
- `limn-service`: `limn-core` plus async / I/O crates only. Do not add `gpui`.
- `limn-ui`: all crates allowed, but business logic belongs in `limn-core` or `limn-service`, not here.

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/) is **required** — release-please uses commit messages to generate `CHANGELOG.md` and determine version bumps automatically.

Commit types:

| Type | Purpose | Appears in CHANGELOG |
|---|---|---|
| `feat` | New feature | ✅ Features |
| `fix` | Bug fix | ✅ Bug Fixes |
| `perf` | Performance improvement | ✅ Performance |
| `refactor` | Refactor with no behavior change | ✅ Refactoring |
| `docs` | Documentation | ✅ Documentation |
| `test` | Adding or updating tests | hidden |
| `ci` | CI / Actions changes | hidden |
| `chore` | Chores (dependency updates, etc.) | hidden |
| `style` | Formatting-only changes | hidden |

Scope is optional: `feat(core): add block parser` is fine.

### Breaking changes

```
feat!: change the public API of `limn-core::Block`

BREAKING CHANGE: `Block::new` now requires a parent reference.
```

Use `!` or a `BREAKING CHANGE:` footer. release-please triggers a major version bump for these (minor bump while the project is still on `0.x`).

### CHANGELOG.md

Do not edit `CHANGELOG.md` by hand — release-please updates it via PR.

## AI operating skills (optional)

This repo ships a small set of AI operating skills under
[`.skillshare/`](.skillshare/OVERVIEW.md) for maintainers using AI
coding tools, distributed via the
[skillshare](https://github.com/runkids/skillshare) sync utility.
Contributors **may** use them, but they are not required — the
standard contribution flow above does not depend on them. See
[ADR-0004](docs/adr/0004-scope-skillshare-to-oss-operations.md) for
context.

## AI tooling (optional)

This repo ships with vendor-neutral AI development scaffolding. Using any of
it (or any specific tool) is **optional** and **not required to be
disclosed**.

- [`AGENTS.md`](AGENTS.md) — onboarding context readable by Claude Code,
  Codex, Cursor, Gemini CLI, GitHub Copilot, and others.
- [`.skillshare/`](.skillshare/) — project-scoped skills managed by
  [skillshare](https://github.com/runkids/skillshare). Run
  `skillshare sync` once to install them into your AI tool of choice.
- [`lefthook.yml`](lefthook.yml) — Git hooks: `pre-commit` runs
  `gitleaks` over the staged diff; `pre-push` runs
  `cargo fmt --check / clippy -D / test` plus a second `gitleaks` sweep;
  `commit-msg` checks the Conventional Commits format. Install with
  `lefthook install`.
- [`.gitleaks.toml`](.gitleaks.toml) — Secret-scanning rules: the
  default ~150 patterns plus a handful of generic identity patterns
  (Anthropic / OpenAI org IDs, SSH key fingerprints). Run a
  full-history scan with `gitleaks detect --source . --log-opts="--all"`.

Per-tool personal areas (`.claude/`, `.cursor/`, `.codex/`, `.gemini/`)
are gitignored as developer-local.

### Local personal patterns (opt-in)

If you want gitleaks and `debt-scan` to also flag your own personally
identifying strings (developer username, real name, personal email),
opt in locally without touching committed files:

- **gitleaks**: copy the structure of `.gitleaks-local.toml` (already
  gitignored — see [`.gitignore`](.gitignore)) and adapt the patterns
  to your own identity. The shared lefthook hook auto-detects this
  file and uses it instead of `.gitleaks.toml` when present.
- **debt-scan**: export `DEBT_SCAN_PERSONAL_NAMES` as a comma-separated
  list of lowercase usernames you want to catch. With no value set the
  `personal-windows-path` count stays at zero.

  ```sh
  export DEBT_SCAN_PERSONAL_NAMES=alice
  cargo run -p debt-scan -- scan
  ```

Neither hook commits your name or email into the repo — they only
make the local checks louder for you.

## Quality metrics in CI (report-only)

The following checks run on every PR and push, but **a drop in any metric does not fail CI** — they are report-only for now and will be promoted to gates once the project gains traction.

- **`coverage` workflow** ([`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) + Codecov) — line, function, and region coverage. Coverage delta appears as a PR comment; the full HTML report is available as the `coverage-html` artifact.
- **`mutants` workflow** ([`cargo-mutants`](https://mutants.rs/)) — mutation testing. Runs on **weekly cron and manual dispatch only** (not per PR). Missed mutants are available as the `mutants-report` artifact.
- **`code-metrics` workflow** — `cargo-outdated` (dependency freshness overview) and `cargo-udeps` (unused dependency detection), on **monthly cron and manual dispatch**. Dependabot handles individual update PRs; this workflow provides a global view.

For the other workflows, the convention follows the OSS norm of soft gates — investigate when something regresses, but don't block on it automatically. To enforce strict thresholds, adjust `codecov.yml` or `.cargo/mutants.toml`.

## Technical debt

Code merged in an unfinished state, or shortcuts taken deliberately, must be recorded in [`docs/debt/dashboard.md`](docs/debt/dashboard.md). **Always include resolution criteria** — what condition closes the item. An entry without criteria is just a complaint.

- Automated metrics are handled by the [`debt-scan`](crates/debt-scan/) crate. Run `cargo run -p debt-scan -- scan` to measure and `-- gate` to compare against the baseline. CI posts a diff as a PR comment — explain any increase. This is **report-only** (no automatic failure) to start.
- Items that automated tooling cannot count — architecture decisions, cross-file concerns, upstream blockers — go into the Open debt section of `docs/debt/dashboard.md`.

**Technical debt is not tracked in GitHub Issues.** Everything goes through `docs/debt/dashboard.md` + `debt-scan`. GitHub Issues are reserved for bug reports and feature requests — the entry points for contributors.

## Opening an issue

GitHub Issues cover **two kinds of things only**:

- `bug` — something is broken or behaves unexpectedly.
- `enhancement` — a new feature or improvement to an existing one.

Questions, ideas, and general discussion go to [GitHub Discussions](https://github.com/sat0-hir0/limn/discussions) (may be disabled before the public launch).

Aligning issue titles with Conventional Commits types makes it easier to link them to PRs later (optional):

- `[bug] window fails to open on Wayland when GPU is absent`
- `[feat] add slash palette to limn-ui`
- `[docs] clarify build instructions for Windows`

## Design documents

Before starting work, read [ARCHITECTURE.md](ARCHITECTURE.md) and [docs/design/testing-strategy.md](docs/design/testing-strategy.md). Architecture decision rationale is in [docs/adr/](docs/adr/).
