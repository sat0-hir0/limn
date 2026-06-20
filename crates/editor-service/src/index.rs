//! リンク索引: `[[file]]` 形式のリンクを解析し、逆引きを保持する。
//!
//! M0: 構造のみ。実装は M6 (リンク・バックリンク Milestone) で。

use std::collections::HashMap;
use std::path::PathBuf;

/// 逆引き索引: ファイル → そのファイルを参照しているファイル群。
#[derive(Debug, Default)]
pub struct BacklinkIndex {
    inner: HashMap<PathBuf, Vec<PathBuf>>,
}

impl BacklinkIndex {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn backlinks_of(&self, path: &PathBuf) -> &[PathBuf] {
        self.inner.get(path).map_or(&[], Vec::as_slice)
    }
}
