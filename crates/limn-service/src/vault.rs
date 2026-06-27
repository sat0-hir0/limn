//! Vault: I/O against a folder full of `.md` files.
//!
//! M1 adds the read path: open a directory, find a Markdown file, and
//! hand it to `limn_core::markdown::parse`. The write path lands in M2.

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

/// The raw, unparsed UTF-8 text of a Markdown file and the path it came
/// from.
///
/// The editable view (M2) seeds `gpui-component`'s `InputState` with raw
/// text rather than a parsed `Vec<Block>` (see ADR-0005). This is the
/// read path for that flow; `limn-ui` performs no direct `std::fs`
/// (ADR-0002), so the raw read lives here.
#[derive(Debug, Clone)]
pub struct RawDocument {
    pub path: PathBuf,
    pub text: String,
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
    /// M3 needs a real walk.
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

    /// Read a specific `.md` path as raw UTF-8 text, without parsing it
    /// into blocks.
    ///
    /// Used by the editable view (ADR-0005): the editor seeds its
    /// `InputState` with the file's verbatim text. Keeping the read here
    /// preserves the rule that `limn-ui` never touches `std::fs`
    /// directly (ADR-0002).
    ///
    /// # Errors
    ///
    /// Returns [`OpenError::Io`] if the file can't be read.
    pub fn open_path_raw(path: &Path) -> Result<RawDocument, OpenError> {
        let text = fs::read_to_string(path)?;
        Ok(RawDocument {
            path: path.to_path_buf(),
            text,
        })
    }
}

fn first_md_in_dir(dir: &Path) -> Result<PathBuf, OpenError> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md"))
        .collect();
    entries.sort();
    entries
        .into_iter()
        .next()
        .ok_or_else(|| OpenError::NoMarkdownFile {
            dir: dir.to_path_buf(),
        })
}
