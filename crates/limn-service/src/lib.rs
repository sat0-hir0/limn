//! Imperative Shell の裏側: I/O / 索引 / AI 呼び出しなど副作用を持つレイヤ。
//!
//! - [`vault`]: フォルダ内の `.md` ファイル I/O
//! - [`index`]: リンク / バックリンク索引
//!
//! 主スレッドを待たせない (spec-handoff-gpui.md §5)。
//! 非同期処理は M2 以降で `tokio` 等を入れる。

pub mod index;
pub mod vault;

pub use vault::{Document, OpenError, Vault};
