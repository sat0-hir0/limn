//! Vault: フォルダ内の `.md` ファイル群の I/O。
//!
//! M0: 構造とシグネチャのみ。実装は M2 で。

use std::path::PathBuf;

/// Vault のルートディレクトリ。
#[derive(Debug, Clone)]
pub struct Vault {
    pub root: PathBuf,
}

impl Vault {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}
