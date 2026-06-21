# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.0]: https://github.com/sat0-hir0/limn/releases/tag/v0.1.0
