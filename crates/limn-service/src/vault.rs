//! Vault: I/O against a folder full of `.md` files.
//!
//! Currently provides the read path: open a directory, find a Markdown
//! file, list every Markdown file for the palette's file search, and
//! hand a file to `limn_core::markdown::parse`. The write path is not
//! yet implemented.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use limn_core::block::Block;
use limn_core::markdown;

/// A parsed Markdown document and the path it came from.
#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub blocks: Vec<Block>,
}

/// Root of a vault — a directory full of `.md` files.
#[derive(Debug, Clone)]
pub struct Vault {
    pub root: PathBuf,
}

/// Things that can go wrong opening a document.
#[derive(Debug)]
pub enum OpenError {
    Io(io::Error),
    NoMarkdownFile { dir: PathBuf },
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::NoMarkdownFile { dir } => {
                write!(f, "no .md file found under {}", dir.display())
            }
        }
    }
}

impl std::error::Error for OpenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::NoMarkdownFile { .. } => None,
        }
    }
}

impl From<io::Error> for OpenError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl Vault {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Read the first `.md` file directly under `self.root` in
    /// alphabetical order, parse it, and return the [`Document`].
    ///
    /// Subdirectories are not searched — keep the surface small until
    /// a real walk is required.
    ///
    /// # Errors
    ///
    /// Returns [`OpenError::Io`] if the directory or file can't be read,
    /// or [`OpenError::NoMarkdownFile`] if no `.md` file exists under
    /// the root.
    pub fn open_first_md(&self) -> Result<Document, OpenError> {
        let path = first_md_in_dir(&self.root)?;
        Self::open_path(&path)
    }

    /// List every `.md` file under `self.root`, recursively, as paths
    /// relative to the root, sorted. Hidden entries (names starting
    /// with `.`, e.g. `.git` / `.obsidian`) are skipped. Unreadable
    /// subdirectories are skipped rather than failing the whole walk.
    ///
    /// This touches the filesystem; call it off the UI thread.
    ///
    /// # Errors
    ///
    /// Returns [`OpenError::Io`] if the root itself can't be read.
    pub fn list_md_files(&self) -> Result<Vec<PathBuf>, OpenError> {
        let mut out = Vec::new();
        let mut stack = vec![fs::read_dir(&self.root)?];
        while let Some(entries) = stack.pop() {
            for entry in entries.filter_map(Result::ok) {
                if entry.file_name().to_string_lossy().starts_with('.') {
                    continue;
                }
                let path = entry.path();
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                if file_type.is_dir() {
                    if let Ok(sub) = fs::read_dir(&path) {
                        stack.push(sub);
                    }
                } else if is_md(&path) {
                    if let Ok(rel) = path.strip_prefix(&self.root) {
                        out.push(rel.to_path_buf());
                    }
                }
            }
        }
        out.sort();
        Ok(out)
    }

    /// Read and parse a specific `.md` path.
    ///
    /// # Errors
    ///
    /// Returns [`OpenError::Io`] if the file can't be read.
    pub fn open_path(path: &Path) -> Result<Document, OpenError> {
        let text = fs::read_to_string(path)?;
        Ok(Document {
            path: path.to_path_buf(),
            blocks: markdown::parse(&text),
        })
    }
}

fn first_md_in_dir(dir: &Path) -> Result<PathBuf, OpenError> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_md(p))
        .collect();
    entries.sort();
    entries
        .into_iter()
        .next()
        .ok_or_else(|| OpenError::NoMarkdownFile {
            dir: dir.to_path_buf(),
        })
}

fn is_md(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("md")
}
