//! ラウンドトリップ統合テスト: Markdown ⇄ ブロックツリー。
//!
//! 本プロジェクトの生命線 (docs/testing-strategy.md §4)。
//! M0 では雛形のみ。本実装は M2 で。

#[test]
#[ignore = "M0: 実装は M2"]
fn heading_roundtrip() {
    // 入力: "# Hello\n"
    // parse → serialize で元と一致すること
}

#[test]
#[ignore = "M0: 実装は M2"]
fn list_roundtrip() {
    // 入力: "- a\n- b\n  - c\n"
}

#[test]
#[ignore = "M0: 実装は M2"]
fn code_fence_roundtrip() {
    // 入力: "```rust\nfn main() {}\n```\n"
}

#[test]
#[ignore = "M0: 実装は M2"]
fn link_and_image_roundtrip() {
    // 入力: "[a](b.md) ![alt](c.png)\n"
}
