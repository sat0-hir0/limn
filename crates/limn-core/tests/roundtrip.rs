//! Round-trip integration tests: Markdown ⇄ block tree.
//!
//! The project's lifeline (see docs/design/testing-strategy.md §4).
//! These tests are scaffolding for the round-trip contract; they stay
//! ignored until `markdown::serialize` is implemented.

#[test]
#[ignore = "serialize() not yet implemented"]
fn heading_roundtrip() {
    // Input: "# Hello\n"
    // parse → serialize must reproduce the original verbatim.
}

#[test]
#[ignore = "serialize() not yet implemented"]
fn list_roundtrip() {
    // Input: "- a\n- b\n  - c\n"
}

#[test]
#[ignore = "serialize() not yet implemented"]
fn code_fence_roundtrip() {
    // Input: "```rust\nfn main() {}\n```\n"
}

#[test]
#[ignore = "serialize() not yet implemented"]
fn link_and_image_roundtrip() {
    // Input: "[a](b.md) ![alt](c.png)\n"
}
