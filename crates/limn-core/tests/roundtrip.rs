//! Round-trip integration tests: Markdown ⇄ block tree.
//!
//! The project's lifeline (see docs/design/testing-strategy.md §4).
//! M0 ships scaffolding only; the real implementation lands in M2.

#[test]
#[ignore = "M0: implementation lands in M2"]
fn heading_roundtrip() {
    // Input: "# Hello\n"
    // parse → serialize must reproduce the original verbatim.
}

#[test]
#[ignore = "M0: implementation lands in M2"]
fn list_roundtrip() {
    // Input: "- a\n- b\n  - c\n"
}

#[test]
#[ignore = "M0: implementation lands in M2"]
fn code_fence_roundtrip() {
    // Input: "```rust\nfn main() {}\n```\n"
}

#[test]
#[ignore = "M0: implementation lands in M2"]
fn link_and_image_roundtrip() {
    // Input: "[a](b.md) ![alt](c.png)\n"
}
