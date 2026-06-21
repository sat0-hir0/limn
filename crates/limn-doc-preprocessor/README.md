# limn-doc-preprocessor

mdBook preprocessor that validates documentation integrity at build time.

## What it checks

| Check | Description |
|---|---|
| **path-mention** | Paths like `crates/…`, `docs/…`, `.github/workflows/…` referenced in docs must exist in the repo. |
| **type-mention** | `limn_core::Foo`, `limn_service::Bar`, `limn_ui::Baz` must appear as public identifiers in the respective crate source. |
| **adr-sequence** | `docs/adr/NNNN-*.md` files must form a contiguous sequence from 0001 with no gaps or duplicates. |
| **adr-crossref** | `ADR-NNNN` references in any doc must point to an existing ADR file. |

## Usage

The preprocessor runs automatically when you build the docs:

```sh
# Build the book (preprocessor invoked automatically)
mdbook build

# Serve locally with live reload
mdbook serve
```

The preprocessor binary must be on `$PATH` or reachable via
`cargo run -p limn-doc-preprocessor` (configured in `book.toml`).

## LIMN_DOC_LINT environment variable

| Value | Behaviour |
|---|---|
| *(unset)* or `error` | Validation failures abort the build with exit 1. |
| `warn` | Failures are printed as warnings; build continues (exit 0). |

Use `LIMN_DOC_LINT=warn` when iterating locally while doc files are
still being written:

```sh
LIMN_DOC_LINT=warn mdbook build
```

CI always runs in the default `error` mode.
