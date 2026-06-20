# AGENTS.md

A keyboard-first, AI-integrated block editor for `.md` files.
Built on `gpui` (Zed's GUI framework), Rust native.

This file is the vendor-neutral source of truth for AI coding tools
(Claude Code, Codex, Cursor, Gemini CLI, GitHub Copilot, and others).
Human contributors should read [`CONTRIBUTING.md`](CONTRIBUTING.md) and
[`docs/spec-handoff-gpui.md`](docs/spec-handoff-gpui.md) first.

## Project layout

Cargo workspace with three crates. Dependency direction is one-way:

```
editor-ui → editor-service → editor-core
editor-ui ───────────────→ editor-core
```

- `crates/editor-core` — Functional core: block tree, Markdown round-trip, completion. `std` only.
- `crates/editor-service` — I/O, link index, AI calls. Depends on `editor-core`.
- `crates/editor-ui` — `gpui` bindings, command palette. Depends on both.

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

## Where to look next

- [`docs/spec-handoff-gpui.md`](docs/spec-handoff-gpui.md) — full design
- [`docs/testing-strategy.md`](docs/testing-strategy.md) — testing approach
- [`docs/basic-features.md`](docs/basic-features.md) — basic editor feature inventory
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — contribution flow
- [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) — Contributor Covenant 2.1
