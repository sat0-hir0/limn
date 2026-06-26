//! Link index: parses `[[file]]`-style links and keeps a reverse map.
//!
//! Structure only — the link/backlink implementation is not yet provided.

use std::collections::HashMap;
use std::path::PathBuf;

/// Reverse index: file → list of files that reference it.
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
