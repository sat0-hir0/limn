# Architecture Decision Records (ADR)

This directory records the significant architecture decisions made in Limn.

ADRs exist to preserve the rationale behind decisions — the history of why
something was decided, which alternatives were rejected, and what trade-offs
were accepted — information that is difficult to capture in code or comments.

---

## What is an ADR?

An Architecture Decision Record (ADR) captures an important software design
decision in a single Markdown document.

This project uses the [MADR](https://adr.github.io/madr/) format.

---

## Status values

| Status | Meaning |
|--------|---------|
| **Proposed** | Under consideration; not yet decided |
| **Accepted** | Adopted; the current policy |
| **Rejected** | Considered but not adopted |
| **Deprecated** | Previously adopted but no longer in use |
| **Superseded by ADR-NNNN** | Replaced by a later ADR |

---

## How to create a new ADR

1. Determine the next sequence number (highest existing number + 1) under `docs/adr/`
2. Copy `docs/adr/template.md` and save it as `docs/adr/NNNN-<slug>.md`
3. Fill in each section
4. Open a PR with the title `docs(adr): add ADR-NNNN <title>`
5. After review, change the Status from `Proposed` to `Accepted` and merge

---

## How to update or invalidate an existing ADR

- **Minor corrections**: typos and supplementary notes may be edited in place
- **Policy change**: create a new ADR and change the old ADR's Status to
  `Superseded by ADR-NNNN`. Do not delete the old ADR
- **Full deprecation**: change the Status to `Deprecated` and add a note explaining why

---

## ADR index

| Number | Title | Status | Date |
|--------|-------|--------|------|
| [ADR-0001](0001-adopt-gpui.md) | Adopt gpui as the GUI Framework | Accepted | 2026-06-21 |
| [ADR-0002](0002-three-crate-layered-architecture.md) | Adopt a Three-Crate Layered Architecture | Accepted | 2026-06-21 |
| [ADR-0003](0003-temporarily-accept-gpl-contamination.md) | Temporarily Accept GPL Contamination | Accepted | 2026-06-21 |
| [ADR-0004](0004-scope-skillshare-to-oss-operations.md) | Scope `.skillshare/` skills to OSS operating procedures | Accepted | 2026-06-23 |
| [ADR-0005](0005-app-icon-runtime-and-embed.md) | Register the app icon at runtime and embed it in the Windows `.exe`, defer packaging | Accepted | 2026-06-25 |
| [ADR-0006](0006-defer-runtime-icon-macos-windows.md) | Defer the macOS Dock icon until gpui exposes a runtime icon API | Accepted | 2026-06-25 |

> This index is maintained by hand. We may switch to auto-generation via a tool such as `adr-log` in the future.
