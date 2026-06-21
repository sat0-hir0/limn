# Limn

[![CI](https://github.com/sat0-hir0/limn/actions/workflows/ci.yml/badge.svg)](https://github.com/sat0-hir0/limn/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/sat0-hir0/limn/branch/main/graph/badge.svg)](https://codecov.io/gh/sat0-hir0/limn)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)

Limn — a tool for organizing thought. Block editing, keyboard-first, `.md` storage, Rust native, AI integration.

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

## Contributing

PRs and issues are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for the development workflow, [Discussions](https://github.com/sat0-hir0/limn/discussions) for questions and ideas, and [SECURITY.md](SECURITY.md) for security reports. All participants are expected to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## License and acknowledgements

[Apache-2.0](LICENSE). Copyright details are in [NOTICE](NOTICE).

Thanks to [Zed Industries](https://zed.dev/) for building and open-sourcing [`gpui`](https://github.com/zed-industries/zed), the UI framework that powers Limn.
