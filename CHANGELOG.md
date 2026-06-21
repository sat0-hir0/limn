# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1](https://github.com/sat0-hir0/limn/compare/v0.1.0...v0.1.1) (2026-06-21)


### Features

* initial public release (v0.1.0) ([3d7d928](https://github.com/sat0-hir0/limn/commit/3d7d928bd457ce1d7dc2f020eeb8e25f98f3d28e))


### Bug Fixes

* **ci:** drop lychee --base flag rejected by v0.23 ([34f7081](https://github.com/sat0-hir0/limn/commit/34f7081b3feb8b5c91cf0a30e4fa49314b001cdc))
* **ci:** unblock Link Check and Docs workflows ([2f1200e](https://github.com/sat0-hir0/limn/commit/2f1200ec39e3dd147e2e4f34a922f4aa2530ad45))
* **ci:** unblock Link Check and release-please after the public release ([546a286](https://github.com/sat0-hir0/limn/commit/546a2867ffca59b2fffdd8b3e7745223d85350f4))
* **doc-preprocessor:** hand-parse PreprocessorContext to tolerate null fields ([e04dcd0](https://github.com/sat0-hir0/limn/commit/e04dcd09ac8e9056a777db733e4802881741063f))
* **doc-preprocessor:** use CmdPreprocessor::parse_input for stdin ([bced174](https://github.com/sat0-hir0/limn/commit/bced1746e799a5d49e518c7cd0f4ac6938b4ab39))
* **doc-preprocessor:** walk Book as serde_json::Value, drop mdbook dep ([c1351b0](https://github.com/sat0-hir0/limn/commit/c1351b0630374a0059c8af662498ae522e961bc7))

## [0.1.0] - 2026-06-21

### Added

- Public release of the operational foundation.
- Three-crate workspace: `limn-core`, `limn-service`, `limn-ui` with a
  strictly one-way dependency direction.
- `cargo run -p limn-ui` opens a single window and renders the
  embedded Welcome document (or a `.md` passed as a CLI argument) as
  parsed blocks (read-only).
- `ARCHITECTURE.md` at the repository root, MADR-format ADRs under
  `docs/adr/`, and an mdBook documentation site backed by a
  self-hosted preprocessor (`limn-doc-preprocessor`) that validates
  path mentions, type references, ADR sequence, and ADR cross-refs
  at build time.
- OSS governance scaffolding: `LICENSE` (Apache-2.0), `NOTICE`,
  `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, issue / PR
  templates, label policy.
- Quality gates: `cargo-deny`, `gitleaks`, `release-please`,
  `debt-scan`, `lychee` link check, and `lefthook` hooks
  (pre-commit / pre-push / commit-msg).

### Known limitations

- License contamination via `gpui`'s transitive `sum_tree` →
  `ztracing` → `zlog` (GPL-3.0-or-later). See ADR-0003 and the
  upstream issue
  [zed-industries/zed#55470](https://github.com/zed-industries/zed/issues/55470).
- CI runs on Ubuntu only; Windows and macOS are verified manually.
- GPU required — a known `gpui` limitation prevents startup on older
  integrated GPUs.

<!-- [0.1.0]: https://github.com/sat0-hir0/limn/releases/tag/v0.1.0 — added when the v0.1.0 GitHub Release is published -->
