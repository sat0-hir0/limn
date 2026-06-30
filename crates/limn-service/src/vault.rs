//! Vault: I/O against a folder full of `.md` files.
//!
//! Currently provides the read path: open a directory, find a Markdown
//! file, and hand it to `limn_core::markdown::parse`. The write path is
//! not yet implemented.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use limn_core::block::Block;
use limn_core::markdown;

/// A parsed Markdown document and the path it came from.
#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub blocks: Vec<Block>,
}

/// One `.md` file discovered in a vault directory listing.
///
/// `name` is the file name including its extension (e.g. `notes.md`),
/// which is what the palette's fuzzy "Open File" search matches against
/// and displays. `path` is the absolute path used to open the file.
#[derive(Debug, Clone)]
pub struct VaultEntry {
    pub path: PathBuf,
    pub name: String,
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

/// Process-wide monotonic counter making each [`Vault::save_raw`] temp
/// file name unique within this process, on top of the PID. Two writers
/// racing on the same destination `path` therefore never share a temp
/// file. `Relaxed` is sufficient: we only need each `fetch_add` to return
/// a distinct value, not any cross-thread ordering of other memory.
static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

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

    /// List every `.md` file directly under `self.root`, sorted
    /// alphabetically by path.
    ///
    /// Subdirectories are not walked — this mirrors [`open_first_md`]'s
    /// shallow scan and keeps the surface small until M3 needs a real
    /// recursive walk. Non-`.md` files are excluded.
    ///
    /// Used by the palette's "Open File" fuzzy search (Wave 6): the
    /// returned [`VaultEntry`] list is the search corpus.
    ///
    /// [`open_first_md`]: Vault::open_first_md
    ///
    /// # Errors
    ///
    /// Returns [`OpenError::Io`] if the directory can't be read.
    pub fn list_md_files(&self) -> Result<Vec<VaultEntry>, OpenError> {
        let paths = list_md_paths(&self.root)?;
        Ok(paths
            .into_iter()
            .map(|path| {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();
                VaultEntry { path, name }
            })
            .collect())
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

    /// Write `text` back to `path` as raw UTF-8, replacing its contents.
    ///
    /// This is the write counterpart of [`open_path_raw`]: the editable
    /// view (ADR-0005) autosaves its `InputState` buffer verbatim, with
    /// no block round-trip. As with the read path, the actual `std::fs`
    /// call lives here so `limn-ui` never touches the filesystem
    /// directly (ADR-0002).
    ///
    /// The write is atomic from a *reader's* point of view: the text is
    /// first written in full to a sibling temporary file, then renamed
    /// over `path`. A concurrent reader therefore always observes either
    /// the previous contents or the complete new ones, never a
    /// half-written file. The temp file is created next to the
    /// destination so the final `rename` stays on one filesystem (a
    /// cross-device rename would fail).
    ///
    /// The temp file name is unique *per write*, not merely per process:
    /// it carries both the PID and a monotonic, process-wide counter
    /// (`path.limn-tmp.<PID>.<counter>`). This protects two writers that
    /// race on the *same* `path` within a single process — e.g. a
    /// debounced background autosave and a synchronous on-switch flush
    /// (ADR-0009) — from sharing one temp file. Without the counter, both
    /// would `File::create` the same temp path; one could truncate the
    /// other mid-write, and a partial temp could then be renamed over
    /// `path`, briefly leaving it empty or corrupt. With a per-write temp,
    /// each writer fills its *own* complete temp file and atomically
    /// renames it into place, so `path` always ends up holding one
    /// writer's full contents — never a torn mix. (Two concurrent writers
    /// still race on rename *order*, but the loser's content is complete,
    /// not damaged; last rename wins.)
    ///
    /// This does **not** guarantee crash durability: we deliberately do
    /// not `fsync` the temp file (or the directory) before renaming.
    /// Autosave fires frequently and unattended, and an `fsync` on every
    /// keystroke-burst would be a heavy, repeated I/O cost for little
    /// benefit — a power loss may lose the most recent unsynced save, but
    /// the on-disk file is never corrupted. Reader-atomicity, not crash
    /// durability, is the property we buy with the temp-file + rename.
    ///
    /// [`open_path_raw`]: Vault::open_path_raw
    ///
    /// # Errors
    ///
    /// Returns [`OpenError::Io`] if the temporary file can't be written
    /// or the rename into place fails.
    pub fn save_raw(path: &Path, text: &str) -> Result<(), OpenError> {
        // Co-locate the temp file with the destination so `rename` is an
        // intra-filesystem (atomic) move. Suffix it with the PID *and* a
        // process-wide monotonic counter so the name is unique per write:
        // the PID separates other processes, the counter separates other
        // writers inside *this* process that target the same `path` (see
        // the `save_raw` doc comment for the race this guards against).
        let mut tmp = path.as_os_str().to_owned();
        let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
        tmp.push(format!(".limn-tmp.{}.{seq}", std::process::id()));
        let tmp = PathBuf::from(tmp);

        // Write the full contents, then drop the handle before renaming.
        // If anything fails, best-effort remove the temp file so we don't
        // leave litter next to the document.
        if let Err(e) = write_temp(&tmp, text) {
            let _ = fs::remove_file(&tmp);
            return Err(e.into());
        }
        if let Err(e) = fs::rename(&tmp, path) {
            let _ = fs::remove_file(&tmp);
            return Err(e.into());
        }
        Ok(())
    }
}

/// Write `text` to `path` in full, returning once `write_all` has handed
/// every byte to the OS.
///
/// There is intentionally no `fsync`/`flush` here: `std::fs::File` does
/// no userspace buffering, so a `flush()` would be a no-op, and a real
/// `sync_all()` is deliberately omitted (see [`Vault::save_raw`] for the
/// durability rationale). The bytes are in the OS page cache when this
/// returns, which is enough for the subsequent rename to promote a
/// complete file for any concurrent reader.
fn write_temp(path: &Path, text: &str) -> io::Result<()> {
    use io::Write as _;

    let mut file = fs::File::create(path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

/// Every `.md` file directly under `dir`, sorted alphabetically by path.
///
/// The single shallow-scan + filter + sort used by both
/// [`Vault::list_md_files`] and [`first_md_in_dir`], so the two stay in
/// lockstep on what counts as a vault `.md` file.
fn list_md_paths(dir: &Path) -> Result<Vec<PathBuf>, OpenError> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md"))
        .collect();
    entries.sort();
    Ok(entries)
}

fn first_md_in_dir(dir: &Path) -> Result<PathBuf, OpenError> {
    list_md_paths(dir)?
        .into_iter()
        .next()
        .ok_or_else(|| OpenError::NoMarkdownFile {
            dir: dir.to_path_buf(),
        })
}
