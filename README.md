# Limn

[![CI](https://github.com/sat0-hir0/limn/actions/workflows/ci.yml/badge.svg)](https://github.com/sat0-hir0/limn/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/sat0-hir0/limn/branch/main/graph/badge.svg)](https://codecov.io/gh/sat0-hir0/limn)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)

> A keyboard-first, AI-integrated native Markdown editor — built in Rust on `gpui`.

"Limn" is the Old English verb for sketching the outline of something — making it visible by drawing it lightly. The experience the project is built around: as you keep writing, patterns of thought gradually take shape on the page.

## Status

> **Early-stage.** v0.1.0 publishes the foundation and operational scaffolding. At this point the editor is essentially a single read-only window that renders `.md` files. Editing (M2) and everything beyond will be layered in incrementally. Feedback and bug reports go to [Issues](https://github.com/sat0-hir0/limn/issues) / [Discussions](https://github.com/sat0-hir0/limn/discussions).

| Milestone | Goal | Status |
|---|---|---|
| M0 | Three-crate layout + test foundation + CI | ✅ |
| M1 | Open a `gpui` window and display `.md` read-only | ✅ |
| M2 | Editing + auto-save | not started |
| M3 | `/` command palette | not started |
| M4 | "Type and transform" (live Markdown rendering) | not started |
| M5 | AI integration (select → instruct) | not started |
| M6 | Links, backlinks, and graph view | not started |

## Quickstart

Requires [Rust stable](https://rustup.rs/) (pinned via `rust-toolchain.toml`).

```sh
git clone https://github.com/sat0-hir0/limn
cd limn
cargo run -p limn-ui
```

Running without arguments opens the built-in Welcome screen. Pass a `.md` path to open it directly:

```sh
cargo run -p limn-ui -- path/to/note.md
```

## Documentation

| Entry point | Contents |
|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Architecture overview, code map, cross-cutting concerns |
| [docs/adr/](docs/adr/) | Architecture Decision Records |
| [docs/design/](docs/design/) | Design documents (basic features, testing strategy) |
| [docs/development/](docs/development/) | Development process (git strategy, feature flags) |
| [docs/](docs/) | Full docs index |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |

## Project layout

Three crates with a strictly one-way dependency direction:

```
limn-ui ─→ limn-service ─→ limn-core
limn-ui ────────────────→ limn-core
```

- `limn-core` — Functional core: block tree, Markdown round-trip, completion. `std` only.
- `limn-service` — Imperative shell: `.md` I/O, link index, AI calls.
- `limn-ui` — `gpui` bindings: rendering, input handling, command palette.

For rationale and decision history see [ARCHITECTURE.md](ARCHITECTURE.md) and [ADR-0002](docs/adr/0002-three-crate-layered-architecture.md).

## Known limitations

- **GPL contamination via `gpui` transitive deps.** A transitive chain (`sum_tree` → `ztracing` → `zlog`) carries a `GPL-3.0-or-later` crate into the build. Limn itself is `Apache-2.0`. The code in question is a no-op log decorator, but it technically breaks copyleft consistency at distribution time. The project is tracking an upstream fix rather than patching locally; see [ADR-0003](docs/adr/0003-temporarily-accept-gpl-contamination.md) for the full rationale and resolution criteria.
- **Platform support.** CI currently runs on Ubuntu only. Windows and macOS are verified manually. A multi-platform matrix is planned for M2 and beyond.
- **GPU required.** A known upstream `gpui` issue prevents startup on older integrated GPUs.

## Contributing

PRs and issues are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for the development workflow, [Discussions](https://github.com/sat0-hir0/limn/discussions) for questions and ideas, and [SECURITY.md](SECURITY.md) for security reports. All participants are expected to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## License and acknowledgements

[Apache-2.0](LICENSE). Copyright details are in [NOTICE](NOTICE).

Thanks to [Zed Industries](https://zed.dev/) for building and open-sourcing [`gpui`](https://github.com/zed-industries/zed), the UI framework that powers Limn.
