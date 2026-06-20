//! Functional Core: 純粋ロジックのみ。`std` 以外の依存を入れない。
//!
//! モジュール:
//! - `block`: ブロックツリーの最小構造
//! - `markdown`: ブロックツリー ⇄ Markdown のシリアライズ (= 生命線)
//! - `completion`: 補完プロバイダの抽象

pub mod block;
pub mod completion;
pub mod markdown;
