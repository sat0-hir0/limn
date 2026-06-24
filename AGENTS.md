# AGENTS.md

Limn — a keyboard-first, AI-integrated block editor for `.md` files.
Built on `gpui`, Rust native.

"Limn" is the Old English verb for sketching the outline of something —
making it visible by drawing it lightly. That's the user-facing
experience the project is built around.

This file is the vendor-neutral source of truth for AI coding tools
(Claude Code, Codex, Cursor, Gemini CLI, GitHub Copilot, and others).
Human contributors should read [`CONTRIBUTING.md`](CONTRIBUTING.md) and
[`ARCHITECTURE.md`](ARCHITECTURE.md) first.

## Project layout

Cargo workspace with three crates. Dependency direction is one-way:

```
limn-ui → limn-service → limn-core
limn-ui ───────────────→ limn-core
```

- `crates/limn-core` — Functional core: block tree, Markdown round-trip, completion. `std` only.
- `crates/limn-service` — I/O, link index, AI calls. Depends on `limn-core`.
- `crates/limn-ui` — `gpui` bindings, command palette. Depends on both.

Reverse dependencies are forbidden.

## Build, test, lint

Requires Rust stable (pinned via `rust-toolchain.toml`).

```sh
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

All four must be green before opening a PR. `lefthook` enforces these
on `pre-push` automatically once installed (`lefthook install`).

## Conventions

- **Code style**: `rustfmt` + `clippy::pedantic`, machine-enforced.
- **Commit messages**: [Conventional Commits](https://www.conventionalcommits.org/).
  release-please consumes them to manage version and `CHANGELOG.md`.
  Do not edit `CHANGELOG.md` by hand.
- **License**: Apache-2.0. Contributions are licensed under the same
  (Apache-2.0 §5 covers this without a separate CLA).

## AI tooling

This repo ships vendor-neutral AI scaffolding. Using any specific tool
is **optional** and **not required to be disclosed**.

- **Skills**: project skills live under `.skillshare/skills/`
  ([skillshare](https://github.com/runkids/skillshare) is the sync tool).
  Run `skillshare sync` once to install them into your tool of choice
  (Claude Code, Codex, Cursor, etc.).
- **Hooks**: `lefthook.yml` runs `gitleaks` on `pre-commit`,
  `fmt --check / clippy -D / test` plus a second `gitleaks` sweep on
  `pre-push`, and a Conventional Commits format check on `commit-msg`.
  Install with `lefthook install`.
- **Secret scanning**: `gitleaks` (configured in `.gitleaks.toml`)
  guards against API keys and personal information leaking into
  commits. CI re-runs the scan as a backstop.
- **Claude Code**: reads `CLAUDE.md`, which is a one-line import of this
  file (`@AGENTS.md`). This is the Anthropic-recommended workaround
  while native `AGENTS.md` support lands.

Per-tool personal areas (`.claude/`, `.cursor/`, `.codex/`, `.gemini/`)
are gitignored and treated as developer-local — do not commit them.

## AI operating skills

A small set of repo-internal AI skills lives under
[`.skillshare/`](.skillshare/OVERVIEW.md). These cover **OSS operating
procedures only** — ADR governance, release flow, Conventional Commits,
and the English-only doc rule. Personal verifiers and orchestrators are
not committed here; maintainers keep those in their own dotfiles.

Skills are **optional** for contributors. The hard quality gates run
through `lefthook` (pre-push) and CI and do not require any AI tool.

| When working on… | Read first | Then invoke |
|---|---|---|
| New design judgement | `docs/adr/*.md`, `ARCHITECTURE.md` Open Questions | `$adr-proposal` |
| Promoting an ADR | `docs/adr/NNNN.md` | `$adr-acceptance` |
| Public-behaviour changes (new `pub mod`, new `[[bin]]`, argv / Launch enum changes) | `docs/adr/`, `docs/adr/template.md` | `$adr-required-check` (advisory) → `$adr-proposal` if signal stands |
| ADR consistency drift suspected | `docs/adr/` | `$adr-consistency-check` |
| Commit / branch / PR title | `docs/development/git-strategy.md` | `$conventional-commits-check` |
| `CHANGELOG.md` change suspected | `CHANGELOG.md` | `$changelog-untouched-check` |
| Committed Markdown language | `docs/design/testing-strategy.md` | `$doc-language-check` |
| release-please PR appears | `docs/maintainer-runbook/release-public.md` | `$release-preparation` → `$release-confirmation` → `$post-release-followup` |

Each skill is self-contained at `.skillshare/skills/<group>/<name>/SKILL.md`;
machine execution lives under `scripts/verify/` and `scripts/release/`.
See [ADR-0004](docs/adr/0004-scope-skillshare-to-oss-operations.md)
for the scope rationale.

## Where to look next

- [`.skillshare/OVERVIEW.md`](.skillshare/OVERVIEW.md) — AI operating skills (optional)
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — architecture overview, code map, cross-cutting concerns
- [`docs/adr/`](docs/adr/) — architecture decision records (gpui, 3-crate layering, GPL contamination, AI skills)
- [`docs/design/testing-strategy.md`](docs/design/testing-strategy.md) — testing approach
- [`docs/design/basic-features.md`](docs/design/basic-features.md) — basic editor feature inventory
- [`docs/README.md`](docs/README.md) — map of everything under `docs/`
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — contribution flow
- [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) — Contributor Covenant 2.1
