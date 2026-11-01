//! Vault: I/O against a folder full of `.md` files.
//!
//! Read path: open a directory, find a Markdown file, and hand it to
//! `limn_core::markdown::parse`. Write path: [`Vault::save_raw`] writes
//! raw UTF-8 back through a temp file and an atomic rename, so an
//! interrupted save never leaves a half-written `.md` behind.

use std::fs;
use std::io::{self, Write};
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

    /// Read a `.md` path as raw UTF-8 text, without parsing it.
    ///
    /// This is the text the editor works on; [`Vault::save_raw`] writes
    /// it back unchanged.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`io::Error`] if the file can't be read
    /// or isn't valid UTF-8.
    pub fn open_path_raw(path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    /// Write `text` to `path` as raw UTF-8, replacing the file
    /// atomically.
    ///
    /// The bytes go to a temp file in the same directory first, are
    /// flushed to disk, and only then renamed over `path`. If anything
    /// fails before the rename, the original file is left untouched
    /// and the temp file is removed.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`io::Error`] if the temp file can't be
    /// written or the rename fails.
    pub fn save_raw(path: &Path, text: &str) -> io::Result<()> {
        let tmp = temp_path_for(path)?;
        let result = write_synced(&tmp, text).and_then(|()| fs::rename(&tmp, path));
        if result.is_err() {
            // Best effort: the original error is the one worth reporting.
            let _ = fs::remove_file(&tmp);
        }
        result
    }
}

/// Sibling temp path for an atomic save of `path`. Lives in the same
/// directory so the final rename never crosses a filesystem boundary.
fn temp_path_for(path: &Path) -> io::Result<PathBuf> {
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("not a file path: {}", path.display()),
        )
    })?;
    let mut tmp_name = std::ffi::OsString::from(".");
    tmp_name.push(name);
    tmp_name.push(format!(".limn-tmp-{}", std::process::id()));
    Ok(path.with_file_name(tmp_name))
}

fn write_synced(path: &Path, text: &str) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()
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
