//! Markdown ⇄ ブロックツリーのシリアライズ。
//!
//! このモジュールのラウンドトリップ性は本プロジェクトの生命線
//! (= docs/testing-strategy.md §4)。
//!
//! M0: 関数シグネチャと testing-strategy で定義された仕様の雛形のみ。
//! 本体実装は M2 で。

use crate::block::Block;

/// Markdown 文字列をブロックツリーへパースする。
///
/// # Panics
///
/// 実装が未完成のため panic する (M0)。
#[must_use]
pub fn parse(_md: &str) -> Vec<Block> {
    unimplemented!("parse: 実装は M2")
}

/// ブロックツリーを Markdown 文字列へシリアライズする。
///
/// # Panics
///
/// 実装が未完成のため panic する (M0)。
#[must_use]
pub fn serialize(_blocks: &[Block]) -> String {
    unimplemented!("serialize: 実装は M2")
}

#[cfg(test)]
mod tests {
    /// ラウンドトリップ: parse → serialize で入力 md が完全一致する (空白の正規化は許容)。
    ///
    /// 入力候補 (M2 で網羅):
    /// - 見出し (`#` 〜 `######`)
    /// - リスト (順序付き / 順序なし / ネスト)
    /// - コードブロック (フェンス記法 ` ``` ` で囲まれた領域)
    /// - リンク / 画像
    /// - テーブル
    #[test]
    #[ignore = "M0: 実装は M2"]
    fn roundtrip_preserves_input() {
        // intentionally empty
    }
}
